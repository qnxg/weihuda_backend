use super::get_db_pool;
use crate::error::{AppResult, ThrowInternalErrorResult};
use crate::utils;
use chrono::NaiveDateTime;

#[tracing::instrument(skip_all, fields(otel.kind = "client", event_type = "db"), err)]
pub async fn add_left_message(
    stu_id: &str,
    desc: &str,
    email: Option<&str>,
    is_agree: bool,
    send_time: NaiveDateTime,
    is_send: bool,
) -> AppResult<u64> {
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
    .await
    .internal_err()?;
    Ok(res.last_insert_id())
}
