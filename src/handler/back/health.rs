use crate::app_result::AppResult;

static MESSAGE: &str = "pong!";

pub async fn health_checker_handler() -> AppResult {
    Ok(MESSAGE.into())
}
