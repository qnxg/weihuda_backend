use crate::{netflow::login::netflow_headers, utils::client};
use anyhow::anyhow;
use serde::Deserialize;
use serde_json::Value;

const NETFLOW_PAY_INFO_URL: &str =
    "http://ll.hnu.edu.cn/api/v1/pay/getpayinfo";

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct PayInfo {
    pub Total: f64,
}

pub async fn raw_pay_info_data(
    stu_id: &str,
) -> Result<PayInfo, crate::Error> {
    let netflow_headers = netflow_headers(stu_id).await?;
    let raw_res = client
        .get(NETFLOW_PAY_INFO_URL)
        .headers(netflow_headers)
        .send()
        .await?
        .error_for_status()?
        .json::<Value>()
        .await?;
    let res = raw_res
        .get("data")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .ok_or(anyhow!("解析支付信息失败: {:?}", raw_res))?;
    Ok(res)
}
