use crate::{
    infra,
    result::AppResult,
    service::{self},
};
use anyhow::anyhow;

const REDIS_ELECTRICITY_KEY_PREFIX: &str = "electricity-";
const REDIS_ELECTRICITY_TTL: u64 = 60 * 60 * 16; // 16小时

/// 默认情况下是带缓存的，设置 refresh=true 则强制刷新
pub async fn get_electricity(
    stu_id: &str,
    refresh: bool,
) -> AppResult<String> {
    // 拉取
    let mut dormitory =
        service::user_info::get_dormitory(stu_id).await?;
    if dormitory.is_none() {
        service::user_info::update_dormitory(stu_id).await?;
        dormitory = service::user_info::get_dormitory(stu_id).await?;
    }
    // 还为空就摆烂
    let dormitory = dormitory.ok_or(anyhow!("获取宿舍信息失败"))?;
    let park = dormitory.park().expect("dormitory 应成功解析");
    let build = dormitory.build().expect("dormitory 应成功解析");
    let room = dormitory.room();
    let key = format!(
        "{}{}/{}/{}",
        REDIS_ELECTRICITY_KEY_PREFIX, park, build, room
    );
    if !refresh
        && let Some(electricity) = infra::redis::get(&key).await?
    {
        return Ok(electricity);
    }
    // 需要强制刷新，或是之前的缓存过期
    let electricity =
        spider_2024::wxpay::get_electricity(dormitory).await?;
    infra::redis::set_with_expire(
        &key,
        &electricity,
        REDIS_ELECTRICITY_TTL,
    )
    .await?;
    Ok(electricity)
}

#[cfg(test)]
mod tests {
    use crate::test::TEST_STU_ID;

    use super::*;

    #[tokio::test]
    async fn test_get_electricity() {
        let res = get_electricity(&TEST_STU_ID, true).await.unwrap();
        println!("电量信息：{}", res);
    }
}
