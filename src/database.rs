use crate::config::CFG;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use std::time::Duration;
use tokio::sync::OnceCell;

pub static DB_POOL: OnceCell<MySqlPool> = OnceCell::const_new();

/// # Performance
/// 参见 [`sqlx::pool::Pool`] 文档：
/// > Cloning `Pool` is cheap as it is simply a
/// > reference-counted handle to the inner pool state.
///
/// 因此实际上没有必要将[`MySqlPool`]用[`std::sync::Arc`]等包裹。
/// 可以直接调用此函数获得全局数据库池。
///
/// # Side Effects
/// 数据库连接异常时，这个函数可能会结束进程。
pub async fn get_db_pool() -> MySqlPool {
    DB_POOL
        .get_or_init(|| async {
            match MySqlPoolOptions::new()
                .max_connections(CFG.database.max_connections)
                .acquire_timeout(Duration::from_secs(3))
                .connect(&CFG.database.database_url)
                .await
            {
                Ok(pool) => {
                    tracing::info!("🔥 Successfully connected to MySQL");
                    pool
                }
                Err(e) => {
                    tracing::error!("🪨 Failed to connect to MySQL: {:?}", e);
                    std::process::exit(1);
                }
            }
        })
        .await
        .clone()
}
