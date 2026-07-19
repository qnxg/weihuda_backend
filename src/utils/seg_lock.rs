use dashmap::DashMap;
use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};
use tokio::sync::OwnedMutexGuard;

/// 一个简单的分段锁实现
///
/// 推荐使用锁粒度更小的 [NewSegLock] 实现
#[deprecated(note = "请改用 `NewSegLock`")]
// 业务已切换到 NewSegLock，仅在基准测试时使用
#[expect(dead_code)]
pub struct SegLock<const N: usize> {
    locks: [tokio::sync::Mutex<()>; N],
}

#[expect(deprecated)]
impl<const N: usize> Default for SegLock<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[expect(deprecated, dead_code)]
impl<const N: usize> SegLock<N> {
    pub fn new() -> Self {
        Self {
            locks: std::array::from_fn(|_| {
                tokio::sync::Mutex::new(())
            }),
        }
    }
    pub async fn lock(
        &self,
        key: &str,
    ) -> tokio::sync::MutexGuard<'_, ()> {
        let mut hasher =
            std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        let index = hasher.finish() as usize % N;
        self.locks[index].lock().await
    }
}

/// 一个更高效的分段锁实现
///
/// 实际上该实现不像是分段锁了。
/// 该实现可以保证对单独 key 加锁时，同 key 的线程是被互斥的，但是不同 key 的线程是可以并发的。
/// 不像 [SegLock] 可能由于哈希冲突导致不同 key 也被互斥。
/// 且相对于单纯使用 [DashMap] 来说，能够在锁释放时及时地回收内存。
///
/// 基准测试：见 `src/benches/seg_lock_bench.rs`
pub struct NewSegLock {
    map: DashMap<String, Arc<tokio::sync::Mutex<bool>>>,
}

impl Default for NewSegLock {
    fn default() -> Self {
        Self::new()
    }
}

impl NewSegLock {
    pub fn new() -> Self {
        Self {
            map: DashMap::new(),
        }
    }
    pub async fn lock(&self, key: &str) -> NewSegLockGuard<'_> {
        loop {
            let value =
                self.map.entry(key.to_string()).or_insert_with(
                    || Arc::new(tokio::sync::Mutex::new(true)),
                );
            // 将 dashmap 中的 arc 复制出来之后立即释放 value，减少占用 dashmap 的时间
            let arc = value.clone();
            drop(value);
            let guard = arc.lock_owned().await;
            if !*guard {
                continue;
            }
            return NewSegLockGuard {
                lock: self,
                guard,
                key: key.to_string(),
            };
        }
    }
}

pub struct NewSegLockGuard<'a> {
    lock: &'a NewSegLock,
    guard: OwnedMutexGuard<bool>,
    key: String,
}

impl Drop for NewSegLockGuard<'_> {
    fn drop(&mut self) {
        *self.guard = false;
        // 及时从 dashmap 中移除 key，减少内存占用
        // 但是有可能其他线程已经在当前线程持锁期间将 key 对应的 arc 复制出来并等待当前锁释放
        // 所以我们需要对锁维护一个有效标记，在释放当前锁之前将有效标记置为 false，
        // 这样其他线程在拿到锁之后发现有效标记为 false，就会立即重新从 dashmap 获取最新的 arc
        self.lock.map.remove(&self.key);
    }
}
