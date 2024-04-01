use serde::Serialize;
use sqlx::FromRow;

#[derive(FromRow, Serialize)]
pub struct ExamNumberInfo {
    #[serde(rename = "num")]
    pub exam_num: String,
    #[serde(rename = "name")]
    pub exam_name: String,
    #[serde(rename = "date")]
    pub exam_date: String,
    pub id: u32,
}
