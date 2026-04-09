use crate::utils::client;
use anyhow::anyhow;
use serde_json::Value;

const QUERY_URL: &str =
    "http://wxpay.hnu.edu.cn/api/appElectricCharge/checkRoomNo";

pub async fn raw_electricity_data(
    park: u8,
    building: &str,
    room: &str,
) -> Result<String, crate::Error> {
    let res = client
        .get(format!(
            "{}?parkNo={}&buildingNo={}&rechargeType=2&roomNo={}",
            QUERY_URL, park, building, room
        ))
        .header(
            "referer",
            "http://wxpay.hnu.edu.cn/electricCharge/home/",
        )
        .header("X-Requested-With", "XMLHttpRequest")
        .send()
        .await?
        .json::<Value>()
        .await?;
    Ok(res
        .get("data")
        .and_then(|data| data.get("Balance"))
        .and_then(|balance| balance.as_str())
        .ok_or(anyhow!("解析数据失败，data: {:?}", res))?
        .to_string())
}
