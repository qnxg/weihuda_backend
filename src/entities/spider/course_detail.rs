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
    pub khfs_name: String,
    pub ktmc_name: String,
    pub skls_name: String,
    // pub kksjdd: String,
    pub xkrs: u32,
    pub zxf: u32,
    pub zxs: u32,
}

#[derive(Serialize, Debug)]
pub struct CourseDetailRes {
    pub classID: String,
    pub serial: String,
    pub name: String,
    pub examType: String,
    pub className: String,
    pub teacher: String,
    pub people: u32,
    pub credit: u32,
    pub school: u32,
    pub timePlace: String,
}
