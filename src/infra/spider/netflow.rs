use serde::{Deserialize, Serialize};

use crate::result::AppResult;

use super::spider_data;

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct SpiderNetflowThisMonth {
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
pub async fn get_netflow_this_month(
    stu_id: &str,
) -> AppResult<SpiderNetflowThisMonth> {
    let params = &[("stuid", stu_id)];
    let spider_res: SpiderNetflowThisMonth =
        spider_data("/netflow", &params).await?;
    Ok(spider_res)
}

#[derive(Deserialize, Debug)]
pub struct SpiderNetflowUnlockStatus {
    pub status: String,
}
pub async fn get_netflow_unlock_status(
    stu_id: &str,
) -> AppResult<SpiderNetflowUnlockStatus> {
    let params = &[("stuid", stu_id)];
    let spider_res: SpiderNetflowUnlockStatus =
        spider_data("/netflow/unlock", &params).await?;
    Ok(spider_res)
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct SpiderNetflowPayInfo {
    pub Total: f64,
}
pub async fn get_netflow_pay_info(
    stu_id: &str,
) -> AppResult<SpiderNetflowPayInfo> {
    let params = &[("stuid", stu_id)];
    let spider_res: SpiderNetflowPayInfo =
        spider_data("/netflow/pay_info", &params).await?;
    Ok(spider_res)
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct SpiderNetflowOrder {
    // pub AddTime: String,
    // pub AllowOverTraffic: f64,
    // pub BaseTraffic: f64,
    // pub Download: f64,
    pub DownloadName: String,
    // pub ExtTraffic: f64,
    pub Month: String,
    // pub PayOrderCode: Option<String>,
    // pub PayState: u32, // 1:已支付 0:未支付
    pub RealOverTraffic: f64,
    pub ShouldPay: f64,
    // pub Total: f64,
    pub UpdateTime: String,
    // pub Upload: f64,
    pub UploadName: String,
    // pub Year: String,
}
pub async fn get_netflow_order(
    stu_id: &str,
) -> AppResult<Vec<SpiderNetflowOrder>> {
    let params = &[("stuid", stu_id)];
    let spider_res: Vec<SpiderNetflowOrder> =
        spider_data("/netflow/order", &params).await?;
    Ok(spider_res)
}

#[derive(Deserialize, Serialize, Debug)]
#[expect(non_snake_case)]
pub struct SpiderNetflowDetail {
    // #[serde(with = "serialize_f64")]
    pub AllDownload: f64,
    // #[serde(with = "serialize_f64")]
    pub AllTotal: f64,
    // #[serde(with = "serialize_f64")]
    pub AllUpload: f64,
    pub FloatDetailList: Vec<SpiderNetflowDetailItem>,
}
#[derive(Deserialize, Serialize, Debug)]
#[expect(non_snake_case)]
pub struct SpiderNetflowDetailItem {
    pub App: String,
    pub Download: f64,
    pub Per: f64,
    pub Total: f64,
    pub Upload: f64,
}
pub async fn get_netflow_month_detail(
    stu_id: &str,
    year: &str,
    month: &str,
) -> AppResult<SpiderNetflowDetail> {
    let params =
        [("stuid", stu_id), ("year", year), ("month", month)];
    let spider_res: SpiderNetflowDetail =
        spider_data("/netflow/month_detail", &params).await?;
    Ok(spider_res)
}

pub async fn get_netflow_day_detail(
    stu_id: &str,
    year: &str,
    month: &str,
    day: &str,
) -> AppResult<SpiderNetflowDetail> {
    let params = [
        ("stuid", stu_id),
        ("year", year),
        ("month", month),
        ("day", day),
    ];
    let spider_res: SpiderNetflowDetail =
        spider_data("/netflow/day_detail", &params).await?;
    Ok(spider_res)
}
