use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct CustomizeCourseInfo {
    pub classname: String,
    pub location: Option<String>,
    pub teachers: Option<String>,
    pub week: String,
    pub day: String,
    pub section: String,
    #[serde(rename = "classID")]
    pub id: u32,
}

// 一个 CourseInfo 相当于课表上一个格子的信息
// 除了 extra 字段外，其他的 Option 字段都是由于支持自定义课程
#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CourseInfo {
    pub course_name: String,       // 课程名称
    pub course_id: Option<String>, // 课程代码
    #[serde(rename = "type")]
    pub _type: String, // 课程类型
    pub class_name: Option<String>, // 上课班级
    pub place: Option<String>,     // 上课地点。有时候 hdjw 也不提供上课地点
    pub area: Option<String>,      // 上课校区
    pub teacher: Option<String>,   // 授课教师
    pub weeks: Vec<u8>,            // 上课周次
    pub day: u8,                   // 周几
    pub time: u8,                  // 上课的节次
    pub credit: Option<f32>,       // 学分
    pub extra: Option<String>,     // 额外备注信息
    pub customize_id: i32,         // 自定义课程id，如果不是自定义课程则为 -1
}
