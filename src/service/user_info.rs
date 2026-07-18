use crate::{
    error::{AppError, AppResult, ThrowInternalError},
    infra::{self},
    service::user_state::{Xgxt, with_token},
    utils::{
        self,
        cache::{CACHE, CacheEnum},
    },
};

pub use hnu_query::xgxt::personal_info::{Level, PersonalInfo};
pub use infra::mysql::user::get_user_setting;
pub use infra::mysql::user::update_user_setting;

/// 带缓存
///
/// - `refresh` 是否强制刷新，如果为 true，则忽略缓存，重新获取
#[tracing::instrument(
    fields(
        otel.kind = "internal", 
        event_type = "service", 
        cache_result = "hit",
    ),
    err
)]
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
            utils::record!(cache_result = "miss");
            let person_info =
                with_token(Xgxt::new(stu_id), async move |token| {
                    hnu_query::xgxt::get_person_info(&token).await
                })
                .await?;
            let cached_value = serde_json::to_string(&person_info)
                .map_err(|e| {
                    e.internal_err().with("序列化个人信息失败")
                })?;
            Ok(cached_value)
        })
        .await?;
    let person_info = serde_json::from_str(&res)
        .map_err(|e| e.internal_err().with("反序列化个人信息失败"))?;
    Ok(person_info)
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
