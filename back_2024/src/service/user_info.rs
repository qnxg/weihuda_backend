use crate::{
    infra::{self},
    result::AppResult,
    utils::cache::{CACHE, CacheEnum},
};

pub use infra::mysql::user::get_user_setting;
pub use infra::mysql::user::update_user_setting;
pub use spider_2024::xgxt::personal_info::PersonalInfo;

/// 缓存到 Redis 中
/// - `refresh` 是否强制刷新，如果为 true，则忽略缓存，重新获取
pub async fn get_person_info(
    stu_id: &str,
    refresh: bool,
) -> AppResult<PersonalInfo> {
    if !refresh
        && let Some(person_info) = CACHE
            .get(&(CacheEnum::PersonalInfo, stu_id.to_string()))
            .await
    {
        // TODO 缓存解析失败要主动失效处理
        return Ok(serde_json::from_str(&person_info)?);
    }
    let person_info =
        spider_2024::xgxt::get_person_info(stu_id).await?;
    CACHE
        .insert(
            (CacheEnum::PersonalInfo, stu_id.to_string()),
            serde_json::to_string(&person_info)?,
        )
        .await;
    Ok(person_info)
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
