use std::time::Duration;

use crate::{
    error::{
        AppResult, ThrowInternalErrorMsg, ThrowInternalErrorResult,
    },
    infra::cache::{
        CacheKey, CacheStrategy, invalidate_cache, with_cache,
    },
    service::{self},
};

#[derive(Debug, Clone)]
struct ElectricityCacheKey {
    park: String,
    build: String,
    room: String,
}

impl ElectricityCacheKey {
    fn new(park: &str, build: &str, room: &str) -> Self {
        Self {
            park: park.to_string(),
            build: build.to_string(),
            room: room.to_string(),
        }
    }
}

impl CacheKey for ElectricityCacheKey {
    const PREFIX: &'static str = "electricity";
    const VERSION: u64 = 1;
    type Value = String;
    fn strategy(&self) -> CacheStrategy {
        CacheStrategy::new(
            format!("{}:{}:{}", self.park, self.build, self.room),
            Duration::from_hours(4),
        )
    }
}

/// 默认情况下是带缓存的，设置 refresh=true 则强制刷新
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
    let key = ElectricityCacheKey::new(park, build, room);
    if refresh {
        invalidate_cache(key.clone()).await?;
    }
    let electricity = with_cache(key, async || {
        hnu_query::wxpay::get_electricity(dormitory)
            .await
            .internal_err()
    })
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
