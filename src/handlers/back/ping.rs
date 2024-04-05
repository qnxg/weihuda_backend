use crate::app_result::AppResult;

static MESSAGE: &str = "I am fine!";

pub async fn health_checker_handler() -> AppResult {
    Ok(MESSAGE.into())
}
