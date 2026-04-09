#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![deny(rustdoc::all)]
#![cfg_attr(not(test), warn(clippy::unwrap_used))]
#![warn(clippy::allow_attributes)]
#![warn(clippy::too_many_lines)]
#![warn(clippy::too_long_first_doc_paragraph)]
#![warn(
    clippy::todo,
    reason = "在`git commit`之前，请确认代码中没有`todo!()`"
)]

mod config;
mod infra;
mod middlewares;
mod result;
mod routers;
mod service;
mod utils;

#[cfg(test)]
mod test;

use crate::{
    config::CFG,
    middlewares::{
        cache::cache_middleware, catch_panic::catch_panic_middleware,
        cors::cors_middleware, default::default_middleware,
        logging::logging_middleware,
        prometheus::prometheus_middleware,
        timeout::timeout_middleware,
    },
};
use salvo::Service;
use salvo::prelude::*;

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    // Config the tracing logge
    // 专用于Linux服务器系统
    // Windows上跑无法获取正确TimeZone，不会报错，但日志记录时间为Utc，慢8小时
    let _guard = clia_tracing_config::build()
        .filter_level(&CFG.log.filter_level)
        .with_ansi(CFG.log.with_ansi)
        .to_stdout(CFG.log.to_stdout)
        .directory(&CFG.log.directory)
        .file_name(&CFG.log.file_name)
        .rolling(&CFG.log.rolling)
        .with_source_location(false) // 在调试时候可以打开，确认日志所处的代码位置
        .with_thread_ids(false) // 无需打开，线程模型有tokio调度
        .with_thread_names(false) // 无需打开，线程模型有tokio调度
        .with_target(false) // 无需打开，打开后日志很累赘
        .format(&CFG.log.format)
        .init();

    // Mark the log level
    tracing::info!("📓 Log level: {}", &CFG.log.filter_level);
    tracing::info!("🚀 Starting Ca Task Worker");
    service::grade_rank::ca::start_ca_task_worker().await;
    tracing::info!("🚀 Server {} is starting", &CFG.server.name);
    tracing::info!("🔄 Listening on port: {}", &CFG.server.address);
    let listener = TcpListener::new(&CFG.server.address).bind().await;
    let routers = routers::routers();
    let service = Service::new(routers)
        .hoop(catch_panic_middleware)
        .hoop(default_middleware)
        .hoop(logging_middleware)
        .hoop(prometheus_middleware)
        .hoop(cors_middleware())
        .hoop(cache_middleware)
        .hoop(timeout_middleware);
    Server::new(listener).serve(service).await;
}
