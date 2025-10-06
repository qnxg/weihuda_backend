use axum::extract::State;
use serde::Serialize;

use crate::{
    app_result::{AppResult, AppState},
    dtos::back::config::GetConfigReq,
    extractors::Query,
};

#[derive(Serialize, Debug)]
pub struct ConfigRes {
    pub count: usize,
    pub rows: Vec<Config>,
}

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

#[expect(non_snake_case)]
pub async fn get_config_handler(
    State(data): AppState,
    Query(req): Query<GetConfigReq>,
) -> AppResult {
    let page = req.page.unwrap_or(1);
    let pageSize = req.pageSize.unwrap_or(10);
    let like = format!("%{}%", req.key.unwrap_or_default());
    let offset = (page - 1) * pageSize;

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
        pageSize
    )
    .fetch_all(&data.db)
    .await?;

    let res = ConfigRes {
        count: res.len(),
        rows: res,
    };

    Ok(res.into())
}
