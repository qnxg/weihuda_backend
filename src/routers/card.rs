use crate::routers::demo::DEMO_STU_ID;
use crate::service::card::{CardHistory, CardHistoryType, CardInfo};
use crate::{result::RouterResult, service, utils};
use salvo::{Request, Router, handler, macros::Extractible};
use serde::Deserialize;

pub fn routers() -> Router {
    Router::with_path("pt")
        .push(Router::with_path("card-info").get(get_card_info))
        .push(Router::with_path("card-history").get(get_card_history))
}

#[handler]
async fn get_card_info(req: &mut Request) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    if stu_id == DEMO_STU_ID {
        return Ok(CardInfo {
            account: 368246,
            balance: 84.0,
        }
        .into());
    }

    let res = service::card::get_card_info(&stu_id).await?;
    Ok(res.into())
}

#[handler]
async fn get_card_history(req: &mut Request) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    if stu_id == DEMO_STU_ID {
        return Ok(CardHistory {
            TranCount: 0.0,
            total: 0.0,
            items: vec![],
        }
        .into());
    }

    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct GetCardHistoryReq {
        pub year: u16,
        pub month: u8,
        #[serde(rename = "type")]
        pub _type: String,
    }
    let GetCardHistoryReq {
        year,
        month,
        _type: typ,
    } = req.extract().await?;
    let history_type = match typ.as_str() {
        "1" => CardHistoryType::Consumption,
        _ => CardHistoryType::Recharge,
    };
    let res = service::card::get_card_history(
        &stu_id,
        year,
        month,
        history_type,
    )
    .await?;
    Ok(res.into())
}
