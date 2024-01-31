use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct CourseInfo {
    pub classname: String,
    pub location: Option<String>,
    pub teachers: Option<String>,
    pub week: String,
    pub day: String,
    pub section: String,
    #[serde(rename = "classID")]
    pub id: u32,
}

// #[derive(FromRow, Serialize, Debug)]
// pub struct Course {
//     classname: String,
//     location: String,
//     teachers: String,
//     week: String,
//     day: String,
//     section: String,
//     xn: u32,
//     xq: u32,
//     mini_bind_id: u32,
// }
