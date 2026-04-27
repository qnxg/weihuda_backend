use crate::{result::AppError, utils::serde::empty_string_as_none};
use salvo::{Request, Router, handler, macros::Extractible};
use serde::Deserialize;

use crate::{result::RouterResult, service, utils};

pub fn routers() -> Router {
    Router::with_path("hdjw/empty-room").get(get_empty_room)
}

#[handler]
async fn get_empty_room(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(
        default_source(from = "query"),
        rename_all = "camelCase"
    ))]
    struct GetEmptyRoomReq {
        /// 楼栋id
        pub build_id: String,
        /// 星期几
        pub day: u8,
        /// 节次
        pub jc: String,
        /// 周次
        pub week: u8,
        /// 学年
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub xn: Option<u32>,
        /// 学期
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub xq: Option<u32>,
    }
    let query: GetEmptyRoomReq = req.extract().await?;
    let stu_id = utils::jwt::auth(req)?;
    let (current_xn, current_xq) =
        service::semester::get_now_xnxq().await?;
    let xn = query.xn.unwrap_or(current_xn) as u16;
    let xq = query.xq.unwrap_or(current_xq) as u8;
    let jc = query
        .jc
        .split(',')
        .map(|time| match time {
            "0102" => Ok(1),
            "0304" => Ok(2),
            "0506" => Ok(3),
            "0708" => Ok(4),
            "091011" => Ok(5),
            _ => Err(AppError::ParseError),
        })
        .collect::<Result<Vec<u8>, AppError>>()?;
    let res = service::public_info::get_empty_room(
        &stu_id,
        &query.build_id,
        query.day,
        jc,
        query.week,
        xn,
        xq,
    )
    .await?;
    Ok(res.into())
}
