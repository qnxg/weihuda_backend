use crate::result::AppResult;
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
    let spider_res = spider_2024::hdjw::get_empty_classroom(
        stu_id,
        build_id,
        week,
        day,
        jc.as_slice(),
        xn,
        xq,
    )
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
