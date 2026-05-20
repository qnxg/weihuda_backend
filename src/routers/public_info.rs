use crate::{
    result::AppError, routers::demo::DEMO_STU_ID,
    service::public_info::EmptyRoom,
    utils::serde::empty_string_as_none,
};
use rand::{Rng, SeedableRng, rngs::StdRng};
use salvo::{Request, Router, handler, macros::Extractible};
use serde::Deserialize;

use crate::{result::RouterResult, service, utils};

pub fn routers() -> Router {
    Router::with_path("hdjw/empty-room").get(get_empty_room)
}

fn mock_empty_rooms(
    week: u8,
    day: u8,
    jc: Vec<u8>,
) -> Vec<EmptyRoom> {
    let seed = (week as u64) * 1000 + (day as u64) * 100;
    let mut rng = StdRng::seed_from_u64(seed);

    let room_nums = (101..=122).chain(201..=221);

    room_nums
        .filter_map(|room_num| {
            // 明显时间跨度越大，可用的空教室就越少
            let probability = 0.7 - jc.len() as f64 * 0.1;

            if rng.gen_bool(probability) {
                let seat = rng.gen_range(3..=6) * 10;
                Some(EmptyRoom {
                    name: room_num.to_string(),
                    _type: "多媒体教室".to_string(),
                    seat,
                    exam_seat: seat / 3,
                })
            } else {
                None
            }
        })
        .collect()
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

    if stu_id == DEMO_STU_ID {
        return Ok(mock_empty_rooms(query.week, query.day, jc).into());
    }

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
