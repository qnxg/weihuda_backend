#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct SpiderExamArrange {
    pub items: Vec<SpiderExamArrangeItem>,
    pub rowCount: u32,
}

#[derive(Deserialize, Debug)]
pub struct SpiderExamArrangeItem {
    pub kcbh: String,
    pub kc_name: String,
    pub kcmc_name: String,
    pub kskssj: String,
    pub ksjssj: String,
    pub zwh: Option<String>,
}

/// 考试安排，机考安排复用此结构体
#[derive(Serialize, Debug)]
pub struct ExamArrangeRes {
    pub number: String,
    pub name: String,
    pub classroom: String,
    pub startTime: String,
    pub endTime: String,
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
