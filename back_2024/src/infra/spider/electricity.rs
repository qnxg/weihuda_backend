use anyhow::anyhow;
use spider_2024::dtos::electricity::GetElectricityReq;

use crate::result::AppResult;

pub async fn get_electricity(
    park: &str,
    build: &str,
    room: &str,
    refresh: bool,
) -> AppResult<String> {
    let res = spider_2024::electricity::get_electricity_handler(
        GetElectricityReq {
            park: park
                .parse::<u8>()
                .map_err(|e| anyhow!("园区代码解析失败 {}", e))?,
            build: build.to_string(),
            room: room.to_string(),
            refresh,
        },
    )
    .await?;

    Ok(res)
}
