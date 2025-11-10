#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct SpiderExamArrangeItem {
    pub kch: String,         // 课程代码
    pub kskcmc: String,      // 课程名称
    pub ksxq: String,        // 考试校区
    pub js_mc: String,       // 考试的教室
    pub kssj: String,        // 考试时间（已经是一个时间区间了）
    pub zwh: Option<String>, // 座位号
}

/// 考试安排
#[derive(Serialize, Debug)]
pub struct ExamArrangeRes {
    pub id: String,
    pub name: String,
    pub place: String,
    pub date: String, // 考试日期，格式为 "YYYY-MM-DD"
    pub time: String, // 考试的时间段，例如：14:00~16:00
    pub seat: String,
}

#[derive(Deserialize, Debug)]
pub struct SpiderComputerExamArrange {
    pub hd_name: String,
    pub hdname1: String,
    pub jf_name: String,
    pub jfbh: String,
    pub jkrq: String,
    pub jssj: String,
    pub jwbh: String,
    pub jxl_name: String,
    pub kc_name: String,
    pub kcbh: String,
    pub kssj: String,
    pub xh: String,
    pub xs_name: String,
    pub yx_name: String,
}
