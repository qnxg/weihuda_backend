use tower_http::{
    trace,
    trace::{HttpMakeClassifier, TraceLayer},
};
use tracing::Level;

/// 日志中间件，为所有请求和响应增加日志
#[inline]
pub fn log_middleware() -> TraceLayer<HttpMakeClassifier> {
    TraceLayer::new_for_http()
        .make_span_with(
            trace::DefaultMakeSpan::new().level(Level::ERROR),
        )
        .on_request(
            trace::DefaultOnRequest::new().level(Level::DEBUG),
        )
        .on_response(
            trace::DefaultOnResponse::new().level(Level::DEBUG),
        )
        .on_failure(
            trace::DefaultOnFailure::new().level(Level::DEBUG),
        )
}

// Register the tracing subscriber
// tracing_subscriber::fmt()
//     // .with_target(false)
//     .with_max_level(tracing::Level::DEBUG)
//     .compact()
//     .init();
