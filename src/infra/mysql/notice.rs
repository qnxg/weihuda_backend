use super::get_db_pool;
use crate::result::AppResult;
use chrono::NaiveDateTime;
use serde::Serialize;

#[expect(non_snake_case)]
#[derive(Serialize, Debug)]
pub struct Notice {
    pub id: u32,
    pub content: String,
    pub stuId: String,
    pub sendTime: NaiveDateTime,
    pub isShow: Option<i8>,
    pub status: Option<i32>,
    pub result: Option<String>,
    pub btnConfig: Option<String>,
}

pub async fn get_notice_list(
    stu_id: &str,
    page: u32,
    page_size: u32,
) -> AppResult<Vec<Notice>> {
    let res = sqlx::query_as!(
        Notice,
        r#"
        SELECT 
            id,
            content,
            stuId,
            sendTime,
            isShow,
            status,
            result,
            btnConfig
        From 
            notices
        WHERE 
            stuId = ?
            AND deletedAt IS NULL
        ORDER BY 
            id DESC
        LIMIT 
            ?, ?
        "#,
        stu_id,
        (page - 1) * page_size,
        page_size,
    )
    .fetch_all(get_db_pool().await)
    .await?;
    Ok(res)
}

/// result 和 status 如果是 None 就不更新
pub async fn update_notice(
    id: u32,
    result: Option<&String>,
    status: Option<i32>,
) -> AppResult<()> {
    if let Some(result) = result {
        sqlx::query!(
            r#"
            UPDATE 
                notices
            SET 
                result = ?
            WHERE 
                id = ?
            "#,
            result,
            id,
        )
        .execute(get_db_pool().await)
        .await?;
    }
    if let Some(status) = status {
        sqlx::query!(
            r#"
            UPDATE
                notices
            SET 
                status = ?
            WHERE 
                id = ?
            "#,
            status,
            id,
        )
        .execute(get_db_pool().await)
        .await?;
    }
    Ok(())
}
