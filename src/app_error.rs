//! 自定义错误处理方便简洁优美地统一处理错误，程序逻辑基本不用关心错误处理，只需要使用？传递错误，在最后统一处理即可。

use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;

use crate::utils::wrapper::error_json;

/// 自定义的错误处理类型，支持多种错误类型，可以通过?操作符链式传播，传播链的初始类型必须是可转换为AppError的分支的类型
#[derive(Error, Debug)]
pub enum AppError {
    /// 未知错误类型，500，服务器内部错误
    #[error("服务器内部错误: {0:?}")]
    AnyHow(#[from] anyhow::Error),
    /// Query参数解析错误，400，请求与接口定义不符合
    #[error("Query参数错误: {0}")]
    QueryError(#[from] serde_urlencoded::de::Error),
    /// Json参数解析错误，400，请求与接口定义不符合
    #[error("Json请求体参数错误: {0}")]
    JsonError(#[from] serde_path_to_error::Error<serde_json::Error>),
    /// Sqlx数据库操作错误，500，服务器内部错误
    #[error("数据库SQL语句执行错误: {0}")]
    SqlxError(#[from] sqlx::Error),
    /// JWT编解码错误
    #[error("JWT编解码错误: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    /// 时间解析错误
    #[error("时间解析错误: {0}")]
    ParseError(#[from] chrono::ParseError),
    // Axum框架错误
    // #[error("Axum error: {0}")]
    // AxumError(#[from] axum::Error),
}

// 方便快速创建一个AppError
impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::AnyHow(anyhow::anyhow!(s.to_string()))
    }
}

// 为AppError实现IntoResponse trait，这样才能在handler中直接返回
impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::AnyHow(e) => {
                tracing::error!("服务器内部错误 {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, error_json(500, &format!("内部错误: {}", e)))
                    .into_response()
                // 展示给前端方便定位错误原因
            }
            AppError::QueryError(e) => {
                tracing::error!("Query参数错误 {}", e);
                (StatusCode::BAD_REQUEST, error_json(400, "参数解析错误")).into_response()
                // 错误信息默认只提示消耗stream流过程中第一个缺失的字段
            }
            AppError::JsonError(e) => {
                tracing::error!("Json请求体参数错误 {}", e);
                (StatusCode::BAD_REQUEST, error_json(400, "Body中Json参数解析错误"))
                    .into_response()
            }
            AppError::SqlxError(e) => {
                tracing::error!("数据库SQL语句执行错误 {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, error_json(500, "数据库内部错误"))
                    .into_response()
            }
            AppError::JwtError(e) => {
                tracing::error!("JWT编解码错误 {}", e);
                (StatusCode::UNAUTHORIZED, error_json(401, "身份验证错误")).into_response()
            }
            AppError::ParseError(e) => {
                tracing::error!("时间解析错误 {}", e);
                (StatusCode::BAD_REQUEST, error_json(400, "时间解析错误")).into_response()
            }
        }
    }
}
