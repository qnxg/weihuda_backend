#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};

//====流量订单====//
/// 流量情况
#[derive(Deserialize, Debug)]
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
/// 欠费情况
#[derive(Deserialize, Debug)]
pub struct SpiderNetflowPayInfo {
    pub Total: f64,
}
/// 解锁情况
#[derive(Deserialize, Debug)]
pub struct SpiderNetflowUnlockStatus {
    pub status: String,
}

#[derive(Serialize, Debug)]
pub struct NetflowRes {
    pub thisMonth: NetflowResItem,
    pub unlock: String,
    pub payInfo: f64,
}
#[derive(Serialize, Debug)]
pub struct NetflowResItem {
    pub download: String,
    pub upload: String,
    pub all: String,
    pub allBasePackageAmount: f64,
    pub allExtendPackageAmount: f64,
    pub basePackageUsed: f64,      //本月可用流量 已用
    pub basePackageUsedPer: f64,   //本月可用流量 使用率
    pub surplusBasePackage: f64,   //本月可用流量 剩余
    pub extendPackageUsed: f64,    //本月超出流量 已用
    pub extendPackageUsedPer: f64, //本月超出流量 使用率
    pub surplusExtendPackage: f64, //本月超出流量 剩余
}

//====历史流量订单====//
// 不解析用不到的字段
#[derive(Deserialize, Debug)]
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

#[derive(Serialize, Debug)]
pub struct NetflowOrderRes {
    pub month: String,
    pub shouldPay: f64, // 应缴费用
    pub updateTime: String,
    pub uploadName: String,   // 上传流量
    pub downloadName: String, // 下载流量
    pub realOverTraffic: f64, // 流量超出数量
}

//====流量明细按月/天查询，爬虫获取结果不做修改直接返回给前端====//
#[derive(Deserialize, Serialize, Debug)]
pub struct SpiderNetflowMonthDetail {
    // #[serde(with = "serialize_f64")]
    pub AllDownload: f64,
    // #[serde(with = "serialize_f64")]
    pub AllTotal: f64,
    // #[serde(with = "serialize_f64")]
    pub AllUpload: f64,
    pub FloatDetailList: Vec<SpiderNetflowMonthDetailItem>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SpiderNetflowMonthDetailItem {
    pub App: String,
    pub Download: f64,
    pub Per: f64,
    pub Total: f64,
    pub Upload: f64,
}
