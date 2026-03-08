use crate::{app_result::HandlerResult, spiders};
use anyhow::anyhow;
use salvo::{Request, handler};

#[handler]
pub async fn get_current_loan(req: &mut Request) -> HandlerResult {
    let stuid = req
        .query::<String>("stuid")
        .ok_or(anyhow!("stuid is required"))?;
    let res = spiders::library::get_current_list(&stuid).await?;
    Ok(res.into())
}

#[handler]
pub async fn get_history_loan(req: &mut Request) -> HandlerResult {
    let stuid = req
        .query::<String>("stuid")
        .ok_or(anyhow!("stuid is required"))?;
    let res = spiders::library::get_history_list(&stuid).await?;
    Ok(res.into())
}

#[handler]
pub async fn get_finance(req: &mut Request) -> HandlerResult {
    let stuid = req
        .query::<String>("stuid")
        .ok_or(anyhow!("stuid is required"))?;
    let res = spiders::library::get_finance_list(&stuid).await?;
    Ok(res.into())
}
