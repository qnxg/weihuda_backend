use super::Result;
use super::get_db_pool;
use crate::utils;
use chrono::NaiveDateTime;

pub async fn add_left_message(
    stu_id: &str,
    desc: &str,
    email: Option<&str>,
    is_agree: bool,
    send_time: NaiveDateTime,
    is_send: bool,
) -> Result<u64> {
    let now = utils::time::now_time();
    let res = sqlx::query!(
        r#"
        INSERT INTO 
            message_lefts (stuId, `desc`, isAgree, sendTime, isSend, email, createdAt, updatedAt)
        VALUES 
            (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        stu_id,
        desc,
        is_agree,
        send_time,
        is_send,
        email,
        now,
        now,
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(res.last_insert_id())
}
