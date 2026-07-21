use dashmap::DashMap;
use std::sync::{Arc, Weak};

pub struct SingleFlight<T> {
    map: DashMap<String, Weak<tokio::sync::Mutex<Option<T>>>>,
}

impl<T> Default for SingleFlight<T> {
    fn default() -> Self {
        Self {
            map: DashMap::new(),
        }
    }
}

impl<T: Clone> SingleFlight<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn call(
        &self,
        key: &str,
        f: impl AsyncFnOnce() -> T,
    ) -> T {
        let local_arc = Arc::new(tokio::sync::Mutex::new(None));
        let mut value = self
            .map
            .entry(key.to_string())
            .or_insert_with(|| Arc::downgrade(&local_arc));
        let arc = match value.upgrade() {
            Some(arc) => arc,
            None => {
                *value = Arc::downgrade(&local_arc);
                local_arc
            }
        };
        drop(value);
        let mut guard = arc.lock().await;
        if let Some(v) = guard.as_ref() {
            v.clone()
        } else {
            let v = f().await;
            *guard = Some(v.clone());
            // 如果调用者在这里被取消掉，那么就会导致 DashMap 中的 key 没有被释放
            // key 会一直在 DashMap 中保留，知道下一次同 key 的调用出现
            self.map.remove(key);
            v
        }
    }
}
