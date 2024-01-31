use crate::{app_error::AppError, utility::wrapper::success_json};
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use serde_json::Value;

/// Result<AppRes, AppError>的type alias，用于简化返回类型书写，由于AppRes和AppError都实现了IntoResponse trait，所以可以直接在handler中返回该类型
pub type AppResult = Result<AppRes, AppError>;

/// 对返回数据进行包装，方便实现IntoResponse trait
pub struct AppRes(Json<Value>);

// 返回的数据必须实现Serialize trait，这样才能被序列化为json，在返回数据处调用.into()方法即可直接完成返回数据的包装
impl<T: Serialize> From<T> for AppRes {
    fn from(data: T) -> Self {
        AppRes(success_json(data))
    }
}

// 为AppRes实现IntoResponse trait，这样才能在handler中直接返回
impl IntoResponse for AppRes {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, self.0).into_response()
    }
}
