use tower_http::timeout::TimeoutLayer;

/// 请求超时中间件，请求超时情况可以根据StatusCode判断，这里不再自定义错误返回信息，使用默认配置
#[inline]
pub fn timeout_middleware() -> TimeoutLayer {
    TimeoutLayer::new(std::time::Duration::from_secs(5))
}

// 用tower结合axum自定义超时中间件，大致逻辑如下
// pub time_out_middleware() {
//     ServiceBuilder::new()
//         .layer(HandleErrorLayer::new(|err: BoxError| {
//             if err.is::<tower::timeout::error::Elapsed>() {
//                 Ok::<_, axum::body::Body>((
//                     axum::http::StatusCode::REQUEST_TIMEOUT,
//                     "Request took too long".into(),
//                 ))
//             } else {
//                 Err(err)
//             }
//         }))
//         .timeout(std::time::Duration::from_secs(5))
// }
