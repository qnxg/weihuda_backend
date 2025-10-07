use axum::Extension;
use chrono::{
    format::{Parsed, StrftimeItems},
    NaiveDate,
};
use tokio::try_join;

use crate::{
    app_result::AppResult,
    dtos::spider::netflow::{
        GetNetflowDayDetailReq, GetNetflowMonthDetailReq,
    },
    entities::spider::netflow::{
        NetflowOrderRes, NetflowRes, NetflowResItem,
        SpiderNetflowMonthDetail, SpiderNetflowOrder,
        SpiderNetflowPayInfo, SpiderNetflowThisMonth,
        SpiderNetflowUnlockStatus,
    },
    extractors::Query,
    utils::{jwt::parse_stu_id, request::spider_data},
};

pub async fn get_netflow_handler(
    Extension(token): Extension<String>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let params = [("stuid", stu_id)];

    // let spider_res_this_month: SpiderNetflowThisMonth = spider_data("/netflow", &params).await?;
    // let spider_res_unlock_status: SpiderNetflowUnlockStatus =
    //     spider_data("/netflow/unlock", &params).await?;
    // let spider_res_pay_info: SpiderNetflowPayInfo =
    //     spider_data("/netflow/pay_info", &params).await?;

    // 三个请求并发
    let (
        spider_res_this_month,
        spider_res_unlock_status,
        spider_res_pay_info,
    ): (
        SpiderNetflowThisMonth,
        SpiderNetflowUnlockStatus,
        SpiderNetflowPayInfo,
    ) = try_join!(
        spider_data("/netflow", &params),
        spider_data("/netflow/unlock", &params),
        spider_data("/netflow/pay_info", &params)
    )?;

    let res = NetflowRes {
        thisMonth: NetflowResItem {
            download: spider_res_this_month.downloadTraffic,
            upload: spider_res_this_month.uploadTraffic,
            all: spider_res_this_month.allTraffic,
            basePackageUsed: spider_res_this_month.basePackageUsed,
            basePackageUsedPer: spider_res_this_month
                .basePackageUsedPer,
            allBasePackageAmount: spider_res_this_month
                .allBasePackageAmount,
            extendPackageUsed: spider_res_this_month
                .extendPackageUsed,
            allExtendPackageAmount: spider_res_this_month
                .allExtendPackageAmount,
            surplusBasePackage: spider_res_this_month
                .surplusBasePackage,
            surplusExtendPackage: spider_res_this_month
                .surplusExtendPackage,
            extendPackageUsedPer: spider_res_this_month
                .extendPackageUsedPer,
        },
        payInfo: spider_res_pay_info.Total,
        unlock: spider_res_unlock_status.status,
    };
    Ok(res.into())
}

pub async fn get_netflow_order_handler(
    Extension(token): Extension<String>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let params = [("stuid", stu_id)];
    let spider_res: Vec<SpiderNetflowOrder> =
        spider_data("/netflow/order", &params).await?;

    let mut res = Vec::with_capacity(spider_res.len());
    for item in spider_res {
        let temp = NetflowOrderRes {
            month: item.Month,
            shouldPay: item.ShouldPay,
            updateTime: item.UpdateTime,
            uploadName: item.UploadName,
            downloadName: item.DownloadName,
            realOverTraffic: item.RealOverTraffic,
        };
        res.push(temp);
    }
    res.sort_by_key(|item| {
        parse_year_month(&item.month).expect("异常的年月字符串")
    });
    res.reverse();
    Ok(res.into())
}

pub async fn get_netflow_month_detail_handler(
    Query(req): Query<GetNetflowMonthDetailReq>,
    Extension(token): Extension<String>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let params =
        [("stuid", stu_id), ("year", req.year), ("month", req.month)];
    let spider_res: SpiderNetflowMonthDetail =
        spider_data("/netflow/month_detail", &params).await?;

    Ok(spider_res.into())
}

pub async fn get_netflow_day_detail_handler(
    Query(req): Query<GetNetflowDayDetailReq>,
    Extension(token): Extension<String>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let params = [
        ("stuid", stu_id),
        ("year", req.year),
        ("month", req.month),
        ("day", req.day),
    ];
    let spider_res: SpiderNetflowMonthDetail =
        spider_data("/netflow/day_detail", &params).await?; // 直接复用月流量明细的结构体
    Ok(spider_res.into())
}

/// 解析`%Y-%m`格式的字符串，将其转为当月的第一天。
pub fn parse_year_month(str: &str) -> Option<NaiveDate> {
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
    use crate::handlers::spider::netflow::parse_year_month;

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
