use std::sync::Arc;

use reqwest::StatusCode;
use salvo::http::ParseError;
use serde_json::Value;
use thiserror::Error;

/// 应用级的错误类型，用于统一处理错误
///
/// 其中Sqlx数据库和Redis的错误不常见，因为主要用于密码的获取操作
#[derive(Error, Debug)]
pub enum AppError {
    // 不需要特殊处理的错误
    #[error(transparent)]
    AnyHow(#[from] anyhow::Error),
    // 登录服务时提供的账号密码错误，前端可能需要重新处理
    #[error("password error")]
    PasswordError,
    #[error("password should change")]
    PasswordShouldChange,
    #[error("password is locked")]
    PasswordLocked,
    #[error("timeout error")]
    Timeout,
    #[error("salvo::http::ParseError:`{0}`")]
    ParseError(#[from] ParseError),
    #[error("sqlx::Error:`{0}`")]
    SqlxError(#[from] sqlx::Error),
    #[error("redis::RedisError:`{0}`")]
    RedisErr(#[from] redis::RedisError),
    #[error("`{0}`")]
    OtherErr(StatusCode, Value),
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::AnyHow(anyhow::anyhow!(err))
    }
}

impl From<reqwest::header::ToStrError> for AppError {
    fn from(err: reqwest::header::ToStrError) -> Self {
        AppError::AnyHow(anyhow::anyhow!(err))
    }
}

impl From<reqwest::header::InvalidHeaderValue> for AppError {
    fn from(err: reqwest::header::InvalidHeaderValue) -> Self {
        AppError::AnyHow(anyhow::anyhow!(err))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::AnyHow(anyhow::anyhow!(err))
    }
}

impl From<Arc<AppError>> for AppError {
    fn from(err: Arc<AppError>) -> Self {
        match &(*err) {
            AppError::AnyHow(_)
            | AppError::ParseError(_)
            | AppError::SqlxError(_)
            | AppError::RedisErr(_)
            | AppError::OtherErr(_, _) => {
                AppError::AnyHow(anyhow::anyhow!(err))
            }
            AppError::PasswordError => AppError::PasswordError,
            AppError::PasswordShouldChange => {
                AppError::PasswordShouldChange
            }
            AppError::PasswordLocked => AppError::PasswordLocked,
            AppError::Timeout => AppError::Timeout,
        }
    }
}

impl From<std::num::ParseIntError> for AppError {
    fn from(err: std::num::ParseIntError) -> Self {
        AppError::AnyHow(anyhow::anyhow!(err))
    }
}

impl From<std::time::SystemTimeError> for AppError {
    fn from(err: std::time::SystemTimeError) -> Self {
        AppError::AnyHow(anyhow::anyhow!(err))
    }
}
