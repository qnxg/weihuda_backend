use std::{
    collections::{HashSet, VecDeque},
    hash::Hash,
};
use tokio::sync::{Mutex, Semaphore};

use crate::error::AppResult;

/// 一个简单的 MPMC 带去重的队列实现
pub struct UniqueTaskQueue<K, V> {
    #[expect(clippy::type_complexity)]
    queue: Mutex<(VecDeque<(K, V)>, HashSet<K>)>,
    sem: Semaphore,
}

impl<K: Hash + Eq + Clone, V> UniqueTaskQueue<K, V> {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new((VecDeque::new(), HashSet::new())),
            sem: Semaphore::new(0),
        }
    }
    /// 如果 `key` 已经存在于队列中，则调用 `on_contains`。
    ///
    /// 否则，则在将 `value` 加入队列之前调用 `before_push`。如果 `before_push` 抛出了错误，则不再继续将 `value` 加入队列，
    /// 同时还会将 `before_push` 的错误返回给调用者。
    pub async fn push(
        &self,
        key: K,
        value: V,
        on_contains: impl AsyncFnOnce() -> (),
        before_push: impl AsyncFnOnce() -> AppResult<()>,
    ) -> AppResult<()> {
        let mut guard = self.queue.lock().await;
        let (queue, set) = &mut *guard;
        if set.contains(&key) {
            on_contains().await;
            return Ok(());
        }
        before_push().await?;
        queue.push_back((key.clone(), value));
        set.insert(key);
        self.sem.add_permits(1);
        Ok(())
    }
    /// 如果队列为空，阻塞等待
    ///
    /// 注意：即使元素被 pop 出去，队列中仍记录着该元素的 `key`，后续同 `key` 元素仍然无法被加入到队列中。
    /// 你需要在确保元素被完全处理后，调用 [UniqueTaskQueue::ack] 来允许后续同 `key` 元素可以再次被加入到队列中。
    ///
    /// # Safety
    ///
    /// 该函数并不能保证任务取消时的安全，调用时请确保当前任务不可能被取消
    pub async fn pop(&self) -> (K, V) {
        let rem = self.sem.acquire().await.expect("获取信号量失败");
        rem.forget();
        let mut guard = self.queue.lock().await;
        let (queue, _) = &mut *guard;
        let (key, value) =
            queue.pop_front().expect("获取到了信号量，但是队列为空");
        (key, value)
    }
    /// 确认元素被完全处理后，允许后续同 `key` 元素可以再次被加入到队列中。
    pub async fn ack(&self, key: K) {
        let mut guard = self.queue.lock().await;
        let (_, set) = &mut *guard;
        set.remove(&key);
    }
}
