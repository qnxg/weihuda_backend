//! 覆盖掉Query解析的Rejection类型，使用AppError作为Rejection类型以便程序整体输出格式保持一致。
use crate::app_error::AppError;
use axum::extract::rejection::QueryRejection;
use axum::extract::FromRequestParts;

// create an extractor that internally uses `axum::extract::Query` but has a custom rejection
// since FromRequest will consume the whole body, we can only use FromRequestParts because Json would be the last to be consumed, not Query.
#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Query), rejection(AppError))]
pub struct Query<T>(pub T);

impl From<QueryRejection> for AppError {
    fn from(rejection: QueryRejection) -> Self {
        AppError::QueryError(rejection.to_string())
    }
}
