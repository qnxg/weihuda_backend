mod app_error;
mod app_result;
mod config;
mod database;
mod dtos;
mod entities;
mod extractors;
mod handlers;
mod middlewares;
mod routers;
mod utils;

use crate::{config::CFG, routers::create_router};
use database::get_db_pool;
use sqlx::mysql::MySqlPool;
use std::sync::Arc;
use tokio::signal;

pub struct DbPool {
    db: MySqlPool,
}

#[tokio::main]
async fn main() {
    // Config the tracing logger，专用于Linux服务器系统，Windows上跑无法获取正确TimeZone，不会报错，但日志记录时间为Utc，慢8小时
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
        .init();

    // Mark the log level
    tracing::info!("📓 Log level: {}", &CFG.log.filter_level);

    // Connect to MySQL
    let pool = get_db_pool().await;

    // Build the final router combined with middleware layers
    let app = create_router(Arc::new(DbPool { db: pool.clone() })); // 将AppState用原子化引用计数包装，使其可以在多个线程中共享
                                                                    // Start the server
    tracing::info!("🚀 Server {} is starting", &CFG.server.name);
    tracing::info!("🔄 Listening on port: {}", &CFG.server.address);
    let listener = tokio::net::TcpListener::bind(&CFG.server.address).await.unwrap();

    // Serve the server
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
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
