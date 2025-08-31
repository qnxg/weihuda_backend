#![allow(non_snake_case)]
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct EmptyRoomRes {
    pub name: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub seat: u32,
    pub examSeat: u32,
}
