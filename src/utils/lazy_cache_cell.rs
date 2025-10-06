use tokio::{
    runtime::Handle,
    sync::{RwLock, RwLockReadGuard},
    task::block_in_place,
};

use std::{
    future::Future,
    pin::Pin,
    sync::LazyLock,
    time::{Duration, Instant},
};

struct Cache<T, U> {
    last_updated: Instant,
    lifetime: Duration,
    inner: Option<T>,
    updater: U,
}

type BoxedAsyncFnMut<T> = Box<
    dyn Send + Sync + FnMut() -> Box<dyn Send + Future<Output = T>>,
>;

/// 按照一定时间间隔过期数据，过期后调用函数重新获取
pub struct CacheCell<T>(RwLock<Cache<T, BoxedAsyncFnMut<T>>>);

impl<T> CacheCell<T> {
    pub fn new(
        lifetime: Duration,
        updater: BoxedAsyncFnMut<T>,
    ) -> Self {
        let last_updated = Instant::now();
        Self(RwLock::new(Cache {
            last_updated,
            lifetime,
            inner: None,
            updater,
        }))
    }

    /// # Panics
    /// 当未在多线程模式的tokio runtime中调用会panic
    /// （[tokio::main]默认为多线程，[tokio::test]默认为单线程）
    ///
    /// 见 [block_in_place]
    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        let now = Instant::now();
        let need_upgrade = {
            let cache = block_in_place(|| self.0.blocking_read());
            cache.inner.is_none()
                || now - cache.last_updated > cache.lifetime
        };
        if need_upgrade {
            block_in_place(|| {
                let mut cache = self.0.blocking_write();
                let res = Handle::current()
                    .block_on(Pin::from((cache.updater)()));
                cache.inner = Some(res);
                cache.last_updated = now;
            });
        }
        RwLockReadGuard::map(
            block_in_place(|| self.0.blocking_read()),
            |me| {
                // 之前已有is_none判断，此处必不为None
                me.inner.as_ref().expect("impossible")
            },
        )
    }
}

pub type LazyCacheCell<T> = LazyLock<CacheCell<T>>;

/// 省略掉Box和async block等
///
/// 无法简化为函数，因为目前Rust无法写出闭包完整类型
macro_rules! lazy_cache_cell {
    ($lifetime:expr, $async_fn:expr) => {
        std::sync::LazyLock::new(move || {
            crate::utils::lazy_cache_cell::CacheCell::new(
                $lifetime,
                Box::new(move || Box::new(($async_fn)())),
            )
        })
    };
}
pub(crate) use lazy_cache_cell;

#[cfg(test)]
mod test {
    use super::*;
    use tokio::time::sleep;

    static STAMP: LazyCacheCell<i64> =
        lazy_cache_cell!(Duration::from_millis(500), stamp);

    async fn stamp() -> i64 {
        chrono::Utc::now().timestamp_millis()
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cache_cell_static() {
        let res1 = { *STAMP.read() };
        assert_eq!(*(STAMP.read()), res1);
        assert_eq!(*(STAMP.read()), res1);
        sleep(Duration::from_millis(100)).await;
        assert_eq!(*(STAMP.read()), res1);
        sleep(Duration::from_millis(500)).await;
        let res2 = { *STAMP.read() };
        assert_ne!(res1, res2);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cache_cell_closure() {
        let mut i = 0;
        let c = lazy_cache_cell!(Duration::from_millis(500), || {
            i += 1;
            async move { i }
        });
        assert_eq!(*(c.read()), 1);
        assert_eq!(*(c.read()), 1);
        sleep(Duration::from_millis(100)).await;
        assert_eq!(*(c.read()), 1);
        sleep(Duration::from_millis(500)).await;
        assert_eq!(*(c.read()), 2);
        sleep(Duration::from_millis(100)).await;
        assert_eq!(*(c.read()), 2);
        sleep(Duration::from_millis(500)).await;
        assert_eq!(*(c.read()), 3);
        assert_eq!(*(c.read()), 3);
    }
}
