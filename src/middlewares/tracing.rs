use crate::utils;
use reqwest::StatusCode;
use salvo::{
    Depot, FlowCtrl, Request, Response, handler,
    http::header::HeaderValue,
};
use tracing::Instrument;
use uuid::Uuid;

#[handler]
pub async fn tracing_middleware(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    // 优先使用上游传入的 X-Request-Id，没有则生成新的 UUID
    let request_id = req
        .headers()
        .get("X-Request-Id")
        .and_then(|x| x.to_str().ok().map(|s| s.to_string()))
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // 请求来源类型，便于统计是校园网内访问还是校园网外访问
    let origin_type = req
        .headers()
        .get("X-Origin-Type")
        .and_then(|x| x.to_str().ok().map(|s| s.to_string()))
        .unwrap_or("unknown".to_string());

    // 记录真实客户端 IP（优先 X-Forwarded-For）
    let remote_addr = req
        .headers()
        .get("X-Forwarded-For")
        .and_then(|x| x.to_str().ok().map(|s| s.to_string()))
        .unwrap_or(req.remote_addr().to_string());

    let stu_id =
        utils::jwt::auth(req).unwrap_or(String::from("unknown"));

    let user_agent = req
        .headers()
        .get("User-Agent")
        .and_then(|x| x.to_str().ok().map(|s| s.to_string()))
        .unwrap_or_default();

    let raw_path = req.uri().path().to_string();
    // 路由匹配后由 salvo matched-path 填入模板路径（如 auth-qrcode/status/{code}），
    // 未匹配时为空，统一记为 unmatched，避免原始 path 抬高 route 基数。
    let matched = req.matched_path();
    let route = if matched.is_empty() {
        "unmatched".to_string()
    } else {
        format!("/{matched}")
    };

    // tracing 要求我们需要把以后会设置的 span 属性提前设置好，进行占位
    // 关于 http_request span 的属性的一个原则是，所有的属性必须确保只在中间件中设置，
    // 因为在中间件中获取到的当前的 span 一定是 http_request span
    let span = tracing::span!(
        tracing::Level::INFO,
        // 面向 tracing 的 span name，tracing 要求该值必须是编译期就确定的
        "http_request",
        otel.kind = "server",
        // tracing opentelemetry 会将 otel.name 覆盖掉上面的 span name
        otel.name = %route,
        // - 当 panic 时，otel.status_code 由 catch_panic 中间件设置，otel.status_message 不会被设置
        // - 当没有发生 panic 且请求进入 router 层时，将由 error.rs 中渲染 RouterResult 时设置
        //   只要产生了 AppError，无论是否为服务器内部错误，span 的 status code 都会被设置为 error
        // - 当没有发生 panic 且请求没有进入 router 层时，将由 default 中间件设置
        otel.status_code = tracing::field::Empty,
        otel.status_message = tracing::field::Empty,
        // -------- 下面的是我们自己定义的 span 属性 --------
        event_type = "http_request",
        request_id = %request_id,
        // 标准 OTel HTTP 语义属性
        http.request.method = %req.method(),
        http.response.status_code = tracing::field::Empty,
        url.scheme = %req.uri().scheme_str().unwrap_or_default(),
        url.query = %req.uri().query().unwrap_or_default(),
        user_agent.original = %user_agent,
        client.address = %remote_addr,
        network.protocol.name = %format!("{:?}", req.version()),
        // path 为原始的，非规格化的路径
        url.path = %raw_path,
        // route 为规格化路径。如果没有被匹配已经注册的路由，则为 unmatched
        http.route = %route,
        // 响应的 http 状态码分类，见 status_class_str 函数
        http.response.status_class = tracing::field::Empty,
        // 如果当前请求没有携带 jwt，则 stu_id 为 unknown
        stu_id = %stu_id,
        // 是否发生了 panic
        panic = false,
        // cache 中间件的缓存命中情况，有如下几种取值
        // - hit: 命中缓存
        // - miss: 没有命中缓存
        // - tracing::field::Empty: 该请求没有被设置缓存
        cache_result = tracing::field::Empty,
        origin_type = %origin_type,
    );

    async move {
        // 将 request_id 回写到响应头，方便前端/调用方在出问题时提供
        if let Ok(v) = HeaderValue::from_str(&request_id) {
            res.headers_mut().insert("X-Request-Id", v);
        }

        ctrl.call_next(req, depot, res).await;

        let status =
            res.status_code.expect("status code should not be none");
        let status_code = status.as_u16();
        utils::record!(
            http.response.status_code = status_code,
            http.response.status_class = %status_class_str(&status),
        );
    }
    .instrument(span)
    .await
}

/// 状态码分类（2xx/3xx/4xx/5xx/other），便于按类聚合。
fn status_class_str(status: &StatusCode) -> &'static str {
    match status.as_u16() {
        200..=299 => "2xx",
        300..=399 => "3xx",
        400..=499 => "4xx",
        500..=599 => "5xx",
        _ => "other",
    }
}
