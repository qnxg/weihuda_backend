use redis::aio::ConnectionManager;
use tokio::sync::OnceCell;

use crate::{
    config::CFG,
    error::{AppResult, ThrowInternalError},
};

static REDIS_CONN: OnceCell<ConnectionManager> =
    OnceCell::const_new();

pub async fn redis_connection() -> AppResult<ConnectionManager> {
    let conn = REDIS_CONN
        .get_or_try_init(|| async {
            let client =
                redis::Client::open(CFG.redis.redis_url.clone())
                    .map_err(|e| {
                        e.internal_err().with("redis 连接失败")
                    })?;
            ConnectionManager::new(client)
                .await
                .map_err(|e| e.internal_err().with("redis 连接失败"))
        })
        .await?;
    Ok(conn.clone())
}
