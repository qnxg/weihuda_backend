use crate::{netflow::login::netflow_headers, utils::client};
use anyhow::anyhow;
use serde::Deserialize;
use serde_json::Value;

const NETFLOW_ORDER_URL: &str =
    "http://ll.hnu.edu.cn/api/v1/historyorder/getpagedlist";

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct OrderItem {
    // pub AddTime: String,
    // pub AllowOverTraffic: f64,
    // pub BaseTraffic: f64,
    pub Download: Option<f64>,
    // pub ExtTraffic: f64,
    pub Month: String,
    // pub PayOrderCode: Option<String>,
    /// 1:已支付 0:未支付
    // pub PayState: u32,
    pub RealOverTraffic: f64,
    pub ShouldPay: f64,
    // pub Total: f64,
    pub UpdateTime: String,
    pub Upload: Option<f64>,
    // pub Year: String,
}

pub async fn raw_order_data(
    stu_id: &str,
) -> Result<Vec<OrderItem>, crate::Error> {
    let netflow_headers = netflow_headers(stu_id).await?;
    let raw_res = client
        .get(NETFLOW_ORDER_URL)
        .headers(netflow_headers)
        .send()
        .await?
        .error_for_status()?
        .json::<Value>()
        .await?;
    let res: Vec<OrderItem> = raw_res
        .get("data")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .ok_or(anyhow!("解析订单失败: {:?}", raw_res))?;
    Ok(res)
}
