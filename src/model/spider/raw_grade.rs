#![allow(non_snake_case)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::serialize_f64;

#[derive(Deserialize, Debug)]
pub struct SpiderRawGrade {
    pub cjxmInfo: Vec<SpiderRawGradeIndex>,
    pub cjxmcj: SpiderRawGradeInfo,
}

#[derive(Deserialize, Debug)]
pub struct SpiderRawGradeIndex {
    pub cjgl001id: String,
    pub jczy013id: String,
    pub shztkfcode: String,
    pub xmbh: String,
    pub xmmc: String,
}

#[derive(Deserialize, Debug)]
pub struct SpiderRawGradeInfo {
    pub items: Vec<SpiderRawGradeInfoItem>,
    pub rowCount: u32,
}

#[derive(Deserialize, Debug)]
pub struct SpiderRawGradeInfoItem {
    pub cjgl009id: String,
    pub cjxm1: Option<f64>,
    pub cjxm2: Option<f64>,
    pub cjxm3: Option<f64>,
    pub cjxm4: Option<f64>,
    pub cjxm5: Option<f64>,
    pub cjxm6: Option<f64>,
    pub cjxm7: Option<f64>,
    pub jczy010id: String,
    pub jczy013id: String,
    pub jczy013name: String,
    pub kc_name: String,
    pub kcbh: String,
    pub kclb_name: String,
    pub kclbcode: String,
    pub kcxz_name: String,
    pub kcxzcode: String,
    pub khfs_name: String,
    pub khfscode: String,
    pub kkdw_name: String,
    pub kkgl004id: String,
    pub ksxz_name: String,
    pub ksxzcode: String,
    pub lrrid: String,
    pub lrrname: String,
    pub rownum_: u32,
    pub xsgl001id: String,
}

// 构建HashMap方便后续程序逻辑处理
pub fn raw_grade_item_struct_to_map(item: &SpiderRawGradeInfoItem) -> HashMap<String, f64> {
    let mut map = HashMap::new();
    if let Some(value) = item.cjxm1 {
        map.insert("1".to_string(), value);
    }
    if let Some(value) = item.cjxm2 {
        map.insert("2".to_string(), value);
    }
    if let Some(value) = item.cjxm3 {
        map.insert("3".to_string(), value);
    }
    if let Some(value) = item.cjxm4 {
        map.insert("4".to_string(), value);
    }
    if let Some(value) = item.cjxm5 {
        map.insert("5".to_string(), value);
    }
    if let Some(value) = item.cjxm6 {
        map.insert("6".to_string(), value);
    }
    if let Some(value) = item.cjxm7 {
        map.insert("7".to_string(), value);
    }
    map
}

//=============Return
/// 需要被包裹在一个Vec中返回
#[derive(Serialize, Debug)]
pub struct RawGradeRes {
    pub name: String,
    pub item: Vec<RawGradeResItem>,
}

#[derive(Serialize, Debug)]
pub struct RawGradeResItem {
    pub name: String,
    #[serde(serialize_with = "serialize_f64")]
    pub grade: f64,
}
