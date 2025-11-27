use salvo::{Request, Router, handler};
use serde::Deserialize;

use crate::{result::RouterResult, service, utils};

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
    let (_, stu_id) = utils::jwt::auth(req)?;
    let res = service::netflow::get_netflow_info(&stu_id).await?;
    Ok(res.into())
}

#[handler]
async fn get_netflow_order(req: &mut Request) -> RouterResult {
    let (_, stu_id) = utils::jwt::auth(req)?;
    let res = service::netflow::get_netflow_order(&stu_id).await?;
    Ok(res.into())
}

#[derive(Deserialize, Debug)]
struct GetNetflowMonthDetailReq {
    pub year: String,
    pub month: String,
}
#[handler]
async fn get_netflow_month_detail(req: &mut Request) -> RouterResult {
    let GetNetflowMonthDetailReq { year, month } =
        req.parse_queries()?;
    let (_, stu_id) = utils::jwt::auth(req)?;
    let res = service::netflow::get_netflow_month_detail(
        &stu_id, &year, &month,
    )
    .await?;
    Ok(res.into())
}

#[derive(Deserialize, Debug)]
struct GetNetflowDayDetailReq {
    pub year: String,
    pub month: String,
    pub day: String,
}
#[handler]
async fn get_netflow_day_detail(req: &mut Request) -> RouterResult {
    let GetNetflowDayDetailReq { year, month, day } =
        req.parse_queries()?;
    let (_, stu_id) = utils::jwt::auth(req)?;
    let res = service::netflow::get_netflow_day_detail(
        &stu_id, &year, &month, &day,
    )
    .await?;
    Ok(res.into())
}
