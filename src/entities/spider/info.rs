#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct SemesterInfoRes {
    pub startDate: String,
    pub term: u32,
    pub year: u32,
    pub vacation: String,
    pub next: String,
}

#[derive(Deserialize, Debug)]
pub struct SpiderUserInfo {
    pub bj_name: Option<String>,
    pub name: String,
    pub ndzy_name: String,
    pub rxnf: String,
    pub skdw_name: String,
    pub xb: String,
    pub xz: String,
    pub xh: String,
}

#[derive(Serialize, Debug)]
pub struct UserInfoRes {
    pub class: String,
    pub name: String,
    pub major: String,
    pub enter: u32,
    pub college: String,
    pub sex: String,
    pub xz: u32,
    pub stuId: String,
}
