use crate::{
    infra::{self},
    result::AppResult,
};
use anyhow::anyhow;

const REDIS_PERSON_INFO_KEY_PREFIX: &str = "person_info-";
const REDIS_PERSON_INFO_TTL: u64 = 60 * 60 * 24 * 7; // 7天

pub use infra::mysql::user::get_user_setting;
pub use infra::mysql::user::update_user_setting;
pub use spider_2024::xgxt::personal_info::{Dormitory, PersonalInfo};

/// 缓存到 Redis 中
pub async fn get_person_info(
    stu_id: &str,
) -> AppResult<PersonalInfo> {
    let key = format!("{}{}", REDIS_PERSON_INFO_KEY_PREFIX, stu_id);
    if let Some(person_info) = infra::redis::get(&key).await? {
        // TODO 缓存解析失败要主动失效处理
        return Ok(serde_json::from_str(&person_info)?);
    }
    let person_info =
        spider_2024::xgxt::get_person_info(stu_id).await?;
    infra::redis::set_with_expire(
        &key,
        &serde_json::to_string(&person_info)?,
        REDIS_PERSON_INFO_TTL,
    )
    .await?;
    Ok(person_info)
}

/// 从数据库中获得宿舍信息
///
/// # Returns
///
/// - 如果数据库中没有宿舍信息，则返回 None
/// - 如果数据库中的宿舍信息解析失败，则返回 None
///     - 数据库中对应的字段如果是由两个 `/` 分割的字符串，则视为解析成功
///     - 两个 `/` 将字符串分成三部分，分别表示园区、楼栋、房间
///     - 构造 `Dormitory` 时会将这三部分视为是原来成功解析的 `Dormitory` 的 `park`、`build`、`room` 字段
///     - 如果数据库被人为修改，或是依赖的爬虫 crate 对 `Dormitory` 字段的约定有变动，则这里会产生未定义行为（TODO）
/// - 正常情况下，这里返回的 `Dormitory` 满足 `Dormitory::successfully_parsed() == true`
pub async fn get_dormitory(
    stu_id: &str,
) -> AppResult<Option<Dormitory>> {
    if let Some(dormitory) =
        infra::mysql::user::get_room(stu_id).await?
    {
        if dormitory == "0" || dormitory.is_empty() {
            return Ok(None);
        }
        let arr: Vec<&str> = dormitory.split("/").collect();
        let [park, build, room] = arr[..] else {
            return Ok(None);
        };
        Ok(Some(Dormitory::from_parsed_value(park, build, room)))
    } else {
        Ok(None)
    }
}

/// 重新调用爬虫更新寝室信息
pub async fn update_dormitory(stu_id: &str) -> AppResult<()> {
    // 先删掉 redis 中之前缓存的个人信息数据，防止宿舍信息没有更新
    let key = format!("{}{}", REDIS_PERSON_INFO_KEY_PREFIX, stu_id);
    infra::redis::del(&key).await?;
    let dormitory = get_person_info(stu_id).await?.dormitory;
    let (Some(park), Some(build), room) =
        (dormitory.park(), dormitory.build(), dormitory.room())
    else {
        return Err(anyhow!("宿舍信息解析失败").into());
    };
    // 解析宿舍信息为我们需要的格式
    let dormitory_str = format!("{}/{}/{}", park, build, room);
    infra::mysql::user::update_room(stu_id, &dormitory_str).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TEST_STU_ID;

    #[tokio::test]
    async fn test_get_dormitory() {
        let dormitory = get_dormitory(&TEST_STU_ID).await.unwrap();
        println!("{:#?}", dormitory);
    }

    #[tokio::test]
    async fn test_update_dormitory() {
        update_dormitory(&TEST_STU_ID).await.unwrap();
    }

    #[tokio::test]
    async fn test_get_person_info() {
        let person_info =
            get_person_info(&TEST_STU_ID).await.unwrap();
        println!("{:#?}", person_info);
    }
}
