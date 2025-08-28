use crate::app_error::AppError;
use crate::dtos::spider::hdjw::GetCourseInfoReq;
use crate::entities::back::course::CourseInfo;
use crate::entities::back::flex_time::{self, FlexTime};
use crate::entities::spider::course_detail::{CourseDetailRes, SpiderCourseDetail};
use crate::entities::spider::grade::{GradeInfo, SpiderGradeInfo};
use crate::utils::semester::{get_class_start_date_by_xnxq, get_now_xnxq};
use crate::{
    app_result::{AppResult, AppState},
    dtos::spider::hdjw::{
        GetClassStartDateReq, GetClassTableReq, GetEmptyRoomReq, GetExamArrangeReq, GetGradeReq,
        GetRawGradeReq,
    },
    entities::{
        back::course::CustomizeCourseInfo,
        spider::{
            class_table::SpiderCourseInfo,
            empty_room::{EmptyRoomRes, SpiderEmptyRoom},
            exam::{ExamArrangeRes, SpiderComputerExamArrange, SpiderExamArrange},
            global_static::{EndMap, StartMap},
            grade::{
                F64OrString, GradeChartRes, GradeRankRes, GradeRankResSemesters,
                GradeRankResSemestersItem, GradeRankResTotal, SpiderGradeChart, SpiderGradeRank,
                U32OrString,
            },
            raw_grade::{
                raw_grade_item_struct_to_map, RawGradeRes, RawGradeResItem, SpiderRawGrade,
            },
        },
    },
    extractors::Query,
    utils::{
        jwt::{parse, parse_stu_id},
        request::spider_data,
    },
};
use anyhow::anyhow;
use axum::extract::{Extension, State};
use regex::Regex;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::vec;
use tokio::try_join;
use tracing::error;

pub async fn get_course_info_handler(
    Query(req): Query<GetCourseInfoReq>,
    Extension(token): Extension<String>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    if req.keyword.is_empty() {
        return Ok(().into());
    }
    let data: SpiderCourseDetail = spider_data(
        "/bks/courseinfo",
        &[
            ("xn", req.xn.to_string()),
            ("xq", req.xq.to_string()),
            ("stuid", stu_id),
            ("prompt", req.keyword),
        ],
    )
    .await?;
    let mut res = Vec::new();
    for item in data.items {
        let temp = CourseDetailRes {
            classID: item.kcbh,
            serial: item.kclb_name,
            name: item.kcmc_name,
            examType: item.khfs_name.unwrap_or("暂无数据".to_string()),
            className: item.ktmc_name,
            teacher: item.skls_name,
            people: item.xkrs,
            credit: item.zxf,
            school: item.zxs,
            place: item.xq_name,
            academy: item.kkdw_name,
        };
        res.push(temp);
    }
    // 兼容前端接口
    let res = json!({"data":{"hdjw": {"course": res}}});
    Ok(res.into())
}

pub async fn get_class_table_handler(
    State(data): AppState,
    Query(req): Query<GetClassTableReq>,
    Extension(token): Extension<String>,
) -> AppResult {
    let (mini_bind_id, stu_id) = parse(&token)?;
    // 后端返回自定义课程
    let back_res = sqlx::query_as!(
        CustomizeCourseInfo,
        r#"
        SELECT id, classname, location, teachers, week, day, section FROM mini_course WHERE xn = ? AND xq = ? AND mini_bind_id = ? AND deleted_at IS NULL
        "#,
        req.xn,
        req.xq - 1,
        mini_bind_id,
    )
    .fetch_all(&data.db)
    .await?;
    // 爬虫返回教务课程
    let params = [("xn", req.xn.to_string()), ("xq", req.xq.to_string()), ("stuid", stu_id)];
    let spider_res: Vec<SpiderCourseInfo> = spider_data("/bks/classtable", &params).await?;
    // 合并两个数据源
    // 注意 res 里的每个元素都对应课程表中的一个格子，至于格子之间的合并什么的交给前端
    let mut res = Vec::new();
    for item in back_res {
        let times: Vec<&str> = item.section.split(',').collect();
        let weeks_from_str: Vec<&str> = item.week.split(',').collect();
        let mut weeks = Vec::new();
        for week in weeks_from_str {
            let week = week.trim().parse::<u8>().map_err(|e| anyhow!("课程周次解析失败 {}", e))?;
            weeks.push(week);
        }
        let day = item.day.parse::<u8>().map_err(|e| anyhow!("课程星期解析失败 {}", e))?;
        for time in &times {
            let time = time.trim().parse::<u8>().map_err(|e| anyhow!("课程节次解析失败 {}", e))?;
            let tmp = CourseInfo {
                course_name: item.classname.clone(),
                course_id: None,
                _type: "自定义课程".to_string(),
                class_name: None,
                place: item.location.clone(),
                area: None,
                teacher: item.teachers.clone(),
                weeks: weeks.clone(),
                credit: None,
                extra: None,
                customize_id: item.id as i32,
                day,
                time,
            };
            res.push(tmp);
        }
    }
    for item in spider_res {
        // 这里主要处理上课时间，每个时间同时对应一个地点。
        // 考虑以 周几+节次+地点作为 key，周数作为 value，用 set 存储。这样做是为了合并一些可以合并的时间（hdjw 可能分开写），并且区分不同地点的课程
        let places = item.skddmc.split(';').collect::<Vec<_>>();
        let mut record = HashMap::new();
        let detail_times = item.sktime.split(';');
        for (i, time) in detail_times.into_iter().enumerate() {
            let re = Regex::new(r"周(.)第([\d、]+)节.*第(.*)周").unwrap();
            let caps = re.captures(time).ok_or(anyhow!("解析课程时间失败"))?;
            let day = caps
                .get(1)
                .ok_or(anyhow!("解析课程时间失败"))?
                .as_str()
                .chars()
                .next()
                .unwrap();
            let day = match day {
                '一' => 1,
                '二' => 2,
                '三' => 3,
                '四' => 4,
                '五' => 5,
                '六' => 6,
                '日' | '七' => 7,
                _ => return Err(AppError::AnyHow(anyhow!("未知的星期字符：{}", day))),
            };
            let times = caps
                .get(2)
                .ok_or("解析课程时间失败")?
                .as_str()
                .split('、')
                .collect::<Vec<_>>();
            let weeks = caps.get(3).ok_or("解析课程时间失败")?.as_str();
            let place = places.get(i).ok_or(anyhow!("解析课程地点失败"))?;
            for week_range in weeks.split(',') {
                let parts = week_range.split('-').collect::<Vec<_>>();
                let l = parts
                    .get(0)
                    .ok_or(anyhow!("解析课程周次失败"))?
                    .parse::<u8>()
                    .map_err(|e| anyhow!("解析课程周次失败 {}", e))?;
                let r = if parts.len() == 1 {
                    l
                } else {
                    parts
                        .get(1)
                        .ok_or(anyhow!("解析课程周次失败"))?
                        .parse::<u8>()
                        .map_err(|e| anyhow!("解析课程周次失败 {}", e))?
                };
                for time in times.iter() {
                    let time =
                        time.parse::<u8>().map_err(|e| anyhow!("解析课程时间失败 {}", e))?;
                    let set = record.entry((day, time, *place)).or_insert(HashSet::new());
                    for week in l..=r {
                        set.insert(week);
                    }
                }
            }
        }
        // record 里的一个元素就对应于课程表中的一个格子
        for ((day, time, place), weeks) in record {
            let tmp = CourseInfo {
                course_name: item.kc_mc.clone(),
                course_id: Some(item.kch.clone()),
                _type: item.kcxz.clone(),
                class_name: Some(item.kt_mc.clone()),
                place: match place {
                    "无" => None,
                    _ => Some(place.to_string()),
                },
                area: Some(item.skxqmc.clone()),
                teacher: Some(item.jg0101mc.clone()),
                weeks: weeks.into_iter().collect(),
                credit: Some(item.xf),
                extra: item.fzmc.clone(),
                customize_id: -1,
                day,
                time,
            };
            res.push(tmp);
        }
    }
    // 处理调休。目前的设计也会调休自定义课程，可能不是一个好做法？
    let flex_time =
        sqlx::query!("SELECT value FROM mini_configs WHERE `key` = ? AND enabled = 1", "flexTime")
            .fetch_one(&data.db)
            .await?
            .value;
    let mut flex_time: Vec<FlexTime> = serde_json::from_str(&flex_time).map_err(|_| {
        error!("解析调休信息失败");
        anyhow::anyhow!("解析调休信息失败")
    })?;
    // 只选择当前学年/学期的调休
    flex_time.retain(|x| x.time.xn == req.xn && x.time.xq == req.xq);
    for flex in flex_time {
        // 先把 to 那天的课程全部毙掉
        for item in res.iter_mut() {
            if item.day != flex.to.day {
                continue;
            }
            item.weeks.retain(|&x| x != flex.to.week);
        }
        // 然后找 from 那天的课，全部加入到 to 那天的课程中
        // 加入到 to 的时候直接创建一个新的课程，和原来的课程做一个区分，这样前端显示起来会好一点
        let mut new_items = Vec::new();
        for item in res.iter_mut() {
            if item.day != flex.from.day || !item.weeks.contains(&flex.from.week) {
                continue;
            }
            let mut new_item = item.clone();
            new_item.day = flex.to.day;
            new_item.weeks = vec![flex.to.week];
            new_item.extra = Some(flex.desc.clone());
            new_items.push(new_item);
        }
        // 再把 from 那天的课程也全部毙掉。注意这三个步骤是有顺序的，不能乱。
        for item in res.iter_mut() {
            if item.day != flex.from.day {
                continue;
            }
            item.weeks.retain(|&x| x != flex.from.week);
        }
        res.extend(new_items);
    }
    // 由于调休那里的操作，会使得某些课程的周次变成空的，所以需要清理一下
    res.retain(|item| !item.weeks.is_empty());
    // 再给 weeks 排个序
    for item in res.iter_mut() {
        item.weeks.sort_unstable();
    }
    Ok(res.into())
}

pub async fn get_class_start_date_handler(Query(req): Query<GetClassStartDateReq>) -> AppResult {
    Ok(get_class_start_date_by_xnxq(req.xn, req.xq).unwrap_or_default().into())
}

pub async fn get_grade_handler(
    Query(req): Query<GetGradeReq>,
    Extension(token): Extension<String>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let params = [("xn", req.xn.to_string()), ("xq", req.xq.to_string()), ("stuid", stu_id)];
    let spider_res: Vec<SpiderGradeInfo> = spider_data("/bks/grade", &params).await?;

    if spider_res.is_empty() {
        return Ok(().into()); // 返回数据为空，直接返回空数据
    }

    let mut res = Vec::new();

    // 新的教务系统的成绩出现顺序并不是按成绩公布时间排序的了，所以正着遍历和倒着遍历没什么区别
    for item in spider_res.into_iter().rev()
    // 参照原有中间件代码，将数据反转
    {
        let tmp = GradeInfo {
            course_id: item.kch,
            course_name: item.kc_mc,
            credit: item.xf,
            course_type1: item.kcsx,
            course_type2: item.kcxzmc,
            gpa: item.jd,
            score: item.zcj,
        };
        res.push(tmp);
    }
    Ok(res.into())
}

// pub async fn get_must_grade_handler(Extension(token): Extension<String>) -> AppResult {
//     let stu_id = parse_stu_id(&token)?;
//     let (xn, xq) = get_now_xnxq();
//     let xn = if xq == 1 { xn - 1 } else { xn }; // 如果是秋季学期，学年减一
//     let xn = xn.to_string();
//     let params_1 = [("stuid", stu_id.clone()), ("xn", xn.clone()), ("xq", "1".to_string())];
//     let params_2 = [("stuid", stu_id.clone()), ("xn", xn.clone()), ("xq", "2".to_string())];
//     let params_3 = [("stuid", stu_id), ("xn", xn), ("xq", "3".to_string())];
//     let (spider_1, spider_2, spider_3): (SpiderGrade, SpiderGrade, SpiderGrade) = try_join!(
//         spider_data("/bks/grade", &params_1),
//         spider_data("/bks/grade", &params_2),
//         spider_data("/bks/grade", &params_3)
//     )?;
//     let mut scores = 0.0;
//     let mut credits = 0.0;
//     let mut count = 0;
//     for item in spider_1
//         .items
//         .iter()
//         .chain(spider_2.items.iter())
//         .chain(spider_3.items.iter())
//     {
//         // 如果遇到了缓考导致成绩为0
//         if item.zcj == 0 {
//             continue;
//         }
//         // 如果kcxzname是必修才加入计算
//         if item.kcxzname == "必修" {
//             scores += item.zcj as f64 * item.xf;
//             credits += item.xf;
//             count += 1;
//         }
//     }
//     if credits == 0.0 {
//         return Ok(().into()); // 返回的data为null值
//     }
//     let weighted_avg = scores / credits;
//     // 转换成String，只保留两位小数
//     let weighted_avg = format!("{:.2}", weighted_avg);
//     let res = [weighted_avg, count.to_string()];
//     Ok(res.into())
// }

pub async fn get_grade_rank_handler(Extension(token): Extension<String>) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let params = [("stuid", stu_id), ("type", 1.to_string())];
    let spider_res: SpiderGradeRank = spider_data("/bks/grade/analyze", &params).await?;
    // res的total字段
    if spider_res.report.is_empty() {
        return Err("汇总数据为空".into());
    }
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

// pub async fn get_raw_grade_handler(
//     Query(req): Query<GetRawGradeReq>,
//     Extension(token): Extension<String>,
// ) -> AppResult {
//     let stu_id = parse_stu_id(&token)?;
//     let params = [("xn", req.xn.to_string()), ("xq", req.xq.to_string()), ("stuid", stu_id)];
//     let spider_res: SpiderRawGrade = match spider_data("/bks/raw/grade", &params).await {
//         Ok(x) => x,
//         Err(e) => {
//             error!("spider_data raw_grade: raw grade error: {}", e);
//             return Ok(().into());
//         } // 返回数据为空，直接返回空数据
//     };

//     let mut res: Vec<RawGradeRes> = Vec::with_capacity(spider_res.cjxmcj.rowCount as usize);

//     for item in spider_res.cjxmcj.items.into_iter().rev()
//     // 参照原有中间件代码，将数据反转
//     {
//         let mut temp = vec![];
//         // 构建HashMap方便后续程序逻辑处理
//         let map = raw_grade_item_struct_to_map(&item);
//         map.into_iter().for_each(|(k, v)| {
//             temp.push(RawGradeResItem {
//                 name: spider_res
//                     .cjxmInfo
//                     .iter()
//                     .find(|&item| item.xmbh == k)
//                     .unwrap() // 不可能找不到，不用担心会panic
//                     .xmmc
//                     .to_string(),
//                 grade: v,
//             });
//             // 对temp按照k值大小升序排列
//             temp.sort_by(|a, b| a.name.cmp(&b.name));
//         });
//         // cjxm1的特例，留下方便以后理解，为了避免重复代码，所以利用HashMap来复用代码
//         // if let Some(x) = item.cjxm1 {
//         //     temp.push(RawGradeResItem {
//         //         name: spider_res
//         //             .cjxmInfo
//         //             .iter()
//         //             .find(|&item| item.xmbh == "1")
//         //             .unwrap() // 不可能找不到
//         //             .xmmc
//         //             .to_string(),
//         //         grade: x,
//         //     });
//         // }
//         res.push(RawGradeRes { name: item.kc_name, item: temp })
//     }

//     Ok(res.into())
// }

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

pub async fn get_empty_room_handler(
    Query(req): Query<GetEmptyRoomReq>,
    Extension(token): Extension<String>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let params = [
        ("build_id", req.buildId),
        ("day", req.day.to_string()),
        ("jc", req.jc),
        ("week", req.week.to_string()),
        ("xn", req.xn.to_string()),
        ("xq", req.xq.to_string()),
        ("stuid", stu_id),
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
