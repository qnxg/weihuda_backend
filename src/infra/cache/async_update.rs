use redis::AsyncCommands;
use tracing::Instrument;

use crate::{
    error::{AppError, AppResult, ThrowInternalErrorResult},
    infra::cache::{
        CacheKey, random_ttl, redis::redis_connection, with_cache,
    },
    utils::{self, task_queue::UniqueTaskQueue},
};
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, LazyLock, Mutex},
    time::Instant,
};

type AsyncUpdateTaskF =
    Pin<Box<dyn Future<Output = AppResult<()>> + Send>>;

struct AsyncUpdateTask {
    context: Option<(String, String)>,
    start_time: Instant,
    f: AsyncUpdateTaskF,
}

static ASYNC_UPDATE_QUEUE: LazyLock<
    UniqueTaskQueue<String, AsyncUpdateTask>,
> = LazyLock::new(UniqueTaskQueue::new);

const ASYNC_UPDATE_WORKER_COUNT: usize = 5;

pub enum CacheAsyncUpdateResult<T> {
    /// 获取新的数据成功，用 T 来更新缓存
    Ok(T),
    /// 获取数据失败，移除掉旧的缓存
    Err(AppError),
    /// 获取数据失败，但是保留旧的缓存
    #[expect(unused)]
    Keep(AppError),
    /// 获取数据失败，延长缓存时间，即重新设置缓存在 ttl 后失效
    Extend(AppError),
}

/// 使用异步更新的方式获取缓存
///
/// 在 [with_cache] 的基础上，还有当缓存命中时，会将 `f` 放入一个队列中，异步更新缓存。
///
/// 一般对于更新频率不高的数据可以使用这个函数，配合设置较大的 TTL，可以做到缓存次次命中，同时
/// 缓存也是较新的（只会有少许延迟，对于更新频率不高的场景下可以接受）
#[tracing::instrument(
    skip(f),
    fields(
        otel.kind = "internal",
        event_type = "cache",
        prefix = tracing::field::Empty,
        version = tracing::field::Empty,
        strategy_key = tracing::field::Empty,
        try_push = false
    ),
    err
)]
#[expect(clippy::too_many_lines)]
pub async fn with_cache_async_update<K, F, Fut>(
    key: K,
    f: F,
) -> AppResult<K::Value>
where
    K: CacheKey,
    F: FnOnce() -> Fut + Send,
    Fut: Future<Output = CacheAsyncUpdateResult<K::Value>>
        + Send
        + 'static,
{
    let strategy = key.strategy();
    utils::record!(
        prefix = K::PREFIX,
        version = K::VERSION,
        strategy_key = strategy.key
    );
    let redis_key =
        format!("{}:{}:{}", K::PREFIX, K::VERSION, strategy.key);
    let ttl = strategy.ttl;

    // with_cache 在命中时不会调用闭包，但仍会吃掉 f；用槽位在「未调用」时取回 f 做异步更新
    let f_slot = Arc::new(Mutex::new(Some(f)));
    let f_for_miss = f_slot.clone();

    let value = with_cache(key, async move || {
        let f = f_for_miss
            .lock()
            .expect("mutex 中毒")
            .take()
            .expect("cache miss 回调 f 已被取走");
        match f().await {
            CacheAsyncUpdateResult::Ok(v) => Ok(v),
            CacheAsyncUpdateResult::Err(app_error) => Err(app_error),
            CacheAsyncUpdateResult::Keep(app_error) => Err(app_error),
            CacheAsyncUpdateResult::Extend(app_error) => {
                Err(app_error)
            }
        }
    })
    .await?;

    if let Some(f) = f_slot.lock().expect("mutex 中毒").take() {
        utils::record!(try_push = true);
        let context = utils::tracing::current_trace_context();
        let span = tracing::info_span!(
            "push_to_async_update_queue",
            prefix = K::PREFIX,
            version = K::VERSION,
            strategy_key = strategy.key,
            already_in_queue = tracing::field::Empty,
        );
        let f_future = f();
        tokio::spawn(
            async move {
                let redis_key2 = redis_key.clone();
                let task_f = Box::pin(async move {
                    let mut conn = redis_connection().await?;
                    match f_future.await {
                        CacheAsyncUpdateResult::Ok(v) => {
                            utils::record!(outcome = "ok");
                            let json_str = serde_json::to_string(&v)
                                .internal_err()?;
                            let _: () = conn
                                .set_ex(
                                    redis_key,
                                    json_str,
                                    random_ttl(ttl),
                                )
                                .await
                                .internal_err()?;
                            Ok(())
                        }
                        CacheAsyncUpdateResult::Err(e) => {
                            utils::record!(outcome = "err");
                            let _: () = conn
                                .del(redis_key)
                                .await
                                .internal_err()?;
                            Err(e)
                        }
                        CacheAsyncUpdateResult::Keep(e) => {
                            utils::record!(outcome = "keep");
                            Err(e)
                        }
                        CacheAsyncUpdateResult::Extend(e) => {
                            utils::record!(outcome = "extend");
                            let _: () = conn
                                .expire(
                                    redis_key,
                                    random_ttl(ttl) as i64,
                                )
                                .await
                                .internal_err()?;
                            Err(e)
                        }
                    }
                });
                let task = AsyncUpdateTask {
                    context,
                    start_time: Instant::now(),
                    f: task_f,
                };
                let _ = ASYNC_UPDATE_QUEUE
                    .push(
                        redis_key2,
                        task,
                        async || {
                            utils::record!(already_in_queue = true);
                        },
                        async || {
                            utils::record!(already_in_queue = false);
                            Ok(())
                        },
                    )
                    .await;
            }
            .instrument(span),
        );
    }

    Ok(value)
}

pub async fn start_async_update_worker() {
    for worker_id in 0..ASYNC_UPDATE_WORKER_COUNT {
        tokio::spawn(async_update_worker(worker_id));
    }
}

async fn async_update_worker(worker_id: usize) {
    loop {
        let (
            key,
            AsyncUpdateTask {
                context,
                start_time,
                f,
            },
        ) = ASYNC_UPDATE_QUEUE.pop().await;
        let wait_time = start_time.elapsed();
        let span = tracing::info_span!(
            "async_update_worker",
            otel.kind = "internal",
            event_type = "cache",
            worker_id = worker_id,
            originating_trace_id =
                %context.as_ref().map(|(trace_id, _)| trace_id.clone()).unwrap_or_default(),
            originating_span_id =
                %context.as_ref().map(|(_, span_id)| span_id.clone()).unwrap_or_default(),
            otel.status_code = tracing::field::Empty,
            otel.status_description = tracing::field::Empty,
            // 任务排队时间，单位：毫秒
            wait_time = wait_time.as_millis(),
            // 任务执行结果，有如下几种取值：
            // - ok：任务执行成功
            // - err: 任务执行失败，同时移除旧的缓存
            // - keep: 任务执行失败，但保留旧的缓存
            // - extend: 任务执行失败，延长缓存时间
            outcome = tracing::field::Empty,
        );
        if let Err(e) = f.instrument(span.clone()).await {
            span.record("otel.status_code", "error");
            span.record("otel.status_description", format!("{e}"));
        }
        ASYNC_UPDATE_QUEUE.ack(key).await;
    }
}
