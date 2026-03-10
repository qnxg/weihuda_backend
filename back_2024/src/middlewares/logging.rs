use std::time::{Duration, Instant};

use reqwest::StatusCode;
use salvo::{
    Depot, FlowCtrl, Request, Response, handler,
    http::{ResBody, header::HeaderValue},
};
use tracing::Instrument;
use uuid::Uuid;

use crate::utils;

const SLOW_REQUEST_THRESHOLD: Duration = Duration::from_secs(3);

#[handler]
pub async fn logging_middleware(
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

    // 记录真实客户端 IP（优先 X-Forwarded-For）
    let remote_addr = req
        .headers()
        .get("X-Forwarded-For")
        .and_then(|x| x.to_str().ok().map(|s| s.to_string()))
        .unwrap_or(req.remote_addr().to_string());

    let stu_id =
        utils::jwt::auth(req).unwrap_or(String::from("unknown"));

    let span = tracing::span!(
        tracing::Level::INFO,
        "Request",
        request_id = %request_id,
        remote_addr = %remote_addr,
        method = %req.method(),
        path = %req.uri(),
        stu_id = %stu_id,
    );

    async move {
        // 将 request_id 回写到响应头，方便前端/调用方在出问题时提供
        if let Ok(v) = HeaderValue::from_str(&request_id) {
            res.headers_mut().insert("X-Request-Id", v);
        }

        let now = Instant::now();
        ctrl.call_next(req, depot, res).await;
        let duration = now.elapsed();

        let status = res.status_code.unwrap_or(match &res.body {
            ResBody::None => StatusCode::NOT_FOUND,
            ResBody::Error(e) => e.code,
            _ => StatusCode::OK,
        });

        if status != StatusCode::OK
            && status != StatusCode::NO_CONTENT
        {
            tracing::warn!(
                %status,
                ?duration,
                "Response"
            );
        } else if duration > SLOW_REQUEST_THRESHOLD {
            tracing::warn!(
                %status,
                ?duration,
                "Slow Request"
            );
        }
    }
    .instrument(span)
    .await
}
