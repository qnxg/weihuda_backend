use salvo::{Request, Router, handler};
use serde::Deserialize;

use crate::{result::RouterResult, service, utils};

pub fn routers() -> Router {
    Router::with_path("pt")
        .push(Router::with_path("card-info").get(get_card_info))
        .push(Router::with_path("card-history").get(get_card_history))
}

#[handler]
async fn get_card_info(req: &mut Request) -> RouterResult {
    let (_, stu_id) = utils::jwt::auth(req)?;
    let res = service::card::get_card_info(&stu_id).await?;
    Ok(res.into())
}

#[derive(Deserialize, Debug)]
struct GetCardHistoryReq {
    pub year: String,
    pub month: String,
    #[serde(rename = "type")]
    pub _type: String,
}
#[handler]
async fn get_card_history(req: &mut Request) -> RouterResult {
    let (_, stu_id) = utils::jwt::auth(req)?;
    let GetCardHistoryReq {
        year,
        month,
        _type: typ,
    } = req.parse_queries::<GetCardHistoryReq>()?;
    let res =
        service::card::get_card_history(&stu_id, &year, &month, &typ)
            .await?;
    Ok(res.into())
}
