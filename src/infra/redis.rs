use std::time::Duration;

use redis::{
    AsyncCommands, Client,
    aio::{ConnectionManager, ConnectionManagerConfig},
};
use tokio::sync::OnceCell;

use crate::{CFG, result::AppResult};

static REDIS_CONN_MGR: OnceCell<ConnectionManager> =
    OnceCell::const_new();

/// # Performance
///
/// 按照[`redis`]文档，异步Redis请求不需要连接池，可以使用多路复用。
/// > For async connections, connection pooling isn't necessary.
/// > The MultiplexedConnection is cloneable and can be used safely from multiple threads,
/// > so a single connection can be easily reused.
/// > For automatic reconnections consider using ConnectionManager with the connection-manager
/// > feature.
/// > Async cluster connections also don't require pooling and are thread-safe and reusable.
async fn get_conn() -> ConnectionManager {
    REDIS_CONN_MGR
        .get_or_init(|| async {
            ConnectionManager::new_with_config(
                Client::open(CFG.redis.redis_url.clone())
                    .expect("获取 redis 配置信息失败"),
                ConnectionManagerConfig::new()
                    // 设置超时是重要的，避免超时中间件触发后任务仍在进行
                    .set_connection_timeout(Duration::from_secs(3))
                    // 设置超时是重要的，避免超时中间件触发后任务仍在进行
                    .set_response_timeout(Duration::from_secs(3)),
            )
            .await
            .expect("创建 redis 连接失败")
        })
        .await
        .clone()
}

/// 清除与指定stu_id相关的Redis缓存
pub async fn clear_stuid_cache(stu_id: &str) -> AppResult<()> {
    let mut conn = get_conn().await;
    let keys: Vec<String> =
        conn.keys(format!("*{}*", stu_id)).await?;
    for key in keys {
        let _: () = conn.del(key).await?;
    }
    Ok(())
}

/// 设置键值对
#[expect(unused)]
pub async fn set(key: &str, value: &str) -> AppResult<()> {
    let mut conn = get_conn().await;
    let _: () = conn.set(key, value).await?;
    Ok(())
}

/// 设置带过期时间的键值对，过期事件的单位为秒
pub async fn set_with_expire(
    key: &str,
    value: &str,
    expire_secs: u64,
) -> AppResult<()> {
    let mut conn = get_conn().await;
    let _: () = conn.set_ex(key, value, expire_secs).await?;
    Ok(())
}

/// 获取某个 key 的 value
pub async fn get(key: &str) -> AppResult<Option<String>> {
    let mut conn = get_conn().await;
    let res: Option<String> = conn.get(key).await?;
    Ok(res)
}

/// 删除某个 key
pub async fn del(key: &str) -> AppResult<()> {
    let mut conn = get_conn().await;
    let _: () = conn.del(key).await?;
    Ok(())
}
