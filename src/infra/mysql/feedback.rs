use super::get_db_pool;
use crate::result::AppResult;
use serde::Serialize;

#[derive(Serialize, Debug)]
#[expect(non_snake_case)]
pub struct FeedbackInfo {
    // pub id: u32,
    pub contact: Option<String>,
    // pub createTime: String,
    pub desc: String,
    pub imgUrl: Option<String>,
    pub stuId: Option<String>,
    #[serde(rename = "type")]
    pub _type: String,
    pub status: Option<i8>,
    pub comment: Option<String>,
}
pub async fn get_feedback_list(
    stu_id: &str,
    page_size: u32,
    page: u32,
) -> AppResult<Vec<FeedbackInfo>> {
    let res: Vec<FeedbackInfo> = sqlx::query_as!(
        FeedbackInfo,
        r#"
        SELECT stuId, contact, type AS "_type", status, imgUrl, comment, `desc` FROM feedbacks WHERE stuId LIKE ? ORDER BY id DESC LIMIT ? OFFSET ?
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
    typ: &str,
    desc: &str,
    contact: Option<&String>,
    img_url: Option<&String>,
    stu_id: &str,
) -> AppResult<u64> {
    let now = chrono::Local::now();
    let res = sqlx::query!(
        r#"
        INSERT INTO feedbacks (stuId, `desc`, contact, imgUrl, type, createTime, createdAt, updatedAt) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        stu_id,
        desc,
        contact,
        img_url,
        typ,
        now,
        now,
        now,
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(res.last_insert_id())
}

pub async fn update_feedback_status(
    id: u32,
    status: i8,
) -> AppResult<()> {
    let now = chrono::Local::now();
    sqlx::query!(
        r#"
        UPDATE feedbacks SET status = ?, updatedAt = ? WHERE id = ?
        "#,
        status,
        now,
        id,
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}
