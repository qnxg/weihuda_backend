use salvo::prelude::Extractible;
use serde::Deserialize;

#[derive(Deserialize, Extractible, Debug)]
pub struct CardHistoryReq {
    pub stuid: String,
    pub year: String,
    pub month: String,
    #[serde(rename = "type")]
    pub _type: String,
}
