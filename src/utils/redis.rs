use lazy_static::lazy_static;
use redis::aio::MultiplexedConnection;

use crate::CFG;

// 定义Redis客户端
lazy_static! {
    pub static ref REDIS: redis::Client = {
        let url = format!("redis://:{}@{}/", CFG.redis.redis_password, CFG.redis.redis_url);
        redis::Client::open(url.as_str()).unwrap()
    };
}

pub async fn get_redis_conn() -> anyhow::Result<MultiplexedConnection> {
    REDIS
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| anyhow::anyhow!("Redis连接失败，请反馈给管理员"))
}
