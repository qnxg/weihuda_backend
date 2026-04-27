//! 在 mysql 里做的一个持久化的缓存表（虽说某种意义上并不算缓存）
use super::Result;
use super::get_db_pool;
use crate::utils::time::now_time;
use chrono::NaiveDateTime;

pub async fn get(
    key: &str,
) -> Result<Option<(String, NaiveDateTime)>> {
    let value = sqlx::query!(
        r#"
        SELECT value, update_at FROM kv_cache WHERE `key` = ?
    "#,
        key
    )
    .fetch_optional(get_db_pool().await)
    .await?
    .map(|r| (r.value, r.update_at));
    Ok(value)
}

/// 插入或更新缓存
pub async fn insert(key: &str, value: &str) -> Result<()> {
    let now = now_time();
    sqlx::query!(
        r#"
        INSERT INTO kv_cache (`key`, value, update_at)
        VALUES (?, ?, ?)
        ON DUPLICATE KEY UPDATE
        value = VALUES(value),
        update_at = VALUES(update_at)
    "#,
        key,
        value,
        now
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}

pub async fn delete(key: &str) -> Result<()> {
    sqlx::query!(
        r#"
        DELETE FROM kv_cache WHERE `key` = ?
    "#,
        key
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}
