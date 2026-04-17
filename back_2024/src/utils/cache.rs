use std::{sync::LazyLock, time::Duration};

use moka::{Expiry, future::Cache};

pub type CacheKey = (CacheEnum, String);
pub type CacheVal = String;

#[derive(Eq, Hash, PartialEq)]
pub enum CacheEnum {
    AuthQrCode,
    Electricity,
    PersonalInfo,
}

impl CacheEnum {
    /// 缓存在moka中的超时时间
    fn expire_after_fetch(&self) -> Option<Duration> {
        use CacheEnum::*;
        match self {
            AuthQrCode => Some(Duration::from_secs(600)),
            Electricity => Some(Duration::from_secs(60 * 60 * 16)),
            PersonalInfo => {
                Some(Duration::from_secs(60 * 60 * 24 * 7))
            }
        }
    }
}

pub static CACHE: LazyLock<Cache<CacheKey, CacheVal>> =
    LazyLock::new(|| {
        Cache::builder()
            .weigher(|k: &CacheKey, v: &CacheVal| {
                (k.1.len() + v.len() + 1)
                    .try_into()
                    .unwrap_or(u32::MAX)
            })
            // capacity是按照上面的weigher来算的，从而可以从内存容量角度来限制
            .max_capacity(2 * 1024 * 1024 * 1024)
            .expire_after(ExpiryPolicy)
            .build()
    });

struct ExpiryPolicy;

impl Expiry<CacheKey, CacheVal> for ExpiryPolicy {
    fn expire_after_create(
        &self,
        key: &CacheKey,
        _value: &CacheVal,
        _created_at: std::time::Instant,
    ) -> Option<Duration> {
        key.0.expire_after_fetch()
    }

    fn expire_after_update(
        &self,
        key: &CacheKey,
        _value: &CacheVal,
        _updated_at: std::time::Instant,
        _duration_until_expiry: Option<std::time::Duration>,
    ) -> Option<std::time::Duration> {
        key.0.expire_after_fetch()
    }
}
