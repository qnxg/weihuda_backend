use crate::{
    error::{
        AppResult, ThrowInternalErrorMsg, ThrowInternalErrorResult,
    },
    service::{self},
    utils,
    utils::cache::{CACHE, CacheEnum::Electricity},
};

/// 默认情况下是带缓存的，设置 refresh=true 则强制刷新
#[tracing::instrument(
    fields(
        otel.kind = "internal",
        event_type = "service", 
        cache_result = tracing::field::Empty,
    ),
    err
)]
pub async fn get_electricity(
    stu_id: &str,
    refresh: bool,
) -> AppResult<String> {
    // 拉取
    let Some(dormitory) =
        service::user_info::get_person_info(stu_id, false)
            .await?
            .dormitory
    else {
        return Err("无法获取到宿舍信息"
            .internal_err()
            .show("无法获取到宿舍信息")
            .into());
    };
    let (Some(park), Some(build)) =
        (dormitory.park(), dormitory.build())
    else {
        tracing::error!(dormitory = ?dormitory, "尚不支持的宿舍");
        return Err("尚不支持你的宿舍"
            .internal_err()
            .show("尚不支持你的宿舍")
            .into());
    };
    let room = dormitory.room();
    let key = format!("{}/{}/{}", park, build, room);
    if !refresh
        && let Some(electricity) =
            CACHE.get(&(Electricity, key.clone())).await
    {
        utils::record!(cache_result = "hit");
        return Ok(electricity);
    }
    // 需要强制刷新，或是之前的缓存过期
    let electricity = hnu_query::wxpay::get_electricity(dormitory)
        .await
        .internal_err()?;
    CACHE
        .insert((Electricity, key.clone()), electricity.clone())
        .await;
    utils::record!(cache_result = "miss");
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
