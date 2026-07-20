mod async_update;
mod redis;

use crate::{
    error::{AppError, AppResult, ThrowInternalErrorResult},
    infra::cache::redis::redis_connection,
    utils::{self, single_flight::SingleFlight},
};
use ::redis::AsyncCommands;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    any::Any,
    fmt::Debug,
    sync::{Arc, LazyLock},
    time::Duration,
};

pub use async_update::{
    CacheAsyncUpdateResult, start_async_update_worker,
    with_cache_async_update,
};

static SINGLE_FLIGHT: LazyLock<
    SingleFlight<AppResult<Arc<dyn Any + Send + Sync>>>,
> = LazyLock::new(SingleFlight::new);

pub struct CacheStrategy {
    key: String,
    ttl: Duration,
}

impl CacheStrategy {
    /// # Arguments
    ///
    /// - `key`: 缓存 key
    /// - `ttl`: 缓存时间
    pub fn new(key: String, ttl: Duration) -> Self {
        Self { key, ttl }
    }
}

pub trait CacheKey: Debug {
    const PREFIX: &'static str;
    const VERSION: u64;
    type Value: for<'a> Deserialize<'a>
        + Serialize
        + Clone
        + Send
        + Sync
        + 'static;
    fn strategy(&self) -> CacheStrategy;
}

/// 在基准 TTL 上叠加最多约 10% 的随机抖动，打散同一时刻的批量过期。
fn random_ttl(duration: Duration) -> u64 {
    let base = duration.as_secs();
    let jitter = rand::thread_rng().gen_range(0..=(base / 10));
    base + jitter
}

#[tracing::instrument(
    skip(f),
    fields(
        otel.kind = "internal",
        event_type = "cache",
        prefix = tracing::field::Empty,
        version = tracing::field::Empty,
        strategy_key = tracing::field::Empty,
        // 如果为 true，表明是这一批次获取同 key 缓存任务中负责去真正获取缓存的任务
        leader = false,
        // leader 为 true 时，该值表示 redis 是否故障导致缓存绕过了 redis
        redis_failed = false,
        // leader 为 true 时且 redis_failed 为 false 时，该值表示缓存是否命中
        cached = tracing::field::Empty,
        // cached 为 false 时，该值表示是否成功地向 reids 更新了缓存
        updated = tracing::field::Empty,
    ),
    err
)]
pub async fn with_cache<K: CacheKey>(
    key: K,
    f: impl AsyncFnOnce() -> AppResult<K::Value>,
) -> AppResult<K::Value> {
    let strategy = key.strategy();
    utils::record!(
        prefix = K::PREFIX,
        version = K::VERSION,
        strategy_key = strategy.key
    );
    let redis_key =
        format!("{}:{}:{}", K::PREFIX, K::VERSION, strategy.key);
    let result = SINGLE_FLIGHT
        .call(&redis_key, async || {
            utils::record!(leader = true);
            let res = match get_cache_inner(&redis_key).await {
                Ok(Some(value)) => {
                    utils::record!(cached = true);
                    value
                }
                Ok(None) => {
                    utils::record!(cached = false);
                    let value = f().await?;
                    let update_result = update_cache_inner(
                        &redis_key,
                        value.clone(),
                        strategy.ttl,
                    )
                    .await;
                    utils::record!(updated = update_result.is_ok());
                    value
                }
                Err(_) => {
                    utils::record!(redis_failed = true);

                    f().await?
                }
            };
            Ok::<_, AppError>(
                Arc::new(res) as Arc<dyn Any + Send + Sync>
            )
        })
        .await?;
    let res = result
        .downcast_ref::<K::Value>()
        .cloned()
        .expect("缓存 singleflight 类型转换失败");
    Ok(res)
}

#[tracing::instrument(
    fields(
        otel.kind = "internal",
        event_type = "cache",
        prefix = tracing::field::Empty,
        version = tracing::field::Empty,
        strategy_key = tracing::field::Empty,
    ),
    err
)]
/// 删除掉某个缓存
pub async fn invalidate_cache<K: CacheKey>(key: K) -> AppResult<()> {
    let strategy = key.strategy();
    let redis_key =
        format!("{}:{}:{}", K::PREFIX, K::VERSION, strategy.key);
    utils::record!(
        prefix = K::PREFIX,
        version = K::VERSION,
        strategy_key = strategy.key
    );
    let mut conn = redis_connection().await?;
    let _: () = conn.del(&redis_key).await.internal_err()?;
    Ok(())
}

async fn update_cache_inner<T: Serialize>(
    key: &str,
    value: T,
    ttl: Duration,
) -> AppResult<()> {
    let mut conn = redis_connection().await?;
    let json_str = serde_json::to_string(&value).internal_err()?;
    let _: () = conn
        .set_ex(key, &json_str, random_ttl(ttl))
        .await
        .internal_err()?;
    Ok(())
}

async fn get_cache_inner<T: for<'a> Deserialize<'a>>(
    key: &str,
) -> AppResult<Option<T>> {
    let mut conn = redis_connection().await?;
    let Some(json_str) =
        conn.get::<&str, Option<String>>(key).await.internal_err()?
    else {
        return Ok(None);
    };
    let value = serde_json::from_str::<T>(&json_str).map_err(|e| {
        tracing::error!(error = ?e, json_str = %json_str, "反序列化缓存值失败");
        e
    }).ok();
    Ok(value)
}

#[tracing::instrument(
    skip(value),
    fields(
        otel.kind = "internal",
        event_type = "cache",
        prefix = tracing::field::Empty,
        version = tracing::field::Empty,
        strategy_key = tracing::field::Empty,
    ),
    err
)]
pub async fn update_cache<K: CacheKey>(
    key: K,
    value: K::Value,
) -> AppResult<()> {
    let strategy = key.strategy();
    let redis_key =
        format!("{}:{}:{}", K::PREFIX, K::VERSION, strategy.key);
    utils::record!(
        prefix = K::PREFIX,
        version = K::VERSION,
        strategy_key = strategy.key
    );
    update_cache_inner(&redis_key, value, strategy.ttl).await
}

#[tracing::instrument(
    fields(
        otel.kind = "internal",
        event_type = "cache",
        prefix = tracing::field::Empty,
        version = tracing::field::Empty,
        strategy_key = tracing::field::Empty,
    ),
    err
)]
pub async fn get_cache<K: CacheKey>(
    key: K,
) -> AppResult<Option<K::Value>> {
    let strategy = key.strategy();
    let redis_key =
        format!("{}:{}:{}", K::PREFIX, K::VERSION, strategy.key);
    utils::record!(
        prefix = K::PREFIX,
        version = K::VERSION,
        strategy_key = strategy.key
    );
    get_cache_inner(&redis_key).await
}
