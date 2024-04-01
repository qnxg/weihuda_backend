#![allow(non_snake_case)]
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct MiniBind {
    pub openid: Option<String>,
    pub stuID: Option<String>,
    pub stuPASS: Option<String>,
    pub hdjwPASS: Option<String>,
    pub id: u32,
}
