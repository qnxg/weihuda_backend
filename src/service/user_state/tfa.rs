use crate::{
    error::AppResult, infra::cache::update_cache,
    service::user_state::systems::CasCookieCacheKey,
};
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
) -> AppResult<()> {
    update_cache(
        CasCookieCacheKey::new(stu_id),
        cas_token.cookie().to_string(),
    )
    .await?;
    Ok(())
}
