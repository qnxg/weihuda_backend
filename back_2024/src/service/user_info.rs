use crate::{
    infra::{self},
    result::{AppError, AppResult, ThrowError},
    service::user_state::{Xgxt, with_token},
    utils::{
        self,
        cache::{CACHE, CacheEnum},
    },
};

pub use infra::mysql::user::get_user_setting;
pub use infra::mysql::user::update_user_setting;
pub use spider_2024::xgxt::personal_info::PersonalInfo;

/// 带缓存
///
/// - `refresh` 是否强制刷新，如果为 true，则忽略缓存，重新获取
pub async fn get_person_info(
    stu_id: &str,
    refresh: bool,
) -> AppResult<PersonalInfo> {
    let key = (CacheEnum::PersonalInfo, stu_id.to_string());
    if refresh {
        CACHE.invalidate(&key).await;
    }
    let res = CACHE
        .try_get_with(key, async {
            let person_info =
                with_token(Xgxt::new(stu_id), async move |token| {
                    spider_2024::xgxt::get_person_info(&token).await
                })
                .await?;
            let cached_value = serde_json::to_string(&person_info)
                .throw_error("序列化个人信息失败")?;
            Ok(cached_value)
        })
        .await?;
    let person_info = serde_json::from_str(&res)
        .throw_error("反序列化个人信息失败")?;
    Ok(person_info)
}

pub async fn get_password(stu_id: &str) -> AppResult<String> {
    let password = infra::mysql::user::get_password(stu_id)
        .await?
        .ok_or(AppError::PasswordError)?;
    let password =
        utils::crypto::decrypt(&password).map_err(|e| {
            tracing::error!(error = %e, "解密密码失败");
            AppError::PasswordError
        })?;
    Ok(password)
}

pub async fn get_lab_password(stu_id: &str) -> AppResult<String> {
    let password = infra::mysql::user::get_lab_password(stu_id)
        .await?
        .ok_or(AppError::PasswordError)?;
    let password =
        utils::crypto::decrypt(&password).map_err(|e| {
            tracing::error!(error = %e, "解密密码失败");
            AppError::PasswordError
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
