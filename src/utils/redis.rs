use lazy_static::lazy_static;
use redis::{aio::MultiplexedConnection, AsyncCommands};

use crate::CFG;

// 定义Redis客户端
lazy_static! {
    pub static ref REDIS: redis::Client = {
        let url = format!(
            "redis://:{}@{}/",
            CFG.redis.redis_password, CFG.redis.redis_url
        );
        redis::Client::open(url.as_str()).unwrap()
    };
}

pub async fn get_redis_conn() -> anyhow::Result<MultiplexedConnection>
{
    REDIS
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| anyhow::anyhow!("Redis连接失败，请反馈给管理员"))
}

/// 清除与指定stu_id相关的Redis缓存
pub async fn clear_redis_cache(
    stu_id: &str,
) -> Result<(), anyhow::Error> {
    let mut con = get_redis_conn().await?;
    let keys: Vec<String> = con.keys(format!("*{}*", stu_id)).await?;
    for key in keys {
        let _: () = con.del(key).await?;
    }
    Ok(())
}
