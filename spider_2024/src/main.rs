#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![deny(rustdoc::all)]
#![warn(clippy::allow_attributes)]
#![warn(clippy::too_many_lines)]
#![warn(clippy::too_long_first_doc_paragraph)]
#![warn(
    clippy::todo,
    reason = "在`git commit`之前，请确认代码中没有`todo!()`"
)]

use crate::router::create_router;
use config::CFG;
use log::info;
use salvo::prelude::*;

mod app_error;
mod app_result;
mod config;
mod dtos;
mod handlers;
mod middlewares;
mod router;
mod spiders;
mod utils;

#[tokio::main]
async fn main() {
    let _guard = clia_tracing_config::build()
        .filter_level(&CFG.log.filter_level)
        .with_ansi(CFG.log.with_ansi)
        .to_stdout(CFG.log.to_stdout)
        .with_source_location(false)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .directory(&CFG.log.directory)
        .file_name(&CFG.log.file_name)
        .rolling(&CFG.log.rolling)
        .init();
    info!("Starting server: {}", CFG.server.name);
    let acceptor = TcpListener::new(&CFG.server.address).bind().await;
    let server = Server::new(acceptor);
    #[expect(unused_variables)] // 防止开发时候报WARN
    let handle = server.handle();
    // 初始化路由
    let router = create_router();
    // 优雅关机
    // 生产环境再启用
    // #[cfg(not(debug_assertions))] // 这个代表非debug模式
    // tokio::spawn(async move {
    //     shutdown_signal().await;
    //     handle.stop_graceful(None);
    // });
    server.serve(router).await;
}

#[expect(dead_code)]
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(
            tokio::signal::unix::SignalKind::terminate(),
        )
        .expect("failed to install signal handler")
        .recv()
        .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
