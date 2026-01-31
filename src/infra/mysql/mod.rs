pub mod announcement;
pub mod config;
pub mod course;
pub mod exam_num;
pub mod feedback;
pub mod flex_time;
pub mod jifen;
pub mod kv_cache;
pub mod left_message;
pub mod notice;
pub mod semester;
pub mod user;
pub mod user_setting;
pub mod zhihu;

use crate::config::CFG;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use std::time::Duration;
use tokio::sync::OnceCell;

static DB_POOL: OnceCell<MySqlPool> = OnceCell::const_new();

/// # Side Effects
/// 数据库连接异常时，这个函数可能会结束进程。
async fn get_db_pool() -> &'static MySqlPool {
    DB_POOL
        .get_or_init(|| async {
            match MySqlPoolOptions::new()
                .max_connections(CFG.database.max_connections)
                .acquire_timeout(Duration::from_secs(3))
                .connect(&CFG.database.database_url)
                .await
            {
                Ok(pool) => {
                    tracing::info!(
                        "🔥 Successfully connected to MySQL"
                    );
                    pool
                }
                Err(e) => {
                    tracing::error!(
                        "🪨 Failed to connect to MySQL: {:?}",
                        e
                    );
                    std::process::exit(1);
                }
            }
        })
        .await
}
