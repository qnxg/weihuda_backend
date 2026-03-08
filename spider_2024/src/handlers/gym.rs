use crate::{app_result::HandlerResult, dtos::gym::GymReq, spiders};
use anyhow::anyhow;
use salvo::{Request, handler};

#[handler]
pub async fn get_gym_grade_handler(
    req: &mut Request,
) -> HandlerResult {
    let req: GymReq = req.parse_queries()?;
    let res = spiders::gym::get_data(&req.stuid, req.xn).await?;
    let res_data = &res["data"];
    if res_data.is_object() {
        Ok(res_data.into())
    } else {
        return Err(anyhow!("意料之外的体测平台数据：{res}").into());
    }
}

#[handler]
pub async fn get_gym_raw_grade_handler(
    req: &mut Request,
) -> HandlerResult {
    let req: GymReq = req.parse_queries()?;
    let res = spiders::gym::get_raw_data(&req.stuid, req.xn).await?;
    let res_data = &res["data"];
    if res_data.is_object() {
        Ok(res_data.into())
    } else {
        return Err(anyhow!("意料之外的体测平台数据：{res}").into());
    }
}

#[handler]
pub async fn get_gym_appoint_handler(
    req: &mut Request,
) -> HandlerResult {
    let stuid = req
        .query::<String>("stuid")
        .ok_or(anyhow!("stuid is required"))?;
    let res = spiders::gym::get_appoint(&stuid).await?;
    let res_data = &res["data"];
    if res_data.is_array() {
        Ok(res_data.into())
    } else {
        return Err(anyhow!("意料之外的体测平台数据：{res}").into());
    }
}

#[handler]
pub async fn get_gym_appoint_detail_handler(
    req: &mut Request,
) -> HandlerResult {
    let stuid = req
        .query::<String>("stuid")
        .ok_or(anyhow!("stuid is required"))?;
    let class_id = req
        .query::<String>("class_id")
        .ok_or(anyhow!("class_id is required"))?;
    let class_time = req
        .query::<String>("class_time")
        .ok_or(anyhow!("class_time is required"))?;
    let test_time = req
        .query::<String>("test_time")
        .ok_or(anyhow!("test_time is required"))?;
    let res = spiders::gym::get_appoint_detail(
        &stuid,
        &class_id,
        &class_time,
        &test_time,
    )
    .await?;
    let res_data = &res["data"];
    if res_data.is_object() {
        Ok(res_data.into())
    } else {
        return Err(anyhow!("意料之外的体测平台数据：{res}").into());
    }
}
