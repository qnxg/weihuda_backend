use salvo::{Request, Router, handler, macros::Extractible};
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

#[handler]
async fn get_electricity(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct GetElectricityReq {
        pub refresh: u8,
    }
    let stu_id = utils::jwt::auth(req)?;
    let GetElectricityReq { refresh } = req.extract().await?;
    let res =
        service::electricity::get_electricity(&stu_id, refresh != 0)
            .await?;
    Ok(res.into())
}

#[handler]
async fn get_dormitory(req: &mut Request) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    let dormitory =
        service::user_info::get_dormitory(&stu_id).await?;
    Ok(dormitory.into())
}

#[handler]
async fn update_dormitory(req: &mut Request) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    service::user_info::update_dormitory(&stu_id).await?;
    Ok("更新成功".into())
}
