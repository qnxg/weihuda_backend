use axum::Json;
use serde::Serialize;
use serde_json::Value;

#[inline]
pub fn success_json<T: Serialize>(data: T) -> Json<Value> {
    let res = serde_json::json!({
        "code": 200,
        "data": data,
        "msg": "请求成功",
    });
    Json(res)
}

#[inline]
pub fn error_json(code: u16, msg: &str) -> Json<Value> {
    let res = serde_json::json!({
        "code": code,
        "data": Value::Null,
        "msg": msg,
    });
    Json(res)
}
