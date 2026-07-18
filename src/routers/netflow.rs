use chrono::{Datelike, NaiveDate};
use rand::{Rng, SeedableRng, rngs::StdRng};
use salvo::{Request, Router, handler, macros::Extractible};
use serde::Deserialize;

use crate::{
    error::RouterResult,
    routers::{ThrowParseError, demo::DEMO_STU_ID},
    service::{
        self,
        netflow::{
            Netflow, NetflowDetailItemRes, NetflowDetailRes,
            NetflowOrder, NetflowResItem,
        },
    },
    utils,
};

pub fn routers() -> Router {
    Router::with_path("netflow")
        .push(Router::with_path("order").get(get_netflow_order))
        .push(
            Router::with_path("month-detail")
                .get(get_netflow_month_detail),
        )
        .push(
            Router::with_path("day-detail")
                .get(get_netflow_day_detail),
        )
        .get(get_netflow_info)
}

#[handler]
async fn get_netflow_info(req: &mut Request) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    if stu_id == DEMO_STU_ID {
        return Ok(Netflow {
            thisMonth: NetflowResItem {
                download: "12.1G".to_string(),
                upload: "978.1M".to_string(),
                all: "13.1G".to_string(),
                allBasePackageAmount: 40.0,
                allExtendPackageAmount: 0.0,
                basePackageUsed: 13.1,
                basePackageUsedPer: 0.3275,
                surplusBasePackage: 0.0,
                extendPackageUsed: 0.0,
                extendPackageUsedPer: 0.0,
                surplusExtendPackage: 0.0,
            },
            unlock: "未锁定".to_string(),
            payInfo: 0.0,
        }
        .into());
    }

    let res = service::netflow::get_netflow_info(&stu_id).await?;
    Ok(res.into())
}

fn mock_netflow_order(date: NaiveDate) -> Option<NetflowOrder> {
    let seed = (date.year() as u32) * 10 + date.month();
    let mut rng = StdRng::seed_from_u64(seed.into());

    let updated_datetime = date.with_day(1)?.and_hms_opt(
        rng.gen_range(0..=6),
        rng.gen_range(0..=15),
        rng.gen_range(0..60),
    )?;

    Some(NetflowOrder {
        month: date.format("%Y-%m").to_string(),
        shouldPay: 0.0,
        updateTime: updated_datetime
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
        uploadName: format!("{:.1} GB", rng.gen_range(0.0..=1.0)),
        downloadName: format!("{:.1} GB", rng.gen_range(0.0..=10.0)),
        realOverTraffic: 0.0,
    })
}

fn mock_netflow_order_list() -> Vec<NetflowOrder> {
    // 只模拟前四个月（不含本月）的数据
    let now = chrono::Local::now().date_naive();

    (1..=4)
        .filter_map(|months_to_sub| {
            // 获取减去相应月数后的日期
            let date = now.checked_sub_months(
                chrono::Months::new(months_to_sub),
            )?;

            mock_netflow_order(date)
        })
        .collect()
}

#[handler]
async fn get_netflow_order(req: &mut Request) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    if stu_id == DEMO_STU_ID {
        return Ok(mock_netflow_order_list().into());
    }

    let res = service::netflow::get_netflow_order(&stu_id).await?;
    Ok(res.into())
}

#[handler]
async fn get_netflow_month_detail(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct GetNetflowMonthDetailReq {
        pub year: u16,
        pub month: u8,
    }
    let GetNetflowMonthDetailReq { year, month } =
        req.extract().await.parse_error()?;
    let stu_id = utils::jwt::auth(req)?;

    if stu_id == DEMO_STU_ID {
        let download = 1.5 * 1024.0 * 1024.0;
        let upload = 0.1 * 1024.0 * 1024.0;
        let total = 1.6 * 1024.0 * 1024.0;

        return Ok(NetflowDetailRes {
            AllDownload: download,
            AllUpload: upload,
            AllTotal: total,
            FloatDetailList: vec![NetflowDetailItemRes {
                App: "其他/其他流量".to_string(),
                Download: download,
                Per: 1.0,
                Total: total,
                Upload: upload,
            }],
        }
        .into());
    }

    let res = service::netflow::get_netflow_month_detail(
        &stu_id, year, month,
    )
    .await?;
    Ok(res.into())
}

#[handler]
async fn get_netflow_day_detail(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct GetNetflowDayDetailReq {
        pub year: u16,
        pub month: u8,
        pub day: u8,
    }
    let GetNetflowDayDetailReq { year, month, day } =
        req.extract().await.parse_error()?;
    let stu_id = utils::jwt::auth(req)?;
    let res = service::netflow::get_netflow_day_detail(
        &stu_id, year, month, day,
    )
    .await?;
    Ok(res.into())
}
