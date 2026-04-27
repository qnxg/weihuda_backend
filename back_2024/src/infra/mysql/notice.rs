use super::Result;
use super::get_db_pool;
use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Notice {
    pub id: u32,
    pub content: String,
    pub stu_id: String,
    pub is_show: bool,
    pub status: u32,
    pub url: Option<String>,
    pub created_at: NaiveDateTime,
}

pub async fn get_notice_list(
    stu_id: &str,
    page: u32,
    page_size: u32,
) -> Result<Vec<Notice>> {
    let res = sqlx::query!(
        r#"
        SELECT 
            id,
            content,
            stuId,
            isShow,
            status,
            url,
            createdAt
        FROM
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
    .await?
    .into_iter()
    .map(|r| Notice {
        id: r.id,
        content: r.content,
        stu_id: r.stuId,
        is_show: r.isShow != 0,
        status: r.status,
        url: r.url,
        created_at: r.createdAt,
    })
    .collect();
    Ok(res)
}

/// result 和 status 如果是 None 就不更新
pub async fn update_notice(id: u32, status: u32) -> Result<()> {
    sqlx::query!(
        r#"
            UPDATE 
                notices
            SET 
                status = ?
            WHERE 
                id = ? AND deletedAt IS NULL
            "#,
        status,
        id,
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}
