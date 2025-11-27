use super::get_db_pool;
use crate::result::AppResult;
use chrono::NaiveDateTime;

pub async fn add_left_message(
    stu_id: &str,
    desc: &str,
    is_agree: bool,
    send_time: NaiveDateTime,
    is_send: bool,
) -> AppResult<u64> {
    let res = sqlx::query!(
        r#"
        INSERT INTO 
            message_lefts (stuId, `desc`, isAgree, sendTime,isSend)
        VALUES 
            (?, ?, ?, ?,?)
        "#,
        stu_id,
        desc,
        is_agree,
        send_time,
        is_send,
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(res.last_insert_id())
}
