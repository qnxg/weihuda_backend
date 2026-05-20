use super::cache::{CACHE, CacheEnum};
use hnu_query::cas::{login::CasToken, tfa::TFAToken};
use moka::future::Cache;
use std::{sync::LazyLock, time::Duration};

pub static TFA_TOKEN: LazyLock<Cache<String, TFAToken>> =
    LazyLock::new(|| {
        Cache::builder()
            .time_to_live(Duration::from_mins(5))
            .max_capacity(1000)
            .build()
    });

pub async fn apply_verified_cas_token(
    stu_id: &str,
    cas_token: &CasToken,
) {
    CACHE
        .insert(
            (CacheEnum::CasToken, stu_id.to_string()),
            cas_token
                .cookie()
                .unwrap_or_else(|| {
                    tracing::warn!(
                        "通过双因子认证后 CasToken 内的 cookie 为空"
                    );
                    ""
                })
                .to_string(),
        )
        .await;
}
