use super::get_db_pool;
use crate::result::AppResult;
use crate::utils::serde::deserialize_option_naive_datetime;
use chrono::NaiveDateTime;
use salvo::macros::Extractible;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Extractible)]
#[expect(non_snake_case)]
#[salvo(extract(default_source(from = "body")))]
pub struct ZhihuListItem {
    pub id: Option<i32>,
    pub title: String,
    #[serde(rename = "type")]
    pub _type: Option<String>,
    pub content: Option<String>,
    pub tags: Option<String>,
    pub cover: Option<String>,
    pub status: Option<i32>,
    #[serde(deserialize_with = "deserialize_option_naive_datetime")]
    pub publishTime: Option<NaiveDateTime>,
    pub stuId: Option<String>,
}

/// title，typ，tags 是模糊匹配，如果传 None 则不进行过滤
pub async fn get_zhihu_list(
    title: Option<String>,
    typ: Option<String>,
    tags: Option<String>,
    stu_id: &str,
    offset: u32,
    count: u32,
) -> AppResult<Vec<ZhihuListItem>> {
    let res: Vec<ZhihuListItem> = sqlx::query_as!(
        ZhihuListItem,
        r#"
        SELECT 
            id, 
            title, 
            type AS _type, 
            tags, 
            cover, 
            IF(type = 'link', content, NULL) AS content, 
            status, 
            publishTime, 
            stuId 
        FROM 
            zhihus 
        WHERE 
            (title LIKE ? AND type LIKE ? AND tags LIKE ?) 
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
    .await?;
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
            (title LIKE ? AND type LIKE ? AND tags LIKE ?) 
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
    let res: Option<ZhihuListItem> = sqlx::query_as!(
        ZhihuListItem,
        r#"
        SELECT 
            id, 
            title, 
            type AS _type, 
            tags, 
            cover, 
            content, 
            status, 
            publishTime, 
            stuId 
        FROM 
            zhihus 
        WHERE 
            id = ? AND deletedAt IS NULL
        "#,
        id
    )
    .fetch_optional(get_db_pool().await)
    .await?;
    Ok(res)
}

pub async fn add_zhihu(item: ZhihuListItem) -> AppResult<u32> {
    let now = chrono::Local::now();
    let res = sqlx::query!(
        r#"
        INSERT INTO zhihus 
            (title, type, tags, cover, content, status, publishTime, stuId, createdAt, updatedAt) 
        VALUES 
            (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        item.title,
        item._type,
        item.tags,
        item.cover,
        item.content,
        item.status,
        item.publishTime,
        item.stuId,
        now,
        now
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(res.last_insert_id() as u32)
}

pub async fn update_zhihu(
    id: u32,
    item: ZhihuListItem,
) -> AppResult<()> {
    let now = chrono::Local::now();
    // 插入到数据库中
    let _ = sqlx::query!(
        r#"
        UPDATE zhihus 
        SET 
            title = ?, 
            type = ?, 
            tags = ?, 
            cover = ?, 
            content = ?, 
            status = ?, 
            publishTime = ?, 
            stuId = ?, 
            updatedAt = ? 
        WHERE 
            id = ? AND deletedAt IS NULL;
        "#,
        item.title,
        item._type,
        item.tags,
        item.cover,
        item.content,
        item.status,
        item.publishTime,
        item.stuId,
        now,
        id,
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}

pub async fn delete_zhihu(id: u32) -> AppResult<()> {
    let now = chrono::Local::now();
    let _ = sqlx::query!(
        r#"
        Update zhihus
        SET deletedAt = ?
        WHERE id = ? AND deletedAt IS NULL;
        "#,
        now,
        id
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}
