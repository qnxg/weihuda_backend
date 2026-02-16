use super::get_db_pool;
use crate::{result::AppResult, utils};
use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackInfo {
    pub id: u32,
    pub contact: Option<String>,
    pub desc: String,
    pub img_url: Option<String>,
    pub stu_id: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub status: u32,
}
pub async fn get_feedback_list(
    stu_id: &str,
    page_size: u32,
    page: u32,
) -> AppResult<Vec<FeedbackInfo>> {
    let res: Vec<FeedbackInfo> = sqlx::query_as!(
        FeedbackInfo,
        r#"
        SELECT 
        id, stuId as stu_id, contact, status, imgUrl as img_url, createdAt as created_at, updatedAt as updated_at, `desc` 
        FROM feedbacks WHERE stuId = ? 
        ORDER BY id DESC 
        LIMIT ? OFFSET ?
        "#,
        stu_id,
        page_size,
        (page - 1) * page_size
    )
    .fetch_all(get_db_pool().await)
    .await?;
    Ok(res)
}

pub async fn add_feedback(
    desc: &str,
    contact: Option<&String>,
    img_url: Option<&String>,
    stu_id: Option<&str>,
) -> AppResult<u64> {
    let now = utils::time::now_time();
    let res = sqlx::query!(
        r#"
        INSERT INTO feedbacks 
        (stuId, `desc`, contact, imgUrl, status, createdAt, updatedAt) 
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
        stu_id,
        desc,
        contact,
        img_url,
        0,
        now,
        now,
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(res.last_insert_id())
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackMsg {
    pub id: u32,
    pub typ: String,
    pub msg: Option<String>,
    pub stu_id: String,
    pub created_at: NaiveDateTime,
}
pub async fn get_feedback_msg(
    stu_id: &str,
    feedback_id: u32,
) -> AppResult<Vec<FeedbackMsg>> {
    let res: Vec<FeedbackMsg> = sqlx::query_as!(
        FeedbackMsg,
        r#"
        SELECT 
        id, typ, msg, stuId as stu_id, createdAt as created_at
        FROM feedback_msg 
        WHERE feedbackId = ? AND stuId = ? AND deletedAt IS NULL
        ORDER BY id DESC
        "#,
        feedback_id,
        stu_id,
    )
    .fetch_all(get_db_pool().await)
    .await?;
    Ok(res)
}
