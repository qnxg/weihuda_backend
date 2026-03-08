use salvo::prelude::Extractible;
use serde::Deserialize;

#[derive(Deserialize, Extractible, Debug)]
pub struct NetflowMonthDetailReq {
    pub stuid: String,
    pub year: String,
    pub month: String,
}

#[derive(Deserialize, Extractible, Debug)]
pub struct NetflowDayDetailReq {
    pub stuid: String,
    pub year: String,
    pub month: String,
    pub day: String,
}
