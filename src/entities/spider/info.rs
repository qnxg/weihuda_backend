#![allow(non_snake_case)]
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct SemesterInfoRes {
    pub startDate: String,
    pub term: u32,
    pub year: u32,
    pub vacation: String,
    pub next: String,
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
