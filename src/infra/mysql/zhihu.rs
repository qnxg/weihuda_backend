use super::get_db_pool;
use crate::result::AppResult;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ZhihuListItem {
    pub id: u32,
    pub stu_id: String,
    pub created_at: NaiveDateTime,
    pub title: String,
    pub typ: String,
    pub content: String,
    pub tags: String,
    pub cover: Option<String>,
    pub top: bool,
    pub status: u32,
}

/// title，typ，tags 是模糊匹配，如果传 None 则不进行过滤
/// 仅显示已发布状态的，或是自己发布的知湖
pub async fn get_zhihu_list(
    title: Option<String>,
    typ: Option<String>,
    tags: Option<String>,
    stu_id: &str,
    offset: u32,
    count: u32,
) -> AppResult<Vec<ZhihuListItem>> {
    let res: Vec<ZhihuListItem> = sqlx::query!(
        r#"
        SELECT 
            id, 
            title, 
            typ, 
            tags, 
            content,
            cover,  
            status, 
            stuId,
            createdAt,
            top
        FROM 
            zhihus 
        WHERE 
            (title LIKE ? AND typ LIKE ? AND tags LIKE ?) 
            AND (status = 1 OR stuId = ?)
            AND deletedAt IS NULL
        ORDER BY 
            id DESC 
        LIMIT 
            ?, ?;
        "#,
        format!("%{}%", title.unwrap_or_default()),
        format!("%{}%", typ.unwrap_or_default()),
        format!("%{}%", tags.unwrap_or_default()),
        stu_id,
        offset,
        count
    )
    .fetch_all(get_db_pool().await)
    .await?
    .into_iter()
    .map(|r| ZhihuListItem {
        id: r.id,
        title: r.title,
        typ: r.typ,
        content: r.content,
        tags: r.tags,
        cover: r.cover,
        status: r.status,
        top: r.top == 1,
        stu_id: r.stuId,
        created_at: r.createdAt,
    })
    .collect();
    Ok(res)
}

pub async fn get_zhihu_count(
    title: Option<String>,
    typ: Option<String>,
    tags: Option<String>,
    stu_id: &str,
) -> AppResult<u32> {
    let rec = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) AS count
        FROM 
            zhihus 
        WHERE 
            (title LIKE ? AND typ LIKE ? AND tags LIKE ?) 
            AND (status = 1 OR stuId = ?)
            AND deletedAt IS NULL
        "#,
        format!("%{}%", title.unwrap_or_default()),
        format!("%{}%", typ.unwrap_or_default()),
        format!("%{}%", tags.unwrap_or_default()),
        stu_id
    )
    .fetch_one(get_db_pool().await)
    .await?;
    Ok(rec.count as u32)
}

pub async fn get_zhihu_by_id(
    id: u32,
) -> AppResult<Option<ZhihuListItem>> {
    let res: Option<ZhihuListItem> = sqlx::query!(
        r#"
        SELECT 
            id, 
            title, 
            typ, 
            tags, 
            content,
            cover, 
            status, 
            stuId,
            createdAt,
            top
        FROM 
            zhihus 
        WHERE 
            id = ? AND deletedAt IS NULL
        "#,
        id
    )
    .fetch_optional(get_db_pool().await)
    .await?
    .map(|r| ZhihuListItem {
        id: r.id,
        title: r.title,
        typ: r.typ,
        content: r.content,
        tags: r.tags,
        cover: r.cover,
        status: r.status,
        top: r.top == 1,
        stu_id: r.stuId,
        created_at: r.createdAt,
    });
    Ok(res)
}
