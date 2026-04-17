use anyhow::anyhow;
use salvo::http::StatusCode;
use salvo::prelude::Json;
use salvo::{Response, Scribe};
use serde::Serialize;
use serde_json::Value;
use spider_2024::Error as SpiderError;
use thiserror::Error;

/// 自定义的错误处理类型，支持多种错误类型，可以通过?操作符链式传播，传播链的初始类型必须是可转换为AppError的分支的类型
#[derive(Error, Debug)]
pub enum AppError {
    /// 未知错误类型，500，服务器内部错误
    #[error("服务器内部错误: {0:?}")]
    AnyHow(#[from] anyhow::Error),
    /// 参数解析错误
    #[error("参数解析错误: {0}")]
    SalvoParseError(#[from] salvo::http::ParseError),
    #[error("参数解析错误")]
    ParseError(),
    /// Sqlx数据库操作错误，500，服务器内部错误
    #[error("数据库SQL语句执行错误: {0}")]
    SqlxError(#[from] sqlx::Error),
    /// JWT编解码错误
    #[error("JWT编解码错误: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    #[error("密码错误")]
    PasswordError,
    #[error("解析JSON错误: {0}")]
    JsonParseError(#[from] serde_json::Error),
    #[error("内部请求错误: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("RabbitMQ错误: {0}")]
    RabbitMQError(#[from] lapin::Error),
    #[error("未授权访问")]
    Unauthorized,
    #[error("请求超时")]
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

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::AnyHow(anyhow::anyhow!(s.to_string()))
    }
}

impl From<SpiderError> for AppError {
    fn from(e: SpiderError) -> Self {
        match e {
            SpiderError::AnyHow(error) => Self::AnyHow(error),
            SpiderError::PasswordError => Self::PasswordError,
            SpiderError::PasswordShouldChange => Self::PasswordError,
            SpiderError::PasswordLocked => Self::AnyHow(anyhow!(
                "账号被锁定，请暂停使用10分钟之后重试。"
            )),
            SpiderError::SqlxError(error) => Self::SqlxError(error),
        }
    }
}

impl Scribe for AppError {
    fn render(self, res: &mut Response) {
        tracing::error!("{}", self);
        match self {
            AppError::AnyHow(_)
            | AppError::JsonParseError(_)
            | AppError::RequestError(_)
            | AppError::RabbitMQError(_)
            | AppError::Unauthorized => res.stuff(
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "code": 500,
                    "data": null,
                    "msg": format!("{}", self)
                })),
            ),
            AppError::ParseError() | AppError::SalvoParseError(_) => {
                // 错误信息默认只提示消耗stream流过程中第一个缺失的字段
                res.stuff(
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "code": 400,
                        "data": null,
                        "msg": "参数解析错误"
                    })),
                )
            }
            AppError::SqlxError(_) => {
                res.stuff(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "code": 500,
                        "data": null,
                        "msg": "数据库内部错误"
                    })),
                );
            }
            AppError::JwtError(_) => {
                res.stuff(
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "code": 401,
                        "data": null,
                        "msg": "身份验证错误"
                    })),
                );
            }
            AppError::PasswordError => {
                // 密码错误信息交给爬虫打印
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

pub type AppResult<T> = Result<T, AppError>;
pub type RouterResult = AppResult<Success>;
