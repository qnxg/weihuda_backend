use super::get_db_pool;
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde::ser::Serializer;

use crate::result::AppResult;

#[derive(Serialize, Debug)]
pub struct AnnouncementInfo {
    #[serde(
        rename = "create_at",
        serialize_with = "serialize_as_date"
    )]
    pub created_at: Option<DateTime<Utc>>,
    pub url: Option<String>,
    pub title: String,
    pub content: String,
    pub id: u32,
}

fn serialize_as_date<S>(
    date: &Option<DateTime<Utc>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match date {
        Some(date) => {
            let s = date
                .with_timezone(
                    &chrono::FixedOffset::east_opt(8 * 3600)
                        .expect("创建时区失败"),
                )
                .naive_local()
                .to_string()
                .split(' ')
                .next()
                .unwrap_or("")
                .to_string();
            serializer.serialize_str(&s)
        }
        None => serializer.serialize_none(),
    }
}

/// count 表示最多获取多少条消息
pub async fn get_announcement_list(
    count: u32,
) -> AppResult<Vec<AnnouncementInfo>> {
    let announcement = sqlx::query_as!(
        AnnouncementInfo,
        r#"
        SELECT title, content, url, id, created_at FROM mini_message WHERE deleted_at IS NULL LIMIT ?
        "#,
        count as i64
    )
    .fetch_all(get_db_pool().await)
    .await?;
    Ok(announcement)
}
