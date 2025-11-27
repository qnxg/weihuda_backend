use super::get_db_pool;
use crate::result::AppResult;
use serde_json::Value;

pub async fn get_user_setting(
    stu_id: &str,
) -> AppResult<Option<Value>> {
    let res = sqlx::query!(
        "
        SELECT settings FROM mini_user_settings WHERE stuid = ? LIMIT 1
        ",
        stu_id
    )
    .fetch_optional(get_db_pool().await)
    .await?
    .map(|r| r.settings);
    Ok(res)
}

pub async fn update_user_setting(
    stu_id: &str,
    settings: &Value,
) -> AppResult<()> {
    sqlx::query!(
        "
        INSERT INTO
        mini_user_settings (stuid, settings) 
        VALUES (?, ?) 
        ON DUPLICATE KEY UPDATE 
        settings = VALUES(settings)
        ",
        stu_id,
        settings,
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}
