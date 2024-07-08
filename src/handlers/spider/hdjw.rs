use std::collections::{HashMap, HashSet};

use crate::{
    app_result::{AppResult, AppState},
    dtos::spider::hdjw::{
        GetClassStartDateReq, GetClassTableReq, GetEmptyRoomReq, GetExamArrangeReq, GetGradeReq, GetMustGradeReq, GetRawGradeReq
    },
    entities::{
        back::course::CourseInfo,
        spider::{
            class_table::{ClassTableRes, SpiderCourseInfo},
            empty_room::{EmptyRoomRes, SpiderEmptyRoom},
            exam::{ExamArrangeRes, SpiderComputerExamArrange, SpiderExamArrange},
            global_static::{ClassStartDateMap, EndMap, StartMap},
            grade::{
                F64OrString, GradeChartRes, GradeRankRes, GradeRankResSemesters,
                GradeRankResSemestersItem, GradeRankResTotal, GradeRes, SpiderGrade,
                SpiderGradeChart, SpiderGradeRank, U32OrString,
            },
            raw_grade::{
                raw_grade_item_struct_to_map, RawGradeRes, RawGradeResItem, SpiderRawGrade,
            },
        },
    },
    extractors::Query,
    utils::{
        jwt::{parse, parse_stu_id},
        request::{spider, spider_data},
    },
};
use axum::extract::{Extension, State};
use tokio::try_join;

pub async fn get_class_table_handler(
    State(data): AppState,
    Query(req): Query<GetClassTableReq>,
    Extension(token): Extension<String>,
) -> AppResult {
    let (mini_bind_id, stu_id) = parse(&token)?;
    // 后端返回自定义课程
    let back_res = sqlx::query_as!(
        CourseInfo,
        r#"
        SELECT id, classname, location, teachers, week, day, section FROM mini_course WHERE xn = ? AND xq = ? AND mini_bind_id = ? AND deleted_at IS NULL
        "#,
        req.xn,
        req.xq - 1,
        mini_bind_id,
    )
    .fetch_all(&data.db)
    .await?; // the type of back_res is Vec<CourseInfo>
             // 爬虫返回教务课程
    let params = [("xn", req.xn.to_string()), ("xq", req.xq.to_string()), ("stuid", stu_id)];
    let spider_res: Vec<SpiderCourseInfo> = spider_data("/bks/classtable", &params).await?;
    // 合并两个数据源
    let mut res = Vec::with_capacity(back_res.len() + spider_res.len());
    for item in back_res {
        let section: Vec<&str> = item.section.split(',').collect();
        let temp = ClassTableRes {
            id: item.id.to_string(),
            class_id: "自定义课程".to_string(),
            classname: item.classname,
            location: item.location.unwrap_or("".to_string()),
            teachers: item.teachers.unwrap_or("".to_string()),
            week: item.week,
            day: item.day,
            start_time: StartMap.get(&section[0].parse::<u32>().unwrap()).unwrap().to_string(),
            section: item.section.clone(), // 由于存在不可变借用，所以不能直接将所有权move到vec中，只能clone
            end_time: EndMap
                .get(&section[section.len() - 1].parse::<u32>().unwrap())
                .unwrap()
                .to_string(),
            _type: 2,
            skqk: "".to_string(),
        };
        res.push(temp);
    }
    for item in spider_res {
        let temp = ClassTableRes {
            id: item.id,
            class_id: item.ktmc_name,
            classname: item.kc_name,
            location: item.js_name.unwrap_or("".to_string()),
            teachers: item.teachernames,
            week: item.pkzcmx,
            day: std::str::from_utf8(&item.pksj.as_bytes()[..1]).unwrap().to_string(), // 考虑性能不采用迭代器写法，选取字符串第一个字节，转换为utf8编码，再转换为字符串
            section: item.jczy01501ids,
            start_time: item.djkssj,
            end_time: item.djjssj,
            _type: 1,
            skqk: item.skqk,
        };
        res.push(temp);
    }
    // 数据去重，根据id去重
    let mut seen = HashSet::new();
    res.retain(|item| seen.insert(item.id.clone()));
    // 返回数据
    Ok(res.into())
}

pub async fn get_class_start_date_handler(Query(req): Query<GetClassStartDateReq>) -> AppResult {
    let key = format!("{}-{}", req.xn, req.xq);
    match ClassStartDateMap.get(key.as_str()) {
        Some(res) => Ok(res.to_string().into()),
        None => Ok(().into()),
    }
}

pub async fn get_grade_handler(
    Query(req): Query<GetGradeReq>,
    Extension(token): Extension<String>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let params = [("xn", req.xn.to_string()), ("xq", req.xq.to_string()), ("stuid", stu_id)];
    let spider_res: SpiderGrade = spider_data("/bks/grade", &params).await?;

    if spider_res.rowCount == 0 {
        return Ok(().into()); // 返回数据为空，直接返回空数据
    }

    let mut res = Vec::with_capacity(spider_res.rowCount as usize);

    for item in spider_res.items.into_iter().rev()
    // 参照原有中间件代码，将数据反转
    {
        let temp = GradeRes {
            number: item.kcbh,
            serial: format!("{}/{}", item.kcxzname, item.kclbname),
            name: item.kcname,
            college: item.kkdwname,
            examType: item.ksxzname,
            credit: item.xf,
            grade: item.zcj,
        };
        res.push(temp);
    }
    Ok(res.into())
}

pub async fn get_must_grade_handler(Query(req): Query<GetMustGradeReq>, Extension(token): Extension<String>) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let xn = req.xn.to_string();
    let params_1 = [("stuid", stu_id.clone()), ("xn", xn.clone()), ("xq", "1".to_string())];
    let params_2 = [("stuid", stu_id.clone()), ("xn", xn.clone()), ("xq", "2".to_string())];
    let params_3 = [("stuid", stu_id), ("xn", xn), ("xq", "3".to_string())];
    let (spider_1, spider_2, spider_3): (SpiderGrade ,SpiderGrade ,SpiderGrade ) = try_join!(
        spider_data("/bks/grade", &params_1),
        spider_data("/bks/grade", &params_2),
        spider_data("/bks/grade", &params_3)
    )?;
    let mut scores = 0.0;
    let mut credits = 0.0;
    for item in spider_1.items.iter().chain(spider_2.items.iter()).chain(spider_3.items.iter()) {
        // 如果kcxzname是必修才加入计算
        if item.kcxzname == "必修" {
            scores += item.zcj as f64 * item.xf;
            credits += item.xf;
        }
    }
    let weighted_avg = scores / credits;
    // 转换成String，只保留两位小数
    let weighted_avg = format!("{:.2}", weighted_avg);
    Ok(weighted_avg.into())
}

pub async fn get_grade_rank_handler(Extension(token): Extension<String>) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let params = [("stuid", stu_id), ("type", 1.to_string())];
    let spider_res: SpiderGradeRank = spider("/bks/grade/analyze", &params).await?;
    // res的total字段
    let res_total = GradeRankResTotal {
        arithmeticAvg: spider_res.report[0].ARITHMETIC_AVG,
        arithmeticAvgRank: spider_res.report[0].ARITHMETIC_AVG_RANK,
        coreArithmeticAvg: spider_res.report[0].CORE_ARITHMETIC_AVG,
        coreArithmeticAvgRank: spider_res.report[0].CORE_ARITHMETIC_AVG_RANK,
        coreWeightAvg: spider_res.report[0].CORE_WEIGHTED_AVG,
        coreWeightAvgRank: spider_res.report[0].CORE_WEIGHTED_AVG_RANK,
        GPA: spider_res.report[0].GPA,
        GPARank: spider_res.report[0].GPA_RANK,
        weightAvg: spider_res.report[0].WEIGHTED_AVG,
        weightAvgRank: spider_res.report[0].WEIGHTED_AVG_RANK,
    };
    // res的semesters字段
    let mut res_semesters = Vec::with_capacity(spider_res.semesters.len()); // 返回结果的semesters字段
    let mut year_map: HashMap<u32, Vec<GradeRankResSemestersItem>> = HashMap::new(); // 用年份来组织数据
    for item in spider_res.semesters.into_iter().rev()
    // 参照原有中间件代码，将数据反转
    {
        let xn = item.XN.parse::<u32>().unwrap();
        let name = match item.XQ {
            Some(xq) => match xq.parse::<u32>().unwrap() {
                1 => "秋季学期",
                2 => "春季学期",
                3 => "夏季学期",
                _ => "未知学期", // 不会出现，只是为了穷举所有情况，使编译通过
            },
            None => "全部学期",
        };
        let temp = GradeRankResSemestersItem {
            coreWeightAvg: match item.CORE_WEIGHTED_AVG {
                F64OrString::F64(x) => x.to_string(),
                F64OrString::String(x) => x,
            },
            coreWeightAvgRank: match item.CORE_WEIGHTED_AVG_RANK {
                U32OrString::U32(x) => x.to_string(),
                U32OrString::String(x) => x,
            },
            GPA: item.GPA,
            GPARank: item.GPA_RANK,
            weightAvg: item.WEIGHTED_AVG,
            weightAvgRank: item.WEIGHTED_AVG_RANK,
            name: name.to_string(),
        };
        // 如果用年份索引year_map没有找到对应的数据，则新建一个，否则将temp插入到对应的vec中
        match year_map.get_mut(&xn) {
            Some(x) => x.push(temp),
            None => {
                year_map.insert(xn, vec![temp]);
            }
        }
    }
    year_map.into_iter().for_each(|(k, v)| {
        let temp = GradeRankResSemesters { items: v, year: format!("{}-{}", k, k + 1) };
        res_semesters.push(temp);
    });

    res_semesters.sort_by(|a, b| b.year.cmp(&a.year)); // 按年份降序排列

    let res = GradeRankRes { total: res_total, semesters: res_semesters };
    Ok(res.into())
}

pub async fn get_raw_grade_handler(
    Query(req): Query<GetRawGradeReq>,
    Extension(token): Extension<String>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let params = [("xn", req.xn.to_string()), ("xq", req.xq.to_string()), ("stuid", stu_id)];
    let spider_res: SpiderRawGrade = match spider_data("/bks/raw/grade", &params).await {
        Ok(x) => x,
        Err(_) => return Ok(().into()), // 返回数据为空，直接返回空数据
    };

    let mut res: Vec<RawGradeRes> = Vec::with_capacity(spider_res.cjxmcj.rowCount as usize);

    for item in spider_res.cjxmcj.items.into_iter().rev()
    // 参照原有中间件代码，将数据反转
    {
        let mut temp = vec![];
        // 构建HashMap方便后续程序逻辑处理
        let map = raw_grade_item_struct_to_map(&item);
        map.into_iter().for_each(|(k, v)| {
            temp.push(RawGradeResItem {
                name: spider_res
                    .cjxmInfo
                    .iter()
                    .find(|&item| item.xmbh == k)
                    .unwrap() // 不可能找不到，不用担心会panic
                    .xmmc
                    .to_string(),
                grade: v,
            });
            // 对temp按照k值大小升序排列
            temp.sort_by(|a, b| a.name.cmp(&b.name));
        });
        // cjxm1的特例，留下方便以后理解，为了避免重复代码，所以利用HashMap来复用代码
        // if let Some(x) = item.cjxm1 {
        //     temp.push(RawGradeResItem {
        //         name: spider_res
        //             .cjxmInfo
        //             .iter()
        //             .find(|&item| item.xmbh == "1")
        //             .unwrap() // 不可能找不到
        //             .xmmc
        //             .to_string(),
        //         grade: x,
        //     });
        // }
        res.push(RawGradeRes { name: item.kc_name, item: temp })
    }

    Ok(res.into())
}

pub async fn get_grade_chart_handler(Extension(token): Extension<String>) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let params = [("stuid", stu_id)];
    let spider_res: Vec<SpiderGradeChart> = spider_data("/bks/grade/chart", &params).await?;

    let mut res: GradeChartRes = GradeChartRes::default();
    for item in spider_res {
        let xn = item.XN;
        let name = match item.XQ.as_str() {
            "1" => format!("{}学年 秋季学期", xn),
            "2" => format!("{}学年 春季学期", xn),
            "3" => format!("{}学年 夏季学期", xn),
            _ => format!("{}学年 全部学期", xn), // 不会出现，只是为了穷举所有情况，使编译通过
        };
        res.semester.push(name);
        res.GPA.push(item.GPA);
        res.GPARank.push(item.GPA_RANK);
        res.weightAvg.push(item.WEIGHTED_AVG);
        res.weightAvgRank.push(item.WEIGHTED_AVG_RANK);
        res.CoreWeightAvg.push(match item.CORE_WEIGHTED_AVG {
            F64OrString::F64(x) => x,
            F64OrString::String(_) => 0.0,
        });
        res.CoreWeightAvgRank.push(match item.CORE_WEIGHTED_AVG_RANK {
            U32OrString::U32(x) => x,
            U32OrString::String(_) => 0,
        });
    }
    Ok(res.into())
}

pub async fn get_exam_arrange_handler(
    Query(req): Query<GetExamArrangeReq>,
    Extension(token): Extension<String>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let params = [("xn", req.xn.to_string()), ("xq", req.xq.to_string()), ("stuid", stu_id)];
    let spider_res: SpiderExamArrange = spider_data("/bks/exam/schedule", &params).await?;

    let mut res = Vec::with_capacity(spider_res.rowCount as usize);
    for item in spider_res.items {
        let temp = ExamArrangeRes {
            number: item.kcbh,
            name: item.kc_name,
            classroom: item.kcmc_name,
            startTime: item.kskssj,
            endTime: item.ksjssj,
            seat: item.zwh.unwrap_or_default(),
        };
        res.push(temp);
    }
    //TODO 将vec按startTime排序，由于现在爬虫无数据返回，所以暂时不实现
    Ok(res.into())
}

pub async fn get_computer_exam_arrange_handler(
    Query(req): Query<GetExamArrangeReq>,
    Extension(token): Extension<String>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let params = [("xn", req.xn.to_string()), ("xq", req.xq.to_string()), ("stuid", stu_id)];
    //TODO 结构正确性有待验证
    let spider_res: Vec<SpiderComputerExamArrange> =
        spider_data("/bks/jkexam/schedule", &params).await?;

    let mut res = Vec::with_capacity(spider_res.len());
    for item in spider_res {
        let temp = ExamArrangeRes {
            number: item.kcbh,
            name: item.kc_name,
            classroom: item.jf_name,
            startTime: format!("{} {}", item.jkrq, item.kssj),
            endTime: format!("{} {}", item.jkrq, item.jssj),
            seat: item.jwbh,
        };
        res.push(temp);
    }
    //TODO 将vec按startTime排序，由于现在爬虫无数据返回，所以暂时不实现
    Ok(res.into())
}

pub async fn get_empty_room_handler(Query(req): Query<GetEmptyRoomReq>) -> AppResult {
    let params = [
        ("build_id", req.buildId),
        ("day", req.day.to_string()),
        ("jc", req.jc),
        ("week", req.week.to_string()),
        ("xn", req.xn.to_string()),
        ("xq", req.xq.to_string()),
    ];
    let spider_res: SpiderEmptyRoom = spider_data("/freeroom/list", &params).await?;

    let mut res = Vec::with_capacity(spider_res.rowCount as usize);

    for item in spider_res.items {
        let temp = EmptyRoomRes {
            name: item.js_name,
            _type: item.classroomtypename,
            seat: item.yxzw,
            examSeat: item.kszw,
        };
        res.push(temp);
    }

    Ok(res.into())
}
