use crate::{netflow::login::netflow_headers, utils::client};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const NETFLOW_MONTH_URL: &str = "http://ll.hnu.edu.cn/api/v1/history/getfloatdetailbymonth?month=";
const NETFLOW_DAY_URL: &str =
    "http://ll.hnu.edu.cn/api/v1/history/getfloatdetailbyday?day=";

#[derive(Deserialize, Serialize, Debug)]
#[expect(non_snake_case)]
pub struct Detail {
    pub AllDownload: f64,
    pub AllTotal: f64,
    pub AllUpload: f64,
    pub FloatDetailList: Vec<DetailItem>,
}

#[derive(Deserialize, Serialize, Debug)]
#[expect(non_snake_case)]
pub struct DetailItem {
    pub App: String,
    pub Download: f64,
    pub Per: f64,
    pub Total: f64,
    pub Upload: f64,
}

pub async fn raw_month_detail_data(
    stu_id: &str,
    year: u16,
    month: u8,
) -> Result<Detail, crate::Error> {
    let netflow_headers = netflow_headers(stu_id).await?;
    let url = format!("{NETFLOW_MONTH_URL}{}-{:0>2}", year, month);
    let raw_res = client
        .get(url)
        .headers(netflow_headers)
        .send()
        .await?
        .error_for_status()?
        .json::<Value>()
        .await?;
    let res: Detail = raw_res
        .get("data")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .ok_or(anyhow!("解析月流量详情失败: {:?}", raw_res))?;
    Ok(res)
}

pub async fn raw_day_detail_data(
    stu_id: &str,
    year: u16,
    month: u8,
    day: u8,
) -> Result<Detail, crate::Error> {
    let netflow_headers = netflow_headers(stu_id).await?;
    let url =
        format!("{NETFLOW_DAY_URL}{}{:0>2}{:0>2}", year, month, day);
    let raw_res = client
        .get(url)
        .headers(netflow_headers)
        .send()
        .await?
        .error_for_status()?
        .json::<Value>()
        .await?;
    let res: Detail = raw_res
        .get("data")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .ok_or(anyhow!("解析日流量详情失败: {:?}", raw_res))?;
    Ok(res)
}
