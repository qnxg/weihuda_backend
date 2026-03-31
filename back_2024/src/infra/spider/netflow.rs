use spider_2024::dtos::netflow::{
    NetflowDayDetailReq, NetflowDetailRes, NetflowMonthDetailReq,
    NetflowOrderReturnItem, NetflowPayInfoRes, NetflowThisMonthRes,
    NetflowUnlockStatusRes,
};

use crate::result::AppResult;

pub async fn get_netflow_this_month(
    stu_id: &str,
) -> AppResult<NetflowThisMonthRes> {
    let spider_res =
        spider_2024::netflow::get_netflow_handler(stu_id).await?;
    Ok(spider_res)
}

pub async fn get_netflow_unlock_status(
    stu_id: &str,
) -> AppResult<NetflowUnlockStatusRes> {
    let spider_res =
        spider_2024::netflow::get_unlock_status_handler(stu_id)
            .await?;
    Ok(spider_res)
}

pub async fn get_netflow_pay_info(
    stu_id: &str,
) -> AppResult<NetflowPayInfoRes> {
    let spider_res =
        spider_2024::netflow::get_netflow_pay_info_handler(stu_id)
            .await?;
    Ok(spider_res)
}

pub async fn get_netflow_order(
    stu_id: &str,
) -> AppResult<Vec<NetflowOrderReturnItem>> {
    let spider_res =
        spider_2024::netflow::get_netflow_order_handler(stu_id)
            .await?;
    Ok(spider_res)
}

pub async fn get_netflow_month_detail(
    stu_id: &str,
    year: &str,
    month: &str,
) -> AppResult<NetflowDetailRes> {
    let spider_res =
        spider_2024::netflow::get_netflow_month_detail_handler(
            NetflowMonthDetailReq {
                stu_id: stu_id.to_string(),
                year: year.to_string(),
                month: month.to_string(),
            },
        )
        .await?;
    Ok(spider_res)
}

pub async fn get_netflow_day_detail(
    stu_id: &str,
    year: &str,
    month: &str,
    day: &str,
) -> AppResult<NetflowDetailRes> {
    let spider_res =
        spider_2024::netflow::get_netflow_day_detail_handler(
            NetflowDayDetailReq {
                stu_id: stu_id.to_string(),
                year: year.to_string(),
                month: month.to_string(),
                day: day.to_string(),
            },
        )
        .await?;
    Ok(spider_res)
}
