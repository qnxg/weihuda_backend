//! 覆盖掉Json解析的Rejection类型，使用AppError作为Rejection类型以便程序整体输出格式保持一致。
use crate::app_error::AppError;
use axum::extract::rejection::JsonRejection;
use axum::extract::FromRequest;

// create an extractor that internally uses `axum::extract::Json` but has a custom rejection
#[derive(FromRequest)]
#[from_request(via(axum::extract::Json), rejection(AppError))]
pub struct Json<T>(pub T);

impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        AppError::JsonError(rejection.to_string())
    }
}
