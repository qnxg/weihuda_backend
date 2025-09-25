use crate::app_error::AppError;
use crate::dtos::spider::hdjw::HdjwGradeRankReq;
use crate::entities::back::course::CourseInfo;
use crate::entities::back::flex_time::FlexTime;
use crate::entities::spider::grade::{CaGradeRank, GradeInfo, HdjwGradeRank, SpiderGradeInfo};
use crate::utils::semester::get_class_start_date_by_xnxq;
use crate::{
    app_result::{AppResult, AppState},
    dtos::spider::hdjw::{
        GetClassStartDateReq, GetClassTableReq, GetEmptyRoomReq, GetExamArrangeReq, GetGradeReq,
    },
    entities::{
        back::course::CustomizeCourseInfo,
        spider::{
            class_table::SpiderCourseInfo,
            empty_room::EmptyRoomRes,
            exam::{ExamArrangeRes, SpiderComputerExamArrange, SpiderExamArrange},
            grade::{F64OrString, GradeChartRes, SpiderGradeChart, U32OrString},
        },
    },
    extractors::Query,
    utils::{
        jwt::{parse, parse_stu_id},
        request::spider_data,
    },
};
use anyhow::{anyhow, Result};
use axum::extract::{Extension, State};
use regex::{Regex, RegexBuilder};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::vec;

// 课表解析的代码还不太稳定，并且逻辑比较长，所以考虑单独提出来
async fn parse_class_table(data: Vec<SpiderCourseInfo>) -> Result<Vec<CourseInfo>> {
    let mut res = Vec::new();
    let re = Regex::new(r"周(.)第(.*)节.*\{第(.*)周\}").unwrap();
    for item in data {
        // 这里主要处理上课时间，每个时间同时对应一个地点。
        // 考虑以 周几+节次+地点作为 key，周数作为 value，用 set 存储。这样做是为了合并一些可以合并的时间（hdjw 可能分开写），并且区分不同地点的课程
        let places = item.skddmc.split(';').collect::<Vec<_>>();
        let mut record = HashMap::new();
        let detail_times = item.sktime.split(';');
        for (i, time) in detail_times.into_iter().enumerate() {
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
                _ => return Err(anyhow!("未知的星期字符：{}", day)),
            };
            let times = caps
                .get(2)
                .ok_or(anyhow!("解析课程时间失败"))?
                .as_str()
                .split('、')
                .collect::<Vec<_>>();
            let weeks = caps.get(3).ok_or(anyhow!("解析课程时间失败"))?.as_str();
            let place = places.get(i).ok_or(anyhow!("解析课程地点失败"))?;
            for week_range in weeks.split(',') {
                // week 这里可能是单个数字，也可能是一个范围，用 ',' 分割
                let parts = week_range.split('-').collect::<Vec<_>>();
                let week_l = parts
                    .first()
                    .ok_or(anyhow!("解析课程周次失败"))?
                    .parse::<u8>()
                    .map_err(|e| anyhow!("解析课程周次失败 {}", e))?;
                let week_r = if parts.len() == 1 {
                    week_l
                } else {
                    parts
                        .get(1)
                        .ok_or(anyhow!("解析课程周次失败"))?
                        .parse::<u8>()
                        .map_err(|e| anyhow!("解析课程周次失败 {}", e))?
                };
                for time in times.iter() {
                    // time 可能是单个数字，也可能是一个范围，用 '、' 分割
                    let parts = time.split('-').collect::<Vec<_>>();
                    let time_l = parts
                        .first()
                        .ok_or(anyhow!("解析课程时间失败"))?
                        .parse::<u8>()
                        .map_err(|e| anyhow!("解析课程时间失败 {}", e))?;
                    let time_r = if parts.len() == 1 {
                        time_l
                    } else {
                        parts
                            .get(1)
                            .ok_or(anyhow!("解析课程时间失败"))?
                            .parse::<u8>()
                            .map_err(|e| anyhow!("解析课程时间失败 {}", e))?
                    };
                    for time in time_l..=time_r {
                        let key = (day, time, *place);
                        let set = record.entry(key).or_insert_with(HashSet::new);
                        for week in week_l..=week_r {
                            set.insert(week);
                        }
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
    Ok(res)
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
    let mut res = parse_class_table(spider_res).await?;
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
    // 处理调休。目前的设计也会调休自定义课程，可能不是一个好做法？
    let flex_time =
        sqlx::query!("SELECT value FROM mini_configs WHERE `key` = ? AND enabled = 1", "flexTime")
            .fetch_one(&data.db)
            .await?
            .value;
    let mut flex_time: Vec<FlexTime> =
        serde_json::from_str(&flex_time).map_err(|_| anyhow::anyhow!("解析调休信息失败"))?;
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
        // 调休存在一个场景，就是简单的某天课程停上，此时 from 为 None
        if flex.from.is_none() {
            continue;
        }
        let from = flex.from.unwrap();
        // 然后找 from 那天的课，全部加入到 to 那天的课程中
        // 加入到 to 的时候直接创建一个新的课程，和原来的课程做一个区分，这样前端显示起来会好一点
        let mut new_items = Vec::new();
        for item in res.iter_mut() {
            if item.day != from.day || !item.weeks.contains(&from.week) {
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
            if item.day != from.day {
                continue;
            }
            item.weeks.retain(|&x| x != from.week);
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
        let mut tags = Vec::new();
        if item.falb != "主修" {
            tags.push(item.falb);
        }
        if let Some(cjbs) = item.cjbs {
            tags.push(cjbs);
        }
        let tmp = GradeInfo {
            course_id: item.kch,
            course_name: item.kc_mc,
            credit: item.xf,
            course_type1: item.kcsx,
            course_type2: item.kcxzmc,
            gpa: item.jd,
            score: item.zcj,
            tags,
        };
        res.push(tmp);
    }
    Ok(res.into())
}

pub async fn get_grade_rank_from_ca_handler(Extension(token): Extension<String>) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let params = [("stuid", stu_id)];
    let spider_res: String = spider_data("/bks/grade-from-ca", &params).await?;
    let regex = RegexBuilder::new(r"平均学分绩点排名 ([0-9/]+).*平均学分绩点 ([0-9.]+).*核心课程平均学分绩点排名 ([0-9/]+).*必修课平均学分绩点 ([0-9.]+).*课程算术平均成绩排名 ([0-9/]+).*算术平均分 ([0-9.]+).*核心课程算术平均成绩排名 ([0-9/]+).*必修课算术平均分 ([0-9.]+).*学分加权平均成绩排名 ([0-9/]+).*加权平均分 ([0-9.]+).*核心课程学分加权平均成绩排名 ([0-9/]+).*必修课加权平均分 ([0-9.]+)")
        .dot_matches_new_line(true)
        .build()
        .unwrap();
    let caps = regex.captures(&spider_res).ok_or(anyhow!("解析可信电子凭证失败"))?;
    // 12 个捕获组，caps[0] 是完整匹配，共 13 个
    if caps.len() != 13 {
        return Err(AppError::AnyHow(anyhow!("解析可信电子凭证失败a1")));
    }
    let mut res = Vec::new();
    for i in 1..=12 {
        res.push(caps.get(i).unwrap().as_str());
    }
    let res = CaGradeRank {
        all_gpa: res[1].to_string(),
        all_gpa_rank: res[0].to_string(),
        all_weighted: res[9].to_string(),
        all_weighted_rank: res[8].to_string(),
        all_arithmetic: res[5].to_string(),
        all_arithmetic_rank: res[4].to_string(),
        must_gpa: res[3].to_string(),
        must_weighted: res[11].to_string(),
        must_arithmetic: res[7].to_string(),
        core_gpa_rank: res[2].to_string(),
        core_arithmetic_rank: res[6].to_string(),
        core_weighted_rank: res[10].to_string(),
    };
    Ok(res.into())
}

pub async fn get_grade_rank_handler(
    Query(req): Query<HdjwGradeRankReq>,
    Extension(token): Extension<String>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let mut params = vec![
        ("stuid", stu_id),
        ("course", req.course.to_string()),
        ("rank", req.rank.to_string()),
    ];
    if let Some(year) = req.year {
        params.push(("year", year.to_string()));
    }
    if let Some(term) = req.term {
        params.push(("term", term.to_string()));
    }
    let spider_res: HdjwGradeRank = spider_data("/bks/rank", &params).await?;
    Ok(spider_res.into())
}

// 暂时用新的
// pub async fn get_grade_rank_handler(Extension(token): Extension<String>) -> AppResult {
//     let stu_id = parse_stu_id(&token)?;
//     let params = [("stuid", stu_id), ("type", 1.to_string())];
//     let spider_res: SpiderGradeRank = spider_data("/bks/grade/analyze", &params).await?;
//     // res的total字段
//     if spider_res.report.is_empty() {
//         return Err("汇总数据为空".into());
//     }
//     let res_total = GradeRankResTotal {
//         arithmeticAvg: spider_res.report[0].ARITHMETIC_AVG,
//         arithmeticAvgRank: spider_res.report[0].ARITHMETIC_AVG_RANK,
//         coreArithmeticAvg: spider_res.report[0].CORE_ARITHMETIC_AVG,
//         coreArithmeticAvgRank: spider_res.report[0].CORE_ARITHMETIC_AVG_RANK,
//         coreWeightAvg: spider_res.report[0].CORE_WEIGHTED_AVG,
//         coreWeightAvgRank: spider_res.report[0].CORE_WEIGHTED_AVG_RANK,
//         GPA: spider_res.report[0].GPA,
//         GPARank: spider_res.report[0].GPA_RANK,
//         weightAvg: spider_res.report[0].WEIGHTED_AVG,
//         weightAvgRank: spider_res.report[0].WEIGHTED_AVG_RANK,
//     };
//     // res的semesters字段
//     let mut res_semesters = Vec::with_capacity(spider_res.semesters.len()); // 返回结果的semesters字段
//     let mut year_map: HashMap<u32, Vec<GradeRankResSemestersItem>> = HashMap::new(); // 用年份来组织数据
//     for item in spider_res.semesters.into_iter().rev()
//     // 参照原有中间件代码，将数据反转
//     {
//         let xn = item.XN.parse::<u32>().unwrap();
//         let name = match item.XQ {
//             Some(xq) => match xq.parse::<u32>().unwrap() {
//                 1 => "秋季学期",
//                 2 => "春季学期",
//                 3 => "夏季学期",
//                 _ => "未知学期", // 不会出现，只是为了穷举所有情况，使编译通过
//             },
//             None => "全部学期",
//         };
//         let temp = GradeRankResSemestersItem {
//             coreWeightAvg: match item.CORE_WEIGHTED_AVG {
//                 F64OrString::F64(x) => x.to_string(),
//                 F64OrString::String(x) => x,
//             },
//             coreWeightAvgRank: match item.CORE_WEIGHTED_AVG_RANK {
//                 U32OrString::U32(x) => x.to_string(),
//                 U32OrString::String(x) => x,
//             },
//             GPA: item.GPA,
//             GPARank: item.GPA_RANK,
//             weightAvg: item.WEIGHTED_AVG,
//             weightAvgRank: item.WEIGHTED_AVG_RANK,
//             name: name.to_string(),
//         };
//         // 如果用年份索引year_map没有找到对应的数据，则新建一个，否则将temp插入到对应的vec中
//         match year_map.get_mut(&xn) {
//             Some(x) => x.push(temp),
//             None => {
//                 year_map.insert(xn, vec![temp]);
//             }
//         }
//     }
//     year_map.into_iter().for_each(|(k, v)| {
//         let temp = GradeRankResSemesters { items: v, year: format!("{}-{}", k, k + 1) };
//         res_semesters.push(temp);
//     });

//     res_semesters.sort_by(|a, b| b.year.cmp(&a.year)); // 按年份降序排列

//     let res = GradeRankRes { total: res_total, semesters: res_semesters };
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
    let spider_res: Value = spider_data("/freeroom/list", &params).await?;
    let data = spider_res
        .as_array()
        .ok_or(anyhow!("解析空教室数据失败"))?
        .get(4)
        .ok_or(anyhow!("解析空教室数据失败"))?
        .as_array()
        .ok_or(anyhow!("解析空教室数据失败"))?;
    let mut res = Vec::new();
    for item in data {
        let item = item.as_array().ok_or(anyhow!("解析空教室数据失败"))?;
        let is_free = item.get(1).ok_or(anyhow!("解析空教室数据失败"))?.is_null();
        if !is_free {
            continue;
        }
        let name = item
            .first()
            .ok_or(anyhow!("解析空教室数据失败"))?
            .as_str()
            .ok_or(anyhow!("解析空教室数据失败"))?;
        let capacity = item
            .get(3)
            .ok_or(anyhow!("解析空教室数据失败"))?
            .as_str()
            .ok_or(anyhow!("解析空教室数据失败"))?;
        if capacity.len() < 3 || !capacity.starts_with('(') || !capacity.ends_with(')') {
            return Err(anyhow!("解析空教室数据失败").into());
        }
        let _type = item
            .get(4)
            .ok_or(anyhow!("解析空教室数据失败"))?
            .as_str()
            .ok_or(anyhow!("解析空教室数据失败"))?;
        let mut capacity = capacity[1..capacity.len() - 1].split('/');
        let seat = capacity
            .next()
            .ok_or(anyhow!("解析空教室数据失败"))?
            .parse::<u32>()
            .map_err(|e| anyhow!("解析空教室数据失败 {}", e))?;
        let exam_seat = capacity
            .next()
            .ok_or(anyhow!("解析空教室数据失败"))?
            .parse::<u32>()
            .map_err(|e| anyhow!("解析空教室数据失败 {}", e))?;
        let temp = EmptyRoomRes {
            name: name.to_string(),
            seat,
            examSeat: exam_seat,
            _type: _type.to_string(),
        };
        res.push(temp);
    }
    Ok(res.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_class_table() {
        let params = [("xn", "2025"), ("xq", "1"), ("stuid", "")];
        let spider_res: Vec<SpiderCourseInfo> =
            spider_data("/bks/classtable", &params).await.unwrap();
        let res = parse_class_table(spider_res).await.unwrap();
        println!("{:#?}", res);
    }
}
