#![allow(non_snake_case)]
use serde::Serialize;
use sqlx::FromRow;

#[derive(FromRow, Serialize, Debug)]
pub struct UserBind {
    pub openid: String,
    pub stuID: String,
    pub stuPASS: String,
    pub hdjwPASS: String,
}
