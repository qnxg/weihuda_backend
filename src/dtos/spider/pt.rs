#![allow(non_snake_case)]
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct GetCardHistoryReq {
    pub year: String,
    pub month: String,
    #[serde(rename = "type")]
    pub _type: String,
}

#[derive(Deserialize, Debug)]
pub struct GetLabGradeReq {
    pub labId: String,
}

#[derive(Deserialize, Debug)]
pub struct GetFitnessReq {
    pub xn: String,
}
