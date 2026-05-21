use crate::result::AppResult;
use moka::{Expiry, future::Cache};
use sha2::Digest;
use std::{sync::LazyLock, time::Duration};
use uuid::Uuid;

/// pow ticket 有效期
const POW_EXPIRE_TIME: Duration = Duration::from_mins(5);
struct ExpiryPolicy;

impl Expiry<String, String> for ExpiryPolicy {
    fn expire_after_create(
        &self,
        _key: &String,
        _value: &String,
        _created_at: std::time::Instant,
    ) -> Option<Duration> {
        Some(POW_EXPIRE_TIME)
    }

    fn expire_after_update(
        &self,
        _key: &String,
        _value: &String,
        _updated_at: std::time::Instant,
        _duration_until_expiry: Option<std::time::Duration>,
    ) -> Option<std::time::Duration> {
        Some(POW_EXPIRE_TIME)
    }
}

static CACHE: LazyLock<Cache<String, String>> = LazyLock::new(|| {
    Cache::builder()
        .weigher(|k: &String, v: &String| {
            (k.len() + v.len() + 1).try_into().unwrap_or(u32::MAX)
        })
        // capacity是按照上面的weigher来算的，从而可以从内存容量角度来限制
        .max_capacity(2 * 1024 * 1024 * 1024)
        .expire_after(ExpiryPolicy)
        .build()
});

pub const POW_DIFFICULTY: usize = 4;

/// 给对应学号生成一个 pow ticket，用于后续的 pow 验证
///
/// ticket 有效期为 [POW_EXPIRE_TIME]
pub async fn generate_pow(stu_id: &str) -> AppResult<String> {
    let ticket = Uuid::new_v4().simple().to_string();
    CACHE.insert(ticket.clone(), stu_id.to_string()).await;
    Ok(ticket)
}

/// 验证 pow 答案
///
/// 如果 ticket 不存在或者已经过期，则返回 None
///
/// 无论验证是否成功，都会删除 ticket
pub async fn verify_pow(
    ticket: &str,
    answer: usize,
) -> AppResult<Option<String>> {
    let Some(stu_id) = CACHE.remove(ticket).await else {
        return Ok(None);
    };
    let text = format!("{}:{}", ticket, answer);
    let hash = sha2::Sha256::digest(text.as_bytes());
    let hash = hex::encode(hash);
    if hash.starts_with(&"0".repeat(POW_DIFFICULTY)) {
        // TODO 加锁？
        Ok(Some(stu_id))
    } else {
        Ok(None)
    }
}
