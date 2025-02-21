use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
    response::IntoResponse,
};
use tower_http::validate_request::ValidateRequestHeaderLayer;

use crate::utils::wrapper::error_json;

type FnAuth = fn(&mut Request<Body>) -> Result<(), Response<Body>>;

/// 验证请求头中的Authorization字段是否合法
#[inline]
pub fn auth_middleware() -> ValidateRequestHeaderLayer<FnAuth> {
    tower_http::validate_request::ValidateRequestHeaderLayer::custom(auth)
}

// 中间件逻辑为错误时直接返回Response，正确时不返回，但是保留对Response的修改
#[inline]
fn auth(req: &mut Request<Body>) -> Result<(), Response<Body>> {
    let jwt = req
        .headers()
        .get("Authorization")
        .ok_or_else(|| {
            (StatusCode::UNAUTHORIZED, error_json(401, "请携带token发送请求")).into_response()
        })?
        .to_str()
        .map_err(|_| {
            (StatusCode::UNAUTHORIZED, error_json(401, "Authorization字段无法解析为文本"))
                .into_response()
        })?
        .to_string();
    req.extensions_mut().insert(jwt);
    Ok(())
}
