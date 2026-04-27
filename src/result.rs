use std::sync::Arc;

use salvo::http::StatusCode;
use salvo::prelude::Json;
use salvo::{Response, Scribe};
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

/// 自定义的错误处理类型，支持多种错误类型，可以通过?操作符链式传播，传播链的初始类型必须是可转换为AppError的分支的类型
#[derive(Error, Debug, Clone)]
pub enum AppError {
    /// 服务器内部错误或是正常的业务错误
    #[error("{0:?}")]
    Text(String),
    /// 参数解析错误
    #[error("参数解析错误")]
    ParseError,
    /// 密码错误
    ///
    /// 下发该错误后前端会强制下线
    #[error("密码错误")]
    PasswordError,
    /// 没有提供 jwt 或是 jwt 解析错误
    #[error("未授权访问")]
    Unauthorized,
    #[error("请求超时, 请稍后重试")]
    TimeoutError,
}
pub struct Success(Value);
impl<T: Serialize> From<T> for Success {
    fn from(value: T) -> Self {
        Success(serde_json::json!({
            "code": 200,
            "data": value,
            "msg": "请求成功"
        }))
    }
}
impl Scribe for Success {
    fn render(self, res: &mut Response) {
        res.stuff(StatusCode::OK, Json(self.0));
    }
}

impl Scribe for AppError {
    fn render(self, res: &mut Response) {
        match self {
            AppError::Text(text) => res.stuff(
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "code": 500,
                    "data": null,
                    "msg": text
                })),
            ),
            AppError::ParseError => res.stuff(
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "code": 400,
                    "data": null,
                    "msg": "参数解析错误"
                })),
            ),
            AppError::Unauthorized => {
                res.stuff(
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "code": 401,
                        "data": null,
                        "msg": "未授权访问"
                    })),
                );
            }
            AppError::PasswordError => {
                res.stuff(
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "code": 401,
                        "data": null,
                        "msg": "密码错误(NO_TOAST)"
                    })),
                );
            }
            AppError::TimeoutError => {
                res.stuff(
                    StatusCode::REQUEST_TIMEOUT,
                    Json(serde_json::json!({
                        "code": 408,
                        "data": null,
                        "msg": "请求超时，请重试"
                    })),
                );
            }
        }
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::Text(s.to_string())
    }
}

impl From<salvo::http::ParseError> for AppError {
    fn from(_: salvo::http::ParseError) -> Self {
        AppError::ParseError
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(_: jsonwebtoken::errors::Error) -> Self {
        AppError::Unauthorized
    }
}

// moka 经常会用到
impl From<Arc<AppError>> for AppError {
    fn from(e: Arc<AppError>) -> Self {
        e.as_ref().clone()
    }
}

/// 抛出错误，并返回 AppError::Text("请求失败，请稍后重试")
///
/// 会把错误打印到日志上，同时日志上还会显示抛出错误的位置信息
///
/// `reason` 将会显示到日志上
fn throw_error_with_loc<E: std::error::Error>(
    loc: &std::panic::Location,
    e: E,
    reason: &str,
) -> AppError {
    tracing::error!(
        error = ?e,
        file = %loc.file(),
        line = %loc.line(),
        column = %loc.column(),
        "{}", reason
    );
    AppError::Text("请求失败，请稍后重试".to_string())
}

/// SEE ALSO [throw_error_with_loc]
#[track_caller]
pub fn throw_error<E: std::error::Error>(
    e: E,
    reason: &str,
) -> AppError {
    let loc = std::panic::Location::caller();
    throw_error_with_loc(loc, e, reason)
}

pub trait ThrowError<T> {
    fn throw_error(self, reason: &str) -> AppResult<T>;
}

impl<T, E: std::error::Error> ThrowError<T> for Result<T, E> {
    /// SEE ALSO [throw_error_with_loc]
    #[track_caller]
    fn throw_error(self, reason: &str) -> AppResult<T> {
        match self {
            Ok(value) => Ok(value),
            Err(e) => {
                let loc = std::panic::Location::caller();
                Err(throw_error_with_loc(loc, e, reason))
            }
        }
    }
}

impl From<sqlx::Error> for AppError {
    #[track_caller]
    fn from(e: sqlx::Error) -> Self {
        let loc = std::panic::Location::caller();
        throw_error_with_loc(loc, e, "数据库操作失败")
    }
}

pub type AppResult<T> = Result<T, AppError>;
pub type RouterResult = AppResult<Success>;
