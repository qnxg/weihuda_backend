use axum::http::Method;
use tower_http::cors::{Any, CorsLayer};

/// 跨域中间件
#[inline]
pub fn cors_middleware() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::PUT])
        .allow_headers(Any)
    // .allow_headers([AUTHORIZATION])
}

// CorsLayer::new()
//     .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
//     .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
//     .allow_credentials(true)
//     .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
