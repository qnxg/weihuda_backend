use serde::{Deserialize, Serialize};
use spider_2024::netflow::user_info::UnlockStatus;
use tokio::try_join;

use crate::{
    result::{AppError, AppResult},
    service::{
        netflow::utils::{
            bytes_to_gb, convert_netflow_detail, parse_year_month,
        },
        user_state::with_token,
    },
};

use crate::service::user_state::Netflow as NetflowSystem;

mod utils;

#[derive(Serialize, Debug)]
#[expect(non_snake_case)]
pub struct Netflow {
    pub thisMonth: NetflowResItem,
    pub unlock: String,
    pub payInfo: f64,
}
#[derive(Serialize, Debug)]
#[expect(non_snake_case)]
pub struct NetflowResItem {
    pub download: String,
    pub upload: String,
    pub all: String,
    pub allBasePackageAmount: f64,
    pub allExtendPackageAmount: f64,
    pub basePackageUsed: f64, //本月可用流量 已用
    pub basePackageUsedPer: f64, //本月可用流量 使用率
    pub surplusBasePackage: f64, //本月可用流量 剩余
    pub extendPackageUsed: f64, //本月超出流量 已用
    pub extendPackageUsedPer: f64, //本月超出流量 使用率
    pub surplusExtendPackage: f64, //本月超出流量 剩余
}

pub async fn get_netflow_info(stu_id: &str) -> AppResult<Netflow> {
    let f_get_this_month_info =
        with_token(NetflowSystem::new(stu_id), async move |token| {
            spider_2024::netflow::get_this_month_info(&token).await
        });
    let f_get_unlock_status =
        with_token(NetflowSystem::new(stu_id), async move |token| {
            spider_2024::netflow::get_unlock_status(&token).await
        });
    let f_get_overdue_payment =
        with_token(NetflowSystem::new(stu_id), async move |token| {
            spider_2024::netflow::get_overdue_payment(&token).await
        });
    let (this_month, unlock_status, pay_info) = try_join!(
        f_get_this_month_info,
        f_get_unlock_status,
        f_get_overdue_payment
    )?;
    let res = Netflow {
        thisMonth: NetflowResItem {
            download: this_month.download_usage,
            upload: this_month.upload_usage,
            all: this_month.total_usage,
            basePackageUsed: this_month.base_package_usage,
            basePackageUsedPer: this_month
                .base_package_usage_percentage,
            allBasePackageAmount: this_month.base_package_amount,
            extendPackageUsed: this_month.extend_package_usage,
            allExtendPackageAmount: this_month.extend_package_amount,
            surplusBasePackage: this_month.base_package_surplus,
            surplusExtendPackage: this_month.extend_package_surplus,
            extendPackageUsedPer: this_month
                .extend_package_usage_percentage,
        },
        payInfo: pay_info,
        unlock: match unlock_status {
            UnlockStatus::Locked => "已锁定".to_string(),
            UnlockStatus::Unlocked => "未锁定".to_string(),
            UnlockStatus::Unknown => "未知状态".to_string(),
        },
    };
    Ok(res)
}

#[derive(Serialize, Debug)]
#[expect(non_snake_case)]
pub struct NetflowOrder {
    pub month: String,
    pub shouldPay: f64, // 应缴费用
    pub updateTime: String,
    pub uploadName: String,   // 上传流量
    pub downloadName: String, // 下载流量
    pub realOverTraffic: f64, // 流量超出数量
}
pub async fn get_netflow_order(
    stu_id: &str,
) -> AppResult<Vec<NetflowOrder>> {
    let spider_res =
        with_token(NetflowSystem::new(stu_id), async move |token| {
            spider_2024::netflow::get_order(&token).await
        })
        .await?;
    let mut res = Vec::with_capacity(spider_res.len());
    for item in spider_res {
        let temp = NetflowOrder {
            month: item.time,
            shouldPay: item.should_pay,
            updateTime: item
                .update_time
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            uploadName: bytes_to_gb(item.upload_usage),
            downloadName: bytes_to_gb(item.download_usage),
            realOverTraffic: item.over_usage,
        };
        res.push((
            parse_year_month(&temp.month).ok_or(AppError::Text(
                "异常的年月字符串".to_string(),
            ))?,
            temp,
        ));
    }
    res.sort_by_key(|(month, _)| *month);
    res.reverse();
    Ok(res.into_iter().map(|(_, item)| item).collect())
}

#[derive(Deserialize, Serialize, Debug)]
#[expect(non_snake_case)]
pub struct NetflowDetailRes {
    // #[serde(with = "serialize_f64")]
    pub AllDownload: f64,
    // #[serde(with = "serialize_f64")]
    pub AllTotal: f64,
    // #[serde(with = "serialize_f64")]
    pub AllUpload: f64,
    pub FloatDetailList: Vec<NetflowDetailItemRes>,
}

#[derive(Deserialize, Serialize, Debug)]
#[expect(non_snake_case)]
pub struct NetflowDetailItemRes {
    pub App: String,
    pub Download: f64,
    pub Per: f64,
    pub Total: f64,
    pub Upload: f64,
}

pub async fn get_netflow_day_detail(
    stu_id: &str,
    year: u16,
    month: u8,
    day: u8,
) -> AppResult<NetflowDetailRes> {
    let spider_res =
        with_token(NetflowSystem::new(stu_id), async move |token| {
            spider_2024::netflow::get_day_detail(
                &token, year, month, day,
            )
            .await
        })
        .await?;
    Ok(convert_netflow_detail(spider_res))
}

pub async fn get_netflow_month_detail(
    stu_id: &str,
    year: u16,
    month: u8,
) -> AppResult<NetflowDetailRes> {
    let spider_res =
        with_token(NetflowSystem::new(stu_id), async move |token| {
            spider_2024::netflow::get_month_detail(
                &token, year, month,
            )
            .await
        })
        .await?;
    Ok(convert_netflow_detail(spider_res))
}
