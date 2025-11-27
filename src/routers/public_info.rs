use crate::utils::serde::empty_string_as_none;
use salvo::{Request, Router, handler};
use serde::Deserialize;

use crate::{result::RouterResult, service, utils};

pub fn routers() -> Router {
    Router::with_path("hdjw/empty-room").get(get_empty_room)
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
struct GetEmptyRoomReq {
    /// 楼栋id
    pub buildId: String,
    /// 星期几
    pub day: u32,
    /// 节次
    pub jc: String,
    /// 周次
    pub week: u32,
    /// 学年
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub xn: Option<u32>,
    /// 学期
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub xq: Option<u32>,
}
#[handler]
async fn get_empty_room(req: &mut Request) -> RouterResult {
    let GetEmptyRoomReq {
        buildId,
        day,
        jc,
        week,
        xn,
        xq,
    } = req.parse_queries()?;
    let (_, stu_id) = utils::jwt::auth(req)?;
    let (current_xn, current_xq) =
        service::semester::get_now_xnxq().await?;
    let xn = xn.unwrap_or(current_xn);
    let xq = xq.unwrap_or(current_xq);
    let res = service::public_info::get_empty_room(
        &stu_id,
        &buildId,
        &day.to_string(),
        &jc,
        week,
        xn,
        xq,
    )
    .await?;
    Ok(res.into())
}
