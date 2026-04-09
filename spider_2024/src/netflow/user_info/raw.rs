use anyhow::anyhow;
use serde_json::Value;
const NETFLOW_USER_INFO_URL: &str =
    "http://ll.hnu.edu.cn/api/v1/account/getuserinfo";
use crate::{netflow::login::netflow_headers, utils::client};

pub async fn raw_user_info_data(
    stu_id: &str,
) -> Result<Value, crate::Error> {
    let netflow_headers = netflow_headers(stu_id).await?;
    let raw_res = client
        .get(NETFLOW_USER_INFO_URL)
        .headers(netflow_headers)
        .send()
        .await?
        .error_for_status()?
        .json::<Value>()
        .await?;
    let res = raw_res
        .get("data")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .ok_or(anyhow!("解析用户信息失败: {:?}", raw_res))?;
    Ok(res)
}
