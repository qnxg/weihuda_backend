use salvo::{Request, Router, handler, macros::Extractible};
use serde::Deserialize;

use crate::{result::RouterResult, service, utils};

pub fn routers() -> Router {
    Router::with_path("pt")
        .push(Router::with_path("card-info").get(get_card_info))
        .push(Router::with_path("card-history").get(get_card_history))
}

#[handler]
async fn get_card_info(req: &mut Request) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    let res = service::card::get_card_info(&stu_id).await?;
    Ok(res.into())
}

#[handler]
async fn get_card_history(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct GetCardHistoryReq {
        pub year: String,
        pub month: String,
        #[serde(rename = "type")]
        pub _type: String,
    }
    let GetCardHistoryReq {
        year,
        month,
        _type: typ,
    } = req.extract().await?;
    let stu_id = utils::jwt::auth(req)?;
    let res =
        service::card::get_card_history(&stu_id, &year, &month, &typ)
            .await?;
    Ok(res.into())
}
