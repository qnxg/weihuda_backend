use super::get_db_pool;
use crate::result::AppResult;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Config {
    pub key: String,
    pub value: String,
}

pub async fn get_config(key: &str) -> AppResult<Option<Config>> {
    let res = sqlx::query_as!(
        Config,
        r#"
        SELECT 
            `key`, 
            value
        FROM 
            mini_configs 
        WHERE 
            `key` = ?
        "#,
        key,
    )
    .fetch_optional(get_db_pool().await)
    .await?;
    Ok(res)
}
