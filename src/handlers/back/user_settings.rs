use crate::{
    app_result::{AppResult, AppState},
    extractors::Json,
    utils::jwt::parse_stu_id,
};
use axum::{extract::State, Extension};
use serde_json::Value;

pub async fn get_all_user_settings_handler(
    State(data): AppState,
    Extension(token): Extension<String>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let res = sqlx::query!(
        "
        SELECT settings FROM mini_user_settings WHERE stuid = ? LIMIT 1
        ",
        stu_id
    )
    .fetch_optional(&data.db)
    .await?
    .map(|r| r.settings);
    Ok(res.into())
}

pub async fn post_all_user_settings_handler(
    State(data): AppState,
    Extension(token): Extension<String>,
    Json(json): Json<Value>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    sqlx::query!(
        "
        INSERT INTO
        mini_user_settings (stuid, settings) 
        VALUES (?, ?) 
        ON DUPLICATE KEY UPDATE 
        settings = VALUES(settings)
        ",
        stu_id,
        json,
    )
    .execute(&data.db)
    .await?;
    Ok("设置提交成功".into())
}
