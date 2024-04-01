#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct SpiderEmptyRoom {
    pub items: Vec<SpiderEmptyRoomItem>,
    pub rowCount: u32,
}

/// 只解析需要的字段，就不全部解析了，其他字段写上也没用
#[derive(Deserialize, Debug)]
pub struct SpiderEmptyRoomItem {
    pub classroomtypename: String,
    pub js_name: String,
    pub kszw: u32,
    pub yxzw: u32,
    // pub bh: String,
    // pub classroomtypecode: String,
    // pub id: String,
    // pub jczy002id: String,
    // pub jczy008id: String,
    // pub jxl: String,
    // pub rnrs: u32,
    // pub rownum_: u32,
    // pub xqname: String,
}

#[derive(Serialize, Debug)]
pub struct EmptyRoomRes {
    pub name: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub seat: u32,
    pub examSeat: u32,
}
