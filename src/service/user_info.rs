use std::time::Duration;

use crate::{
    error::{AppError, AppResult},
    infra::{
        self,
        cache::{
            CacheAsyncUpdateResult, CacheKey, CacheStrategy,
            invalidate_cache, with_cache_async_update,
        },
    },
    service::user_state::{Xgxt, with_token},
    utils,
};

pub use hnu_query::xgxt::personal_info::{Level, PersonalInfo};
pub use infra::mysql::user::get_user_setting;
pub use infra::mysql::user::update_user_setting;

#[derive(Debug, Clone)]
struct PersonalInfoCacheKey {
    stu_id: String,
}

impl PersonalInfoCacheKey {
    pub fn new(stu_id: &str) -> Self {
        Self {
            stu_id: stu_id.to_string(),
        }
    }
}

impl CacheKey for PersonalInfoCacheKey {
    const PREFIX: &'static str = "personal_info";
    const VERSION: u64 = 1;
    type Value = PersonalInfo;
    fn strategy(&self) -> CacheStrategy {
        CacheStrategy::new(
            self.stu_id.clone(),
            Duration::from_hours(24 * 7),
        )
    }
}

/// 带缓存
///
/// - `refresh` 是否强制刷新，如果为 true，则忽略缓存，重新获取
pub async fn get_person_info(
    stu_id: &str,
    refresh: bool,
) -> AppResult<PersonalInfo> {
    let key = PersonalInfoCacheKey::new(stu_id);
    if refresh {
        invalidate_cache(key.clone()).await?;
    }
    let res = with_cache_async_update(key, || {
        let stu_id = stu_id.to_string();
        async move {
            match with_token(Xgxt::new(stu_id), async move |token| {
                hnu_query::xgxt::get_person_info(&token).await
            })
            .await
            {
                Ok(v) => CacheAsyncUpdateResult::Ok(v),
                Err(e) => CacheAsyncUpdateResult::Extend(e),
            }
        }
    })
    .await?;
    Ok(res)
}

/// 判断学号是否为研究生
pub async fn is_graduate(stu_id: &str) -> AppResult<bool> {
    let info = get_person_info(stu_id, false).await?;
    Ok(info.level == Level::Postgraduate
        || info.level == Level::Doctoral)
}

pub async fn get_password(stu_id: &str) -> AppResult<String> {
    let password = infra::mysql::user::get_password(stu_id)
        .await?
        .ok_or_else(AppError::password_error)?;
    let password =
        utils::crypto::decrypt(&password).map_err(|e| {
            tracing::error!(error = %e, "解密密码失败");
            AppError::password_error()
        })?;
    Ok(password)
}

pub async fn get_lab_password(stu_id: &str) -> AppResult<String> {
    let password = infra::mysql::user::get_lab_password(stu_id)
        .await?
        .ok_or_else(AppError::password_error)?;
    let password =
        utils::crypto::decrypt(&password).map_err(|e| {
            tracing::error!(error = %e, "解密密码失败");
            AppError::password_error()
        })?;
    Ok(password)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TEST_STU_ID;

    #[tokio::test]
    async fn test_get_person_info() {
        let person_info =
            get_person_info(&TEST_STU_ID, true).await.unwrap();
        println!("{:#?}", person_info);
    }
}
