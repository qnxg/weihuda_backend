#![allow(non_snake_case)]
use super::serialize_f64;
use serde::{Deserialize, Serialize};

//=============成绩
#[derive(Deserialize, Debug)]
pub struct SpiderGrade {
    pub rowCount: u32,
    pub items: Vec<SpiderGradeInfo>,
}

#[derive(Deserialize, Debug)]
pub struct SpiderGradeInfo {
    pub kcbh: String,
    pub kclbname: String,
    pub kcname: String,
    pub kcxzname: String,
    pub kkdwname: String,
    pub ksxzname: String,
    pub xf: f64,
    pub zcj: u32,
    pub zxs: u32,
}

#[derive(Serialize, Debug)]
pub struct GradeRes {
    pub number: String,
    pub serial: String,
    pub name: String,
    pub college: String,
    pub examType: String,
    #[serde(serialize_with = "serialize_f64")]
    pub credit: f64,
    pub grade: u32,
}

//=============成绩排名
#[derive(Deserialize, Debug)]
pub struct SpiderGradeRank {
    pub report: Vec<SpiderGradeRankReportInfo>,
    pub semesters: Vec<SpiderGradeRankSemestersInfo>,
}

#[derive(Deserialize, Debug)]
pub struct SpiderGradeRankReportInfo {
    pub ARITHMETIC_AVG: f64,
    pub ARITHMETIC_AVG_RANK: u32,
    pub CORE_ARITHMETIC_AVG: f64,
    pub CORE_ARITHMETIC_AVG_RANK: u32,
    pub CORE_WEIGHTED_AVG: f64,
    pub CORE_WEIGHTED_AVG_RANK: u32,
    pub GPA: f64,
    pub GPA_RANK: u32,
    pub WEIGHTED_AVG: f64,
    pub WEIGHTED_AVG_RANK: u32,
}

// 为了解决返回类型不固定的问题，使用serde的untagged属性
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum F64OrString {
    F64(f64),
    String(String),
}

// 为了解决返回类型不固定的问题，使用serde的untagged属性
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum U32OrString {
    U32(u32),
    String(String),
}

#[derive(Deserialize, Debug)]
pub struct SpiderGradeRankSemestersInfo {
    pub CORE_WEIGHTED_AVG: F64OrString,
    pub CORE_WEIGHTED_AVG_RANK: U32OrString,
    pub GPA: f64,
    pub GPA_RANK: u32,
    pub WEIGHTED_AVG: f64,
    pub WEIGHTED_AVG_RANK: u32,
    pub XN: String,
    pub XQ: Option<String>, // 后续逻辑中要处理XQ为None的情况，即json返回值没有XQ字段
}

#[derive(Serialize, Debug)]
pub struct GradeRankRes {
    pub total: GradeRankResTotal,
    pub semesters: Vec<GradeRankResSemesters>,
}

#[derive(Serialize, Debug)]
pub struct GradeRankResTotal {
    pub arithmeticAvg: f64,
    pub arithmeticAvgRank: u32,
    pub coreArithmeticAvg: f64,
    pub coreArithmeticAvgRank: u32,
    pub coreWeightAvg: f64,
    pub coreWeightAvgRank: u32,
    pub GPA: f64,
    pub GPARank: u32,
    pub weightAvg: f64,
    pub weightAvgRank: u32,
}

#[derive(Serialize, Debug)]
pub struct GradeRankResSemesters {
    pub year: String,
    pub items: Vec<GradeRankResSemestersItem>,
}

#[derive(Serialize, Debug)]
pub struct GradeRankResSemestersItem {
    pub coreWeightAvg: String,
    pub coreWeightAvgRank: String,
    pub GPA: f64,
    pub GPARank: u32,
    pub weightAvg: f64,
    pub weightAvgRank: u32,
    pub name: String,
}

//=============成绩趋势
/// 需要包裹在一个Vec中
#[derive(Deserialize, Debug)]
pub struct SpiderGradeChart {
    pub CORE_WEIGHTED_AVG: F64OrString,
    pub CORE_WEIGHTED_AVG_RANK: U32OrString,
    pub GPA: f64,
    pub GPA_RANK: u32,
    pub WEIGHTED_AVG: f64,
    pub WEIGHTED_AVG_RANK: u32,
    pub XN: String,
    pub XQ: String,
}

#[derive(Serialize, Debug, Default)]
pub struct GradeChartRes {
    pub semester: Vec<String>,
    pub GPA: Vec<f64>,
    pub GPARank: Vec<u32>,
    pub weightAvg: Vec<f64>,
    pub weightAvgRank: Vec<u32>,
    pub CoreWeightAvg: Vec<f64>,
    pub CoreWeightAvgRank: Vec<u32>,
}
