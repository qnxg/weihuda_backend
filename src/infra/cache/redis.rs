use redis::{Client, aio::MultiplexedConnection};
use tokio::sync::OnceCell;

use crate::{
    config::CFG,
    error::{AppResult, ThrowInternalError},
};

static REDIS_CLIENT: OnceCell<Client> = OnceCell::const_new();

async fn get_redis_client() -> &'static Client {
    REDIS_CLIENT
        .get_or_init(|| async {
            redis::Client::open(CFG.redis.redis_url.clone())
                .expect("redis 连接失败")
        })
        .await
}

pub async fn redis_connection() -> AppResult<MultiplexedConnection> {
    let client = get_redis_client().await;
    let connection = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| e.internal_err().with("redis 连接失败"))?;
    Ok(connection)
}
