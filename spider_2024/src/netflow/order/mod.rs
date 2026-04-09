use crate::netflow::order::raw::raw_order_data;
use anyhow::anyhow;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

mod raw;

#[derive(Deserialize, Serialize, Debug)]
pub struct OrderItem {
    /// 时间
    ///
    /// 为 `YYYY-MM` 格式，如 `2025-01`
    pub time: String,
    /// 使用的下载流量
    ///
    /// 单位: 字节
    pub download_usage: f64,
    /// 使用的上传流量
    ///
    /// 单位: 字节
    pub upload_usage: f64,
    /// 超额流量
    ///
    /// 单位: GB
    pub over_usage: f64,
    /// 应缴费用
    ///
    /// 单位: 元
    pub should_pay: f64,
    /// 更新时间
    pub update_time: NaiveDateTime,
}

pub async fn get_order(
    stu_id: &str,
) -> Result<Vec<OrderItem>, crate::Error> {
    let raw_data = raw_order_data(stu_id).await?;
    let mut res = Vec::with_capacity(raw_data.len());
    for item in raw_data {
        let temp = OrderItem {
            time: item.Month,
            download_usage: item.Download.unwrap_or_default(),
            upload_usage: item.Upload.unwrap_or_default(),
            over_usage: item.RealOverTraffic,
            should_pay: item.ShouldPay,
            update_time: NaiveDateTime::parse_from_str(
                &item.UpdateTime,
                "%Y-%m-%d %H:%M:%S",
            )
            .map_err(|e| anyhow!("解析更新时间失败: {e}"))?,
        };
        res.push(temp);
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TEST_STU_ID;

    #[tokio::test]
    async fn test_get_order() {
        let res = get_order(&TEST_STU_ID).await.unwrap();
        println!("{:#?}", res);
    }
}
