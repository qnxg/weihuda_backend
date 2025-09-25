#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};

//=============成绩
#[derive(Deserialize, Debug)]
pub struct SpiderGradeInfo {
    // pub cj0708id: String, // 未知字段
    // pub xnxqid: String,   // 学年学期信息（暂时不用）
    pub kch: String,   // 课程代码
    pub kc_mc: String, // 课程名称
    // pub ksdw: String,     // 开课学院（暂时不用）
    // pub xqmc: String, // 似乎和 xnxqid 重复
    pub xf: f32, // 学分
    // pub zxs: u32,      // 总学时（暂时不用）
    // pub ksfs: String,  // 考试方式（暂时不用）
    pub kcsx: String, // 课程属性（必修/选修等）
    // pub xqstr: String, // 似乎又和 xnxqid 重复
    pub zcj: u8, // 总成绩
    // pub zcjstr: String,   // 总成绩字符串形式（暂时不用）
    // pub kz: u8,           // 未知字段
    pub kcxzmc: String, // 课程性质（通识必修/专业核心等）
    // pub xs0101id: String, // 未知字段
    // pub jx0404id: Option<String>, // 似乎和 kch 重复，部分成绩没有该字段
    pub jd: f32, // 绩点
    // pub ksxz: String,         // 考试性质（暂时不用）
    pub falb: String,         // 主修还是辅修
    pub cjbs: Option<String>, // 成绩标识（缓考/重修等，注意这个标识是挂在为 0 分的那个成绩 item 上）
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GradeInfo {
    pub course_id: String,    // 课程代码
    pub course_name: String,  // 课程名称
    pub credit: f32,          // 学分
    pub course_type1: String, // 课程性质1（必修还是选修）
    pub course_type2: String, // 课程性质2（通识必修/专业核心等）
    pub gpa: f32,             // 绩点
    pub score: u8,            // 成绩
    pub tags: Vec<String>, // 其他标签，如缓考还是什么（参考 SpiderGradeInfo 的 cjbs 说明），或者辅修等
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

#[derive(Serialize, Debug, Deserialize)]
pub struct HdjwGradeRank {
    pub score: String,
    pub rank: String,
}

#[derive(Serialize, Debug)]
pub struct CaGradeRank {
    pub all_gpa: String,
    pub all_gpa_rank: String,
    pub all_weighted: String,
    pub all_weighted_rank: String,
    pub all_arithmetic: String,
    pub all_arithmetic_rank: String,
    pub must_gpa: String,
    pub must_weighted: String,
    pub must_arithmetic: String,
    pub core_gpa_rank: String,
    pub core_weighted_rank: String,
    pub core_arithmetic_rank: String,
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
