use salvo::{Request, Router, handler};
use serde::Deserialize;

use crate::{result::RouterResult, service, utils};

pub fn routers() -> Router {
    Router::new()
        .push(Router::with_path("electricity").get(get_electricity))
        .push(
            Router::with_path("dormitory")
                .push(Router::with_path("query").get(get_dormitory))
                .push(
                    Router::with_path("update").get(update_dormitory),
                ),
        )
}

#[derive(Deserialize, Debug)]
struct GetElectricityReq {
    pub refresh: u8,
}
#[handler]
async fn get_electricity(req: &mut Request) -> RouterResult {
    let (_, stu_id) = utils::jwt::auth(req)?;
    let GetElectricityReq { refresh } = req.parse_queries()?;
    let res =
        service::electricity::get_electricity(&stu_id, refresh != 0)
            .await?;
    Ok(res.into())
}

#[handler]
async fn get_dormitory(req: &mut Request) -> RouterResult {
    let (_, stu_id) = utils::jwt::auth(req)?;
    let dormitory =
        service::user_info::get_dormitory(&stu_id).await?;
    Ok(dormitory.into())
}

#[handler]
async fn update_dormitory(req: &mut Request) -> RouterResult {
    let (_, stu_id) = utils::jwt::auth(req)?;
    service::user_info::update_dormitory(&stu_id).await?;
    Ok("更新成功".into())
}
