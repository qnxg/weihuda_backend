use anyhow::anyhow;
use chrono::{
    NaiveDate,
    format::{Parsed, StrftimeItems},
};
use serde::Serialize;
use tokio::try_join;

use crate::{
    infra::{self},
    result::AppResult,
};

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
    let (this_month, unlock_status, pay_info) = try_join!(
        infra::spider::netflow::get_netflow_this_month(stu_id),
        infra::spider::netflow::get_netflow_unlock_status(stu_id),
        infra::spider::netflow::get_netflow_pay_info(stu_id)
    )?;
    let res = Netflow {
        thisMonth: NetflowResItem {
            download: this_month.downloadTraffic,
            upload: this_month.uploadTraffic,
            all: this_month.allTraffic,
            basePackageUsed: this_month.basePackageUsed,
            basePackageUsedPer: this_month.basePackageUsedPer,
            allBasePackageAmount: this_month.allBasePackageAmount,
            extendPackageUsed: this_month.extendPackageUsed,
            allExtendPackageAmount: this_month.allExtendPackageAmount,
            surplusBasePackage: this_month.surplusBasePackage,
            surplusExtendPackage: this_month.surplusExtendPackage,
            extendPackageUsedPer: this_month.extendPackageUsedPer,
        },
        payInfo: pay_info.Total,
        unlock: unlock_status.status,
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
        infra::spider::netflow::get_netflow_order(stu_id).await?;
    let mut res = Vec::with_capacity(spider_res.len());
    for item in spider_res {
        let temp = NetflowOrder {
            month: item.Month,
            shouldPay: item.ShouldPay,
            updateTime: item.UpdateTime,
            uploadName: item.UploadName,
            downloadName: item.DownloadName,
            realOverTraffic: item.RealOverTraffic,
        };
        res.push((
            parse_year_month(&temp.month)
                .ok_or(anyhow!("异常的年月字符串"))?,
            temp,
        ));
    }
    res.sort_by_key(|(month, _)| *month);
    res.reverse();
    Ok(res.into_iter().map(|(_, item)| item).collect())
}

pub use infra::spider::netflow::get_netflow_day_detail;
pub use infra::spider::netflow::get_netflow_month_detail;

/// 解析`%Y-%m`格式的字符串，将其转为当月的第一天。
fn parse_year_month(str: &str) -> Option<NaiveDate> {
    let mut parsed = Parsed::new();
    chrono::format::parse(
        &mut parsed,
        str,
        StrftimeItems::new("%Y-%m"),
    )
    .ok()?;
    parsed.set_day(1).ok()?;
    parsed.to_naive_date().ok()
}
#[cfg(test)]
mod test {
    use super::parse_year_month;
    #[test]
    fn test_parse_year_month() {
        assert_eq!(
            parse_year_month("2025-01").unwrap(),
            "2025-01-01".parse().unwrap()
        );
        assert_eq!(
            parse_year_month("2077-12").unwrap(),
            "2077-12-01".parse().unwrap()
        );
        assert_eq!(
            parse_year_month("2077-3").unwrap(),
            "2077-03-01".parse().unwrap()
        );
        assert_eq!(parse_year_month("2077-13"), None);
    }
}
