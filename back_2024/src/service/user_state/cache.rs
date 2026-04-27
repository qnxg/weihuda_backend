use std::{sync::LazyLock, time::Duration};

use moka::{Expiry, future::Cache};

pub type CacheKey = (CacheEnum, String);
pub type CacheVal = String;

#[derive(Eq, Hash, PartialEq)]
#[expect(clippy::enum_variant_names)]
pub enum CacheEnum {
    CasToken,
    HdjwToken,
    GymToken,
    LabToken,
    CaToken,
    PtToken,
    NetflowToken,
    XGXTToken,
}

impl CacheEnum {
    /// 缓存在moka中的超时时间
    fn expire_after_fetch(&self) -> Option<Duration> {
        use CacheEnum::*;
        match self {
            CasToken => Some(Duration::from_secs(1800)),
            HdjwToken => Some(Duration::from_secs(1800)),
            GymToken => Some(Duration::from_secs(600)),
            LabToken => Some(Duration::from_secs(600)),
            CaToken => Some(Duration::from_secs(600)),
            PtToken => Some(Duration::from_secs(1800)),
            NetflowToken => Some(Duration::from_secs(1800)),
            XGXTToken => Some(Duration::from_secs(600)),
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
