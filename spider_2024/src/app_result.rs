use crate::app_error::AppError;
use salvo::prelude::{Json, Response, Scribe, StatusCode};
use serde::Serialize;
use serde_json::Value;

pub type AppResult<T> = Result<T, AppError>;
pub type HandlerResult = AppResult<Success>;

pub struct Success(Value);

impl<T: Serialize> From<T> for Success {
    fn from(data: T) -> Self {
        Success(serde_json::json!({
            "code": 200,
            "data": data,
            "msg": "请求成功",
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
            AppError::PasswordError => res.stuff(
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "code": 401,
                    "data": Value::Null,
                    "msg": "账号密码错误",
                })),
            ),
            AppError::AnyHow(e) => res.stuff(
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "code": 500,
                    "data": Value::Null,
                    "msg": format!("服务器内部错误: {}", e),
                })),
            ),
            AppError::Timeout => res.stuff(
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "code": 503,
                    "data": Value::Null,
                    "msg": format!("服务器超时"),
                })),
            ),
            AppError::ParseError(e) => res.stuff(
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "code": 400,
                    "data": Value::Null,
                    "msg": format!("请求参数解析错误: {}", e),
                })),
            ),
            AppError::SqlxError(e) => res.stuff(
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "code": 500,
                    "data": Value::Null,
                    "msg": format!("数据库操作错误: {}", e),
                })),
            ),
            AppError::RedisErr(e) => res.stuff(
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "code": 500,
                    "data": Value::Null,
                    "msg": format!("Redis操作错误: {}", e),
                })),
            ),
            AppError::OtherErr(status_code, msg) => res.stuff(
                status_code,
                Json(serde_json::json!({
                    "code": status_code.as_u16(),
                    "data": Value::Null,
                    "msg": msg,
                })),
            ),
        };
    }
}

// 暂时放弃这种返回设计结构，因为在处理层要多加一层，不想多加层，会多写模板代码
// pub type AppResult<T> = Result<T, AppError>;
// /// 程序最终返回的数据结构，包含数据、状态码和消息，由此框架生成返回的json
// #[derive(Serialize)]
// pub struct AppResReturn<T: Serialize> {
//     pub data: Option<T>,
//     pub code: u16,
//     pub msg: String,
// }
//
// /// 返回为Response的实现
// impl<T: Serialize + Send> Scribe for AppResReturn<T> {
//     fn render(self, res: &mut Response) {
//         res.stuff(StatusCode::from_u16(self.code).unwrap(), Json(self));
//     }
// }
//
// // 从任何实现了Serialize trait的数据类型转换为AppRes
// impl<T: Serialize> From<T> for AppResReturn<T>
// {
//     fn from(data: T) -> Self {
//         AppResReturn {
//             data: Some(data),
//             code: 200,
//             msg: "请求成功".to_string(),
//         }
//     }
// }
//
// // 从AppError转换为AppRes的实现
// impl<T: Serialize> From<AppError> for AppResReturn<T> {
//     fn from(err: AppError) -> Self {
//         match err {
//             AppError::AnyHow(e) => AppResReturn {
//                 data: None,
//                 code: 500,
//                 msg: format!("服务器内部错误: {}", e),
//             },
//             AppError::ParseError(e) => AppResReturn {
//                 data: None,
//                 code: 400,
//                 msg: format!("请求参数解析错误: {}", e),
//             },
//             AppError::SqlxError(e) => AppResReturn {
//                 data: None,
//                 code: 500,
//                 msg: format!("数据库操作错误: {}", e),
//             },
//             AppError::RedisErr(e) => AppResReturn {
//                 data: None,
//                 code: 500,
//                 msg: format!("Redis操作错误: {}", e),
//             },
//         }
//     }
// }
