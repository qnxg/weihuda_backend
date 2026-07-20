#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![deny(rustdoc::all)]
// async 埋点（tracing .instrument() 包裹）+ 多层 service 调用让 handler 的 future 类型较深，
// 默认 128 会 overflow，提到 512 给编译器留余量。
#![recursion_limit = "512"]
#![cfg_attr(not(test), warn(clippy::unwrap_used))]
#![warn(clippy::allow_attributes)]
#![warn(clippy::too_many_lines)]
#![warn(clippy::too_long_first_doc_paragraph)]
#![warn(
    clippy::todo,
    reason = "在`git commit`之前，请确认代码中没有`todo!()`"
)]

mod config;
mod error;
mod infra;
mod middlewares;
mod observability;
mod routers;
mod service;
mod utils;

#[cfg(test)]
mod test;

use crate::{
    config::CFG,
    middlewares::{
        catch_panic::catch_panic_middleware, cors::cors_middleware,
        default::default_middleware, timeout::timeout_middleware,
        tracing::tracing_middleware,
    },
};
use salvo::prelude::*;
use salvo::{Service, server::ServerHandle};
use std::panic::PanicHookInfo;

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    observability::init();
    std::panic::set_hook(Box::new(panic_hook));

    tracing::info!("📓 Log level: {}", &CFG.server.log_level);
    tracing::info!("🚀 Starting Ca Task Worker");
    service::grade_rank::ca::start_ca_task_worker().await;
    infra::cache::start_async_update_worker().await;
    tracing::info!("🚀 Server {} is starting", &CFG.server.name);
    tracing::info!("🔄 Listening on port: {}", &CFG.server.address);
    let listener = TcpListener::new(&CFG.server.address).bind().await;
    let routers = routers::routers();
    let service = Service::new(routers)
        // tracing 一定要放在最外层来确保日志能够被记录
        .hoop(tracing_middleware)
        .hoop(catch_panic_middleware)
        .hoop(default_middleware)
        .hoop(cors_middleware())
        .hoop(timeout_middleware);
    let server = Server::new(listener);
    let handle = server.handle();

    // 优雅退出：收到 ctrl_c / SIGTERM 时 flush 可观测性 OTLP 数据，
    // 同时停止接收新的请求，等待已经接受的请求处理完毕后再退出
    tokio::spawn(shutdown_signal_handler(handle));

    server.serve(service).await;
}

fn panic_hook(info: &PanicHookInfo) {
    let msg = info
        .payload()
        .downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| info.payload().downcast_ref::<String>().cloned())
        .unwrap_or(format!(
            "Unknown panic, type_id: {:?}",
            info.payload().type_id()
        ));
    let location = info
        .location()
        .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()));
    // 产生一个 error 等级且不含 message 的 span event，tracing opentelemetry 会自动设置 span 的 status_code 和 status_description
    tracing::error!(error = %format!("thread panicked: {}", msg), ?location);
}

async fn shutdown_signal_handler(handle: ServerHandle) {
    // Wait Shutdown Signal
    let ctrl_c = async {
        // Handle Ctrl+C signal
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        // Handle SIGTERM on Unix systems
        tokio::signal::unix::signal(
            tokio::signal::unix::SignalKind::terminate(),
        )
        .expect("failed to install signal handler")
        .recv()
        .await;
    };

    #[cfg(windows)]
    let terminate = async {
        // Handle Ctrl+C on Windows (alternative implementation)
        tokio::signal::windows::ctrl_c()
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => tracing::info!("ctrl_c signal received"),
        _ = terminate => tracing::info!("terminate signal received"),
    };

    handle.stop_graceful(None);
    observability::shutdown();
    std::process::exit(0);
}
