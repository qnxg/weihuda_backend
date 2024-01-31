use crate::{app_result::AppResult, model::back::message::MessageInfo, Pool};
use axum::extract::State;
use std::sync::Arc;

pub async fn get_message_handler(State(data): State<Arc<Pool>>) -> AppResult {
    let messages = sqlx::query_as!(
        MessageInfo,
        r#"
        SELECT title, content, url, id, created_at FROM mini_message WHERE deleted_at IS NULL LIMIT 10
        "#,
    )
    .fetch_all(&data.db)
    .await?;

    Ok(messages.into())
}
