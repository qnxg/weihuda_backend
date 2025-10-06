use crate::app_result::{AppResult, AppState};
use crate::entities::back::flex_time::FlexTime;
use axum::extract::State;
use tracing::error;

pub async fn get_flex_time_handler(
    State(data): AppState,
) -> AppResult {
    // 获取调休信息
    let flex_time = sqlx::query!(
        "SELECT value FROM mini_configs WHERE `key` = ? AND enabled = 1",
        "flexTime"
    )
    .fetch_one(&data.db)
    .await?
    .value;
    // 解析到json
    let flex_time: Vec<FlexTime> = serde_json::from_str(&flex_time)
        .map_err(|_| {
        error!("解析调休信息失败");
        anyhow::anyhow!("解析调休信息失败")
    })?;

    Ok(flex_time.into())
}
