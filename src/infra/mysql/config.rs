use super::get_db_pool;
use crate::result::AppResult;
use serde::Serialize;

#[expect(non_snake_case)]
#[derive(Serialize, Debug)]
pub struct Config {
    pub id: u32,
    pub key: String,
    pub value: String,
    pub description: Option<String>,
    pub valueType: String,
    pub enabled: Option<i8>,
}

pub async fn get_config_list(
    like: &str,
    offset: u32,
    page_size: u32,
) -> AppResult<Vec<Config>> {
    let res = sqlx::query_as!(
        Config,
        r#"
        SELECT 
            id, 
            `key`, 
            value, 
            description, 
            valueType,
            enabled
        FROM 
            mini_configs 
        WHERE 
            `key` LIKE ? AND enabled = 1
        ORDER BY 
            id DESC 
        LIMIT 
            ?, ?;
        "#,
        like,
        offset,
        page_size
    )
    .fetch_all(get_db_pool().await)
    .await?;
    Ok(res)
}
