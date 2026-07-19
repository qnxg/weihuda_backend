use crate::utils;
use salvo::http::StatusCode;
use salvo::prelude::Json;
use salvo::{Response, Scribe};
use serde::Serialize;
use serde_json::Value;
use std::fmt::Display;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AppError {
    /// 错误类型
    error_kind: AppErrorKind,
    /// 返回给前端的错误信息
    show_msg: String,
    /// 呈现在日志上的错误信息，对于非 InternalError 类型的错误，该字段和 show_msg 相同
    log_msg: String,
    /// http 错误代码
    http_status: StatusCode,
}

#[derive(Debug, Clone)]
enum AppErrorKind {
    Other,
    PasswordError,
    InternalError,
}

impl Display for AppError {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{}", self.log_msg)
    }
}

pub type AppResult<T> = Result<T, AppError>;
pub struct Success(Value);
pub type RouterResult = AppResult<Success>;

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

impl AppError {
    /// 构造一个 show_msg 和 log_msg 都是 msg 的错误
    fn from_msg(
        msg: impl Into<String>,
        http_status: StatusCode,
    ) -> Self {
        let msg = msg.into();
        Self {
            error_kind: AppErrorKind::Other,
            show_msg: msg.clone(),
            log_msg: msg,
            http_status,
        }
    }
    pub fn timeout() -> Self {
        Self::from_msg(
            "请求超时, 请稍后重试",
            StatusCode::REQUEST_TIMEOUT,
        )
    }
    pub fn unauthorized() -> Self {
        Self::from_msg("未授权访问", StatusCode::UNAUTHORIZED)
    }
    pub fn parse_error() -> Self {
        Self::from_msg(
            "参数解析错误",
            StatusCode::UNPROCESSABLE_ENTITY,
        )
    }
    pub fn password_error() -> Self {
        let mut e =
            Self::from_msg("密码错误", StatusCode::UNAUTHORIZED);
        e.error_kind = AppErrorKind::PasswordError;
        e
    }
    /// 业务逻辑错误
    pub fn customized(msg: impl Into<String>) -> Self {
        Self::from_msg(msg, StatusCode::BAD_REQUEST)
    }
}

impl AppError {
    pub fn is_password_error(&self) -> bool {
        matches!(self.error_kind, AppErrorKind::PasswordError)
    }
    pub fn is_internal_error(&self) -> bool {
        matches!(self.error_kind, AppErrorKind::InternalError)
    }
    #[expect(unused)]
    pub fn is_other_error(&self) -> bool {
        matches!(self.error_kind, AppErrorKind::Other)
    }
}

impl Scribe for AppError {
    fn render(self, res: &mut Response) {
        res.stuff(
            self.http_status,
            Json(serde_json::json!({
                "code": self.http_status.as_u16(),
                "data": null,
                "msg": self.show_msg
            })),
        );
        let status_message = if self.is_internal_error() {
            format!("{}: {}", self.show_msg, self.log_msg)
        } else {
            self.show_msg.clone()
        };
        utils::record!(
            otel.status_code = "error",
            otel.status_description = %status_message,
        );
    }
}

pub struct InternalError {
    file: String,
    line: u32,
    column: u32,
    error_chain: String,
    error_msg: String,
    show_msg: String,
}

impl InternalError {
    fn new(
        loc: &std::panic::Location,
        error: String,
        error_chain: String,
    ) -> Self {
        Self {
            file: loc.file().to_string(),
            line: loc.line(),
            column: loc.column(),
            error_chain,
            error_msg: error,
            show_msg: "服务器出现了内部错误".to_string(),
        }
    }
    /// 添加额外的错误解释，比如
    ///
    /// ```ignore
    /// e.with("数据库连接失败")
    /// ```
    ///
    /// 于是最后在日志上的错误消息会是 "数据库连接失败: <e 的 display 格式>"
    ///
    /// 每次调用该函数都相当于在已有的 `Self::error_msg` 前面加 `error_msg: `
    pub fn with(self, error_msg: &str) -> Self {
        Self {
            error_msg: format!("{}: {}", error_msg, self.error_msg),
            ..self
        }
    }
    /// 重新设置呈现给前端的错误信息
    ///
    /// 默认呈现给前端的错误信息是: "服务器出现了内部错误"
    pub fn show(self, msg: &str) -> Self {
        Self {
            show_msg: msg.to_string(),
            ..self
        }
    }
}

impl From<InternalError> for AppError {
    fn from(e: InternalError) -> Self {
        tracing::error!(
            error = %e.error_msg,
            file = %e.file,
            line = %e.line,
            column = %e.column,
            error_chain = %e.error_chain,
        );
        AppError {
            error_kind: AppErrorKind::InternalError,
            show_msg: e.show_msg,
            log_msg: e.error_msg,
            http_status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub trait ThrowInternalError {
    fn internal_err(self) -> InternalError;
}

impl<E: std::error::Error> ThrowInternalError for E {
    #[track_caller]
    fn internal_err(self) -> InternalError {
        let loc = std::panic::Location::caller();
        InternalError::new(
            loc,
            format!("{}", self),
            utils::debug_error_chain(&self),
        )
    }
}

// 如果不单独开这个 trait，而是直接给 String 和 &str 实现 ThrowInternalError，则会出现编译错误
// Rust 编译器宣称标准库后续可能会给 String 和 &str 实现 std::error::Error，这就导致了冲突
pub trait ThrowInternalErrorMsg {
    /// 将 String 或是 &str 作为一个错误来构造 InternalError
    fn internal_err(self) -> InternalError;
}

impl ThrowInternalErrorMsg for String {
    #[track_caller]
    fn internal_err(self) -> InternalError {
        let loc = std::panic::Location::caller();
        InternalError::new(loc, self.clone(), self)
    }
}

impl ThrowInternalErrorMsg for &str {
    #[track_caller]
    fn internal_err(self) -> InternalError {
        let loc = std::panic::Location::caller();
        InternalError::new(loc, self.to_string(), self.to_string())
    }
}

pub trait ThrowInternalErrorResult<T> {
    fn internal_err(self) -> AppResult<T>;
}

impl<T, E: std::error::Error> ThrowInternalErrorResult<T>
    for Result<T, E>
{
    #[track_caller]
    fn internal_err(self) -> AppResult<T> {
        let loc = std::panic::Location::caller();
        self.map_err(|e| {
            InternalError::new(
                loc,
                e.to_string(),
                utils::debug_error_chain(&e),
            )
            .into()
        })
    }
}

// moka 经常会用到
impl From<Arc<AppError>> for AppError {
    fn from(e: Arc<AppError>) -> Self {
        e.as_ref().clone()
    }
}
