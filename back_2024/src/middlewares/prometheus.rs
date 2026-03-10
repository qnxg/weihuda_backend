use std::time::Instant;

use prometheus::{
    CounterVec, Encoder, HistogramOpts, HistogramVec, TextEncoder,
    register_counter_vec, register_histogram_vec,
};
use salvo::{Depot, FlowCtrl, Request, Response, handler};
use tokio::sync::OnceCell;

const BUCKETS: &[f64] =
    &[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0];

static METRICS: OnceCell<(CounterVec, HistogramVec)> =
    OnceCell::const_new();

async fn metrics() -> &'static (CounterVec, HistogramVec) {
    METRICS
        .get_or_init(|| async {
            let counter = register_counter_vec!(
                "http_requests_total",
                "Total HTTP requests",
                &["method", "path", "status"]
            )
            .expect("register http_requests_total");
            let histogram = register_histogram_vec!(
                HistogramOpts::new(
                    "http_request_duration_seconds",
                    "HTTP request duration in seconds"
                )
                .buckets(BUCKETS.to_vec()),
                &["method", "path"]
            )
            .expect("register http_request_duration_seconds");
            (counter, histogram)
        })
        .await
}

/// Normalize path: replace numeric or UUID-like segments with ":id" to limit label cardinality.
fn normalize_path(path: &str) -> String {
    let segments: Vec<&str> =
        path.trim_matches('/').split('/').collect();
    let normalized: Vec<String> = segments
        .iter()
        .map(|seg| {
            if seg.is_empty() {
                return String::new();
            }
            if seg.chars().all(|c| c.is_ascii_digit()) {
                return ":id".to_string();
            }
            if seg.len() >= 8
                && seg.len() <= 36
                && seg
                    .chars()
                    .all(|c| c.is_ascii_hexdigit() || c == '-')
            {
                return ":id".to_string();
            }
            (*seg).to_string()
        })
        .collect();
    "/".to_string() + &normalized.join("/")
}

#[handler]
pub async fn prometheus_middleware(
    req: &mut Request,
    resp: &mut Response,
    ctrl: &mut FlowCtrl,
    depot: &mut Depot,
) {
    let path = req.uri().path().to_string();
    if path == "/metrics" {
        ctrl.call_next(req, depot, resp).await;
        return;
    }

    let method = req.method().as_str().to_string();
    let path_normalized = normalize_path(&path);
    let start = Instant::now();

    ctrl.call_next(req, depot, resp).await;

    let elapsed = start.elapsed();
    let (counter, histogram) = metrics().await;
    let status = resp
        .status_code
        .map(|c| c.as_u16().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    histogram
        .with_label_values(&[&method, &path_normalized])
        .observe(elapsed.as_secs_f64());
    counter
        .with_label_values(&[
            method.as_str(),
            path_normalized.as_str(),
            status.as_str(),
        ])
        .inc();
}

/// Encode default registry as Prometheus text format and write to response.
pub async fn render_metrics(res: &mut Response) {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::with_capacity(4096);
    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        tracing::error!(error = ?e, "prometheus encode error");
        res.status_code(
            salvo::http::StatusCode::INTERNAL_SERVER_ERROR,
        );
        return;
    }
    let body = match String::from_utf8(buffer) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(error = ?e, "prometheus metrics utf8 error");
            res.status_code(
                salvo::http::StatusCode::INTERNAL_SERVER_ERROR,
            );
            return;
        }
    };
    res.headers_mut().insert(
        salvo::http::header::CONTENT_TYPE,
        "text/plain; version=0.0.4; charset=utf-8"
            .parse()
            .expect("content-type"),
    );
    res.status_code(salvo::http::StatusCode::OK);
    res.render(body);
}
