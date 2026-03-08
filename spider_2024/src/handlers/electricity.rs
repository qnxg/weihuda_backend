use crate::{
    app_result::HandlerResult,
    dtos::electricity::GetElectricityReq,
    spiders,
    utils::redis::{add_cookie_to_redis, get_cookie_from_redis},
};
use salvo::{Request, handler};

const CACHE_TIMEOUT: i64 = 60 * 60 * 16;

#[handler]
pub async fn get_electricity_handler(
    req: &mut Request,
) -> HandlerResult {
    let param: GetElectricityReq = req.parse_queries()?;
    let key = format!(
        "e{}{}{}",
        param.park.clone(),
        param.build.clone(),
        param.room.clone()
    );
    let mut res = get_cookie_from_redis(key.as_str(), "").await;
    if res.is_err() || param.refresh.unwrap_or(0) == 1 {
        let t = spiders::electricity::get_electricity(
            param.park,
            param.build,
            param.room,
        )
        .await?;
        add_cookie_to_redis(
            key.as_str(),
            t.as_str(),
            "",
            CACHE_TIMEOUT,
        )
        .await?;
        res = Ok(t);
    }
    let res = res?;
    Ok(res.into())
}
