#![allow(non_snake_case)]
use crate::utils::default::*;
use serde::{Deserialize, Deserializer};

/// 获取课表
#[derive(Deserialize, Debug)]
pub struct GetClassTableReq {
    pub xn: u32,
    pub xq: u32,
}

/// 获取开课时间
#[derive(Deserialize, Debug)]
pub struct GetClassStartDateReq {
    pub xn: u32,
    pub xq: u32,
}

/// 获取成绩
#[derive(Deserialize, Debug)]
pub struct GetGradeReq {
    pub xn: u32,
    pub xq: u32,
}

/// 获取所有必修课的算术总成绩和加权总成绩
#[derive(Deserialize, Debug)]
pub struct GetMustGradeReq {
    pub xn: u32,
} 

// 获取成绩排名(无需请求参数)

/// 获取项目成绩
#[derive(Deserialize, Debug)]
pub struct GetRawGradeReq {
    pub xn: u32,
    pub xq: u32,
}

// 获取成绩趋势(无需请求参数)

/// 查询课程信息，暂不实现，爬虫还有bug
#[derive(Deserialize, Debug)]
pub struct GetCourseInfoReq {
    pub xn: u32,
    pub xq: u32,
    pub keyword: String,
}

/// 获取考试安排，机考安排复用此结构体
#[derive(Deserialize, Debug)]
pub struct GetExamArrangeReq {
    #[serde(default = "default_xn", deserialize_with = "xn_default")]
    pub xn: u32,
    #[serde(default = "default_xq", deserialize_with = "xq_default")]
    pub xq: u32,
}

/// 获取空教室
#[derive(Deserialize, Debug)]
pub struct GetEmptyRoomReq {
    pub buildId: String, // 楼栋id
    pub day: u32,        // 星期几
    pub jc: String,      // 节次
    pub week: u32,       // 周次
    pub xn: u32,
    pub xq: u32,
}

// 解决参数为空字符串时候的解析问题
fn xn_default<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(if s.is_empty() { default_xn() } else { s.parse().unwrap() })
}

// 解决参数为空字符串时候的解析问题
fn xq_default<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(if s.is_empty() { default_xq() } else { s.parse().unwrap() })
}
