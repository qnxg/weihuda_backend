use crate::{
    result::{AppResult, ThrowError},
    service::{self},
    utils::cache::{CACHE, CacheEnum::Electricity},
};

/// 默认情况下是带缓存的，设置 refresh=true 则强制刷新
pub async fn get_electricity(
    stu_id: &str,
    refresh: bool,
) -> AppResult<String> {
    // 拉取
    let dormitory =
        service::user_info::get_person_info(stu_id, false)
            .await?
            .dormitory;
    let (Some(park), Some(build)) =
        (dormitory.park(), dormitory.build())
    else {
        tracing::error!(dormitory = ?dormitory, "尚不支持的宿舍");
        return Err("尚不支持你的宿舍".into());
    };
    let room = dormitory.room();
    let key = format!("{}/{}/{}", park, build, room);
    if !refresh
        && let Some(electricity) =
            CACHE.get(&(Electricity, key.clone())).await
    {
        return Ok(electricity);
    }
    // 需要强制刷新，或是之前的缓存过期
    let electricity = hnu_query::wxpay::get_electricity(dormitory)
        .await
        .throw_error("获取电量信息失败")?;
    CACHE
        .insert((Electricity, key.clone()), electricity.clone())
        .await;
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
