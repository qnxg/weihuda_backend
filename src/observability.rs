use crate::config::CFG;
use base64::Engine;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::Protocol;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_otlp::WithHttpConfig;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::SdkTracerProvider;
use std::collections::HashMap;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const GREPTIME_TRACE_PIPELINE: &str = "greptime_trace_v1";

// 持有 TracerProvider，使其在整个进程生命周期存活；`shutdown` 时显式 flush 剩余 span。
static TRACER_PROVIDER: tokio::sync::OnceCell<SdkTracerProvider> =
    tokio::sync::OnceCell::const_new();

/// 初始化日志输出和 trace 上报
pub fn init() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_ansi(true)
        .with_writer(std::io::stdout)
        .with_target(false)
        .with_filter(tracing_subscriber::EnvFilter::new(
            &CFG.server.log_level,
        ));

    if let Some(endpoint) = CFG.observability.otlp_endpoint() {
        let provider = build_tracer_provider(endpoint);
        let tracer =
            provider.tracer(CFG.observability.service_name.clone());
        let _ = TRACER_PROVIDER.set(provider);

        // OTLP 固定日志等级为 debug
        let otel_layer = tracing_opentelemetry::layer()
            .with_tracer(tracer)
            .with_filter(tracing_subscriber::EnvFilter::new("debug"));

        tracing_subscriber::registry()
            .with(fmt_layer)
            .with(otel_layer)
            .init();
        tracing::info!(%endpoint, "OTLP tracing 已启用");
    } else {
        tracing_subscriber::registry().with(fmt_layer).init();
        tracing::info!(
            "未配置 observability.endpoint，跳过 OTLP 上报"
        );
    }
}

/// flush 并关闭 TracerProvider（导出缓冲区里剩余的 span）。在进程退出前调用。
pub fn shutdown() {
    if let Some(provider) = TRACER_PROVIDER.get() {
        let _ = provider.shutdown();
    }
}

fn build_tracer_provider(otlp_base: &str) -> SdkTracerProvider {
    // 配置里给的是 GreptimeDB 的 OTLP base（如 http://greptime:4000/v1/otlp）。
    // OTLP/HTTP 标准会在 base 后追加 /v1/traces；而 opentelemetry-otlp 的 .with_endpoint()
    // 取值原样使用（不自动追加），所以这里手动拼上 /v1/traces。
    let traces_endpoint =
        format!("{}/v1/traces", otlp_base.trim_end_matches('/'));
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_endpoint(&traces_endpoint)
        .with_protocol(Protocol::HttpBinary)
        .with_headers(greptime_headers())
        .build()
        .expect("构建 OTLP SpanExporter 失败");

    // 用带运行时的 BatchSpanProcessor，导出循环跑在 Tokio 上
    let processor = opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessor::builder(
        exporter,
        opentelemetry_sdk::runtime::Tokio,
    )
    .build();
    SdkTracerProvider::builder()
        .with_span_processor(processor)
        .with_resource(
            Resource::builder()
                .with_service_name(
                    CFG.observability.service_name.as_str(),
                )
                .build(),
        )
        .build()
}

/// GreptimeDB OTLP 请求头：固定带 trace pipeline 名；若配置了鉴权则附加 Basic Auth。
fn greptime_headers() -> HashMap<String, String> {
    let mut headers = HashMap::new();
    headers.insert(
        "x-greptime-pipeline-name".to_string(),
        GREPTIME_TRACE_PIPELINE.to_string(),
    );
    if let (Some(user), Some(pass)) =
        (&CFG.observability.username, &CFG.observability.password)
    {
        let cred = base64::engine::general_purpose::STANDARD
            .encode(format!("{user}:{pass}"));
        headers.insert(
            "Authorization".to_string(),
            format!("Basic {cred}"),
        );
    }
    headers
}
