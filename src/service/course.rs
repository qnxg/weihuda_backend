use crate::{
    error::{AppError, AppResult, ThrowInternalErrorResult},
    infra::{self},
    service::{
        self,
        user_info::is_graduate,
        user_state::{Hdjw, Yjsxt, with_token},
    },
    utils,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use infra::mysql::course::CustomizeCourseInfo;
pub use infra::mysql::course::add_course as add_customize_course;
pub use infra::mysql::course::delete_course as delete_customize_course;
use infra::mysql::course::get_course_list as get_customize_course;
pub use infra::mysql::course::get_custom_course_details_by_id;
pub use infra::mysql::course::update_course as update_customize_course;
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
        let week = week.trim().parse::<u8>().internal_err()?;
        weeks.push(week);
    }
    let day = item.day.parse::<u8>().internal_err()?;
    for time in &times {
        let time = time.trim().parse::<u8>().internal_err()?;
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

/// 将爬虫返回的课程信息解析，放入课表中
fn push_hdjw_course(
    classtable: &mut Vec<CourseInfo>,
    item: hnu_query::hdjw::class_table::Course,
) -> AppResult<()> {
    // record 中的一个 key 就对应于前端课表上的一个格子
    // 根据星期几和节次可以定位到前端课表上的一个格子
    // 一个格子里只显示一个地点。一个课程可能出现上课地点变动的情况，此时需要多个格子来区分
    // 所以 key 的格式为 (day, time, place)
    // 一个格子里可以显示多个周次，所以周次信息为 value
    let mut record = HashMap::new();
    for schedule_item in item.schedule {
        for time in schedule_item.time {
            record
                .entry((
                    schedule_item.day,
                    time,
                    schedule_item.place.clone(),
                ))
                .or_insert(Vec::new())
                .push(schedule_item.week);
        }
    }
    for ((day, time, place), weeks) in record {
        let tmp = CourseInfo {
            course_name: item.course_name.clone(),
            course_id: Some(item.course_id.clone()),
            _type: item.course_type.clone(),
            class_name: Some(item.class_name.clone()),
            place: match place.as_str() {
                "无" => None,
                _ => Some(place),
            },
            area: Some(item.area.clone()),
            teacher: item.teacher.clone(),
            weeks: weeks.into_iter().collect(),
            credit: Some(item.credit),
            extra: item.extra.clone(),
            customize_id: -1,
            day,
            time,
            people: item.people,
        };
        classtable.push(tmp);
    }
    Ok(())
}

/// 将爬虫返回的研究生课程信息解析，放入课表中
fn push_yjsxt_course(
    classtable: &mut Vec<CourseInfo>,
    item: hnu_query::yjsxt::class_table::Course,
) -> AppResult<()> {
    // yjsxt 的 schedule 是 Option<Vec<CourseSchedule>>
    // 如果是无节次课程则为 None，直接跳过
    let Some(schedule) = item.schedule else {
        return Ok(());
    };
    // 将天数，节次，地点相同的课程的周次合并到一起
    let mut record = HashMap::new();
    for schedule_item in schedule {
        for time in schedule_item.time {
            record
                .entry((
                    schedule_item.day,
                    time,
                    schedule_item.place.clone(),
                ))
                .or_insert(Vec::new())
                .push(schedule_item.week);
        }
    }
    for ((day, time, place), weeks) in record {
        let item = CourseInfo {
            course_name: item.course_name.clone(),
            course_id: Some(item.course_id.clone()),
            // 研究生课表不显示课程类型，这里按理说应该给 None，但是现在前后端
            // 接口对接有点混乱，所以暂时给一个空字符串
            _type: "".to_string(),
            class_name: Some(item.class_name.clone()),
            place: Some(place),
            area: None,
            teacher: item.teacher.clone(),
            weeks: weeks.into_iter().collect(),
            credit: None,
            extra: None,
            customize_id: -1,
            day,
            time,
            people: 0,
        };
        classtable.push(item);
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
#[tracing::instrument(
    fields(
        otel.kind = "internal", 
        event_type = "service", 
        is_graduate = tracing::field::Empty,
    ),
    err
)]
pub async fn get_classtable(
    stu_id: &str,
    xn: u16,
    xq: u8,
) -> AppResult<Vec<CourseInfo>> {
    let mut classtable = Vec::new();
    let customize_course =
        get_customize_course(stu_id, xn, xq).await?;
    for item in customize_course {
        push_customize_course(&mut classtable, item)?;
    }
    let is_graduate = is_graduate(stu_id).await?;
    utils::record!(is_graduate = is_graduate);
    if is_graduate {
        let semester =
            with_token(Yjsxt::new(stu_id), |token| async move {
                hnu_query::yjsxt::get_semester(&token).await
            })
            .await?;
        let semester_id = semester
            .into_iter()
            .find_map(|v| {
                if v.xn == xn && v.xq == xq {
                    Some(v.id)
                } else {
                    None
                }
            })
            .ok_or_else(|| AppError::customized("学期不存在"))?;
        let yjsxt_course = with_token(Yjsxt::new(stu_id), |token| {
            let semester_id_value = &semester_id;
            async move {
                hnu_query::yjsxt::class_table::get_class_table(
                    &token,
                    semester_id_value,
                )
                .await
            }
        })
        .await?;
        for item in yjsxt_course {
            push_yjsxt_course(&mut classtable, item)?;
        }
    } else {
        let hdjw_course =
            with_token(Hdjw::new(stu_id), |token| async move {
                hnu_query::hdjw::get_class_table(&token, xn, xq).await
            })
            .await?;
        for item in hdjw_course {
            push_hdjw_course(&mut classtable, item)?;
        }
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
    xn: u16,
    xq: u8,
) -> AppResult<Vec<ExtraCourseInfo>> {
    // 研究生系统不支持 extra course，返回空
    if is_graduate(stu_id).await? {
        return Ok(Vec::new());
    }
    let spider_res =
        with_token(Hdjw::new(stu_id), |token| async move {
            hnu_query::hdjw::get_class_table_extra(&token, xn, xq)
                .await
        })
        .await?;
    let mut res = Vec::new();
    for item in spider_res {
        res.push(ExtraCourseInfo {
            class_name: item.class_name,
            course_id: item.course_id,
            course_name: item.course_name,
            _type: item.course_type,
            area: item.area,
            teacher: item.teacher,
            credit: item.credit,
            people: item.people,
            extra: item.extra,
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
    pub xn: u16, // 学年
    pub xq: u8,  // 学期
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
    use crate::test::{TEST_STU_ID, TEST_XN, TEST_XQ};

    #[tokio::test]
    async fn test_get_classtable() {
        let classtable =
            get_classtable(&TEST_STU_ID, TEST_XN, TEST_XQ)
                .await
                .unwrap();
        println!("{:#?}", classtable);
    }
}
