use salvo::prelude::Extractible;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Extractible, Debug)]
pub struct GradeReq {
    pub stuid: String,
    pub xn: u16,
    pub xq: u8,
}

#[derive(Deserialize, Extractible, Debug)]
pub struct EmptyRoomReq {
    pub stuid: String, // 旧爬虫不需要学号信息，但是新爬虫决定让各自账号去各自请求空教室信息
    pub build_id: String,
    pub day: u8,
    pub jc: String, // 节次信息
    pub week: String,
    pub xn: u16,
    pub xq: u8,
}

#[derive(Deserialize, Debug, Serialize)]
#[expect(dead_code)]
pub struct GradeRank {
    pub arithmetic_rank: String, // 算术平均成绩排名
    pub arithmetic_score: String, // 算术平均成绩
    pub weighted_rank: String,   // 加权平均成绩排名
    pub weighted_score: String,  // 加权平均成绩
    pub gpa_rank: String,        // GPA排名
    pub gpa: String,             // GPA
}

// 注释见前端
#[derive(Debug, Deserialize, Extractible)]
pub struct HdjwGradeRankReq {
    pub stuid: String,
    pub year: Option<u16>,
    pub term: Option<u8>,
    pub course: u8,
    pub rank: u8,
}
