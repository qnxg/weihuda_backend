use anyhow::anyhow;
use serde::Deserialize;
use serde_json::Value;

use crate::{netflow::login::netflow_headers, utils::client};

const THIS_MONTH_URL: &str =
    "http://ll.hnu.edu.cn/api/v1/history/gettrafficinfobythismonth";

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct ThisMonthInfo {
    pub allBasePackageAmount: f64,
    pub allExtendPackageAmount: f64,
    pub allTraffic: String,
    pub basePackageUsed: f64,
    pub basePackageUsedPer: f64,
    pub downloadTraffic: String,
    pub extendPackageUsed: f64,
    pub extendPackageUsedPer: f64,
    pub surplusBasePackage: f64,
    pub surplusExtendPackage: f64,
    pub uploadTraffic: String,
}

/// 本月流量数据
pub async fn raw_this_month_data(
    stu_id: &str,
) -> Result<ThisMonthInfo, crate::Error> {
    let netflow_headers = netflow_headers(stu_id).await?;
    let raw_res: Value = client
        .get(THIS_MONTH_URL)
        .headers(netflow_headers)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let res = raw_res
        .get("data")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .ok_or(anyhow!("解析本月流量数据失败: {:?}", raw_res))?;
    Ok(res)
}
