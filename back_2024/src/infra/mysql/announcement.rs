use super::get_db_pool;
use chrono::NaiveDateTime;
use serde::Serialize;

use crate::result::AppResult;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AnnouncementInfo {
    pub url: Option<String>,
    pub title: String,
    pub content: String,
    pub id: u32,
    pub created_at: NaiveDateTime,
}

/// count 表示最多获取多少条消息
pub async fn get_announcement_list(
    count: u32,
) -> AppResult<Vec<AnnouncementInfo>> {
    let announcement = sqlx::query_as!(
        AnnouncementInfo,
        r#"
        SELECT title, content, url, id, createdAt as 'created_at' FROM announcement WHERE deletedAt IS NULL LIMIT ?
        "#,
        count
    )
    .fetch_all(get_db_pool().await)
    .await?;
    Ok(announcement)
}
