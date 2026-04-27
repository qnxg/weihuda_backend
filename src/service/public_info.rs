use crate::{
    result::AppResult,
    service::user_state::{Hdjw, with_token},
};
use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EmptyRoom {
    pub name: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub seat: u32,
    pub exam_seat: u32,
}

pub async fn get_empty_room(
    stu_id: &str,
    build_id: &str,
    day: u8,
    jc: Vec<u8>,
    week: u8,
    xn: u16,
    xq: u8,
) -> AppResult<Vec<EmptyRoom>> {
    let build_id_value = build_id.to_string();
    let spider_res =
        with_token(Hdjw::new(stu_id), async move |token| {
            hnu_query::hdjw::get_empty_classroom(
                &token,
                build_id_value.as_str(),
                week,
                day,
                jc.as_slice(),
                xn,
                xq,
            )
            .await
        })
        .await?;
    let mut res = Vec::new();
    for item in spider_res {
        let temp = EmptyRoom {
            name: item.room_name,
            seat: item.seat_count,
            exam_seat: item.exam_seat_count,
            _type: item.room_type,
        };
        res.push(temp);
    }
    Ok(res)
}
