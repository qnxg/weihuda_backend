use std::collections::{HashMap, HashSet};

use anyhow::anyhow;
use regex::Regex;
use serde::{Deserialize, Serialize};
use spider_2024::dtos::hdjw::CourseInfoRes;

use crate::{
    infra::{self},
    result::AppResult,
    service,
};

pub use infra::mysql::course::CustomizeCourseInfo;
pub use infra::mysql::course::add_course as add_customize_course;
pub use infra::mysql::course::delete_course as delete_customize_course;
use infra::mysql::course::get_course_list as get_customize_course;
pub use infra::mysql::course::get_custom_course_details_by_id;

const FLEX_TIME_CONFIG_KEY: &str = "flexTime";

// 除了 extra 字段外，其他的 Option 字段都是由于支持自定义课程
#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CourseInfo {
    pub course_name: String,       // 课程名称
    pub course_id: Option<String>, // 课程代码
    #[serde(rename = "type")]
    pub _type: String, // 课程类型
    pub class_name: Option<String>, // 上课班级
    pub place: Option<String>, // 上课地点。有时候 hdjw 也不提供上课地点
    pub area: Option<String>,  // 上课校区
    pub teacher: Option<String>, // 授课教师
    pub weeks: Vec<u8>,        // 上课周次
    pub day: u8,               // 周几
    pub time: u8,              // 上课的节次
    pub credit: Option<f32>,   // 学分
    pub extra: Option<String>, // 额外备注信息
    pub customize_id: i32, // 自定义课程id，如果不是自定义课程则为 -1
    pub people: u16,       // 上课人数
}

#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExtraCourseInfo {
    pub class_name: String,  // 上课班级
    pub course_id: String,   // 课程代码
    pub course_name: String, // 课程名称
    #[serde(rename = "type")]
    pub _type: String, // 课程类型
    pub area: String,        // 上课校区
    pub teacher: String,     // 授课教师
    pub credit: f32,         // 学分
    pub people: u16,         // 上课人数
    pub extra: Option<String>, // 额外备注信息
}

// 将 CustomizeCourseInfo 解析，并放入课表
fn push_customize_course(
    classtable: &mut Vec<CourseInfo>,
    item: CustomizeCourseInfo,
) -> AppResult<()> {
    let times: Vec<&str> = item.section.split(',').collect();
    let weeks_from_str: Vec<&str> = item.week.split(',').collect();
    let mut weeks = Vec::new();
    for week in weeks_from_str {
        let week = week
            .trim()
            .parse::<u8>()
            .map_err(|e| anyhow!("课程周次解析失败 {}", e))?;
        weeks.push(week);
    }
    let day = item
        .day
        .parse::<u8>()
        .map_err(|e| anyhow!("课程星期解析失败 {}", e))?;
    for time in &times {
        let time = time
            .trim()
            .parse::<u8>()
            .map_err(|e| anyhow!("课程节次解析失败 {}", e))?;
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
            people: 0,
        };
        classtable.push(tmp);
    }
    Ok(())
}

/// 将 hdjw::SpiderCourseInfo 解析，并放入课表
#[expect(clippy::too_many_lines, reason = "REFACTOR ME")]
fn push_hdjw_course(
    classtable: &mut Vec<CourseInfo>,
    item: CourseInfoRes,
) -> AppResult<()> {
    let re = Regex::new(r"周(.)第(.*)节.*\{第(.*)周\}")
        .expect("创建正则表达式失败");
    // 这里主要处理上课时间，每个时间同时对应一个地点。
    // 考虑以 周几+节次+地点作为 key，周数作为 value，用 set 存储。这样做是为了合并一些可以合并的时间（hdjw 可能分开写），并且区分不同地点的课程
    let places = item.skddmc.split(';').collect::<Vec<_>>();
    let mut record = HashMap::new();
    let detail_times = item.sktime.split(';');
    for (i, time) in detail_times.into_iter().enumerate() {
        let caps =
            re.captures(time).ok_or(anyhow!("解析课程时间失败"))?;
        let day = caps
            .get(1)
            .ok_or(anyhow!("解析课程时间失败"))?
            .as_str()
            .chars()
            .next()
            .ok_or(anyhow!("解析课程时间失败"))?;
        let day = match day {
            '一' => 1,
            '二' => 2,
            '三' => 3,
            '四' => 4,
            '五' => 5,
            '六' => 6,
            '日' | '七' => 7,
            _ => {
                return Err(anyhow!("未知的星期字符：{}", day).into());
            }
        };
        let times = caps
            .get(2)
            .ok_or(anyhow!("解析课程时间失败"))?
            .as_str()
            .split('、')
            .collect::<Vec<_>>();
        let weeks =
            caps.get(3).ok_or(anyhow!("解析课程时间失败"))?.as_str();
        let place =
            places.get(i).ok_or(anyhow!("解析课程地点失败"))?;
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
                        .map_err(|e| {
                            anyhow!("解析课程时间失败 {}", e)
                        })?
                };
                for time in time_l..=time_r {
                    let key = (day, time, *place);
                    let set = record
                        .entry(key)
                        .or_insert_with(HashSet::new);
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
            people: item.xkrs,
        };
        classtable.push(tmp);
    }
    Ok(())
}

/// 注意，调用后会使得某些元素的周次变成空的，因此需要调用该函数后手动清除这些元素，由于可能的性能原因，该函数不负责清除工作
fn apply_flex_time(
    classtable: &mut Vec<CourseInfo>,
    flex: FlexTime,
) -> AppResult<()> {
    // 先把 to 那天的课程全部毙掉
    for item in classtable.iter_mut() {
        if item.day != flex.to.day {
            continue;
        }
        item.weeks.retain(|&x| x != flex.to.week);
    }
    // 调休存在一个场景，就是简单的某天课程停上，此时 from 为 None
    let Some(from) = flex.from else {
        return Ok(());
    };
    // 然后找 from 那天的课，全部加入到 to 那天的课程中
    // 加入到 to 的时候直接创建一个新的课程，和原来的课程做一个区分，这样前端显示起来会好一点
    let mut new_items = Vec::new();
    for item in classtable.iter_mut() {
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
    for item in classtable.iter_mut() {
        if item.day != from.day {
            continue;
        }
        item.weeks.retain(|&x| x != from.week);
    }
    classtable.extend(new_items);
    Ok(())
}

/// 包含了用户自定义的课程，同时根据调休将课程进行了调整
/// 这个函数会生成一个 `Vec<CourseInfo>` 表示课表。`Vec<CourseInfo>` 内的每个元素表示前端课表页面上的一个格子
pub async fn get_classtable(
    stu_id: &str,
    xn: u32,
    xq: u32,
) -> AppResult<Vec<CourseInfo>> {
    let mut classtable = Vec::new();
    let customize_course =
        get_customize_course(stu_id, xn, xq).await?;
    for item in customize_course {
        push_customize_course(&mut classtable, item)?;
    }
    let hdjw_course =
        infra::spider::hdjw::get_course(xn, xq, stu_id).await?;
    for item in hdjw_course {
        push_hdjw_course(&mut classtable, item)?;
    }
    // 处理调休
    let mut flex_time = get_flex_time_list().await?;
    // 只保留当前学期的调休
    flex_time.retain(|x| x.time.xn == xn && x.time.xq == xq);
    for item in flex_time {
        apply_flex_time(&mut classtable, item)?;
    }
    // 由于调休那里的操作，会使得某些课程的周次变成空的，所以需要清理一下
    classtable.retain(|item| !item.weeks.is_empty());
    // 再给 weeks 排个序
    for item in classtable.iter_mut() {
        item.weeks.sort_unstable();
    }
    Ok(classtable)
}

pub async fn get_extra_course(
    stu_id: &str,
    xn: u32,
    xq: u32,
) -> AppResult<Vec<ExtraCourseInfo>> {
    let spider_res =
        infra::spider::hdjw::get_class_table_extra(stu_id, xn, xq)
            .await?;
    let mut res = Vec::new();
    for item in spider_res {
        res.push(ExtraCourseInfo {
            class_name: item.kt_mc,
            course_id: item.kch,
            course_name: item.kc_mc,
            _type: item.kcxz,
            area: item.skxqmc,
            teacher: item.jg0101mc,
            credit: item.xf,
            people: item.xkrs,
            extra: item.fzmc,
        });
    }
    Ok(res)
}

/// 调休的结构体
/// 将会将 from 的课程全部转移到 to 上去，且 to 的课程全部毙掉
#[derive(Debug, Serialize, Deserialize)]
pub struct FlexTime {
    // 如果这里是 None，表示仅 to 那天的课停上，不会有课程转移
    pub from: Option<FlexDay>,
    pub to: FlexDay,
    pub desc: String, // 描述，将会返回给前端用作展示
    pub time: XnXq,   // 学年学期
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FlexDay {
    pub week: u8, // 第几周
    pub day: u8,  // 星期几
}

#[derive(Debug, Serialize, Deserialize)]
pub struct XnXq {
    pub xn: u32, // 学年
    pub xq: u32, // 学期
}
pub async fn get_flex_time_list() -> AppResult<Vec<FlexTime>> {
    let config = service::config::get_config(FLEX_TIME_CONFIG_KEY)
        .await?
        .expect("获取调休信息失败");
    let flex_time: Vec<FlexTime> =
        serde_json::from_str(&config.value)
            .expect("解析调休信息失败");
    Ok(flex_time)
}

#[cfg(test)]
mod tests {
    use super::*;

    const STUID: &str = "";

    #[tokio::test]
    async fn test_get_classtable() {
        let classtable =
            get_classtable(STUID, 2025, 1).await.unwrap();
        println!("{:#?}", classtable);
    }
}
