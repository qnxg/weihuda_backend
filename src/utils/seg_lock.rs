use std::hash::{Hash, Hasher};

pub struct SegLock<const N: usize> {
    locks: [tokio::sync::Mutex<()>; N],
}

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
