#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct SpiderCourseDetail {
    pub rowCount: u32,
    pub items: Vec<SpiderCourseDetailItem>,
}

#[derive(Deserialize, Debug)]
pub struct SpiderCourseDetailItem {
    pub kcbh: String,
    pub kclb_name: String,
    pub kcmc_name: String,
    pub khfs_name: Option<String>, // 可能考核方式是未知的
    pub ktmc_name: String,
    pub skls_name: String,
    pub kkdw_name: String,
    // pub kksjdd: String,
    pub xkrs: u32,
    pub zxf: u32,
    pub zxs: u32,
    pub xq_name: String,
}

#[derive(Serialize, Debug)]
pub struct CourseDetailRes {
    pub classID: String,   // 课程编号
    pub serial: String,    // 课程类型
    pub name: String,      // 课程名称
    pub examType: String,  // 考核方式
    pub className: String, // 课堂名称
    pub teacher: String,   // 老师名称
    pub people: u32,       // 人数
    pub credit: u32,       // 学分
    pub school: u32,       // 学时
    pub place: String,     // 校区
    pub academy: String,   // 开课学院
}
