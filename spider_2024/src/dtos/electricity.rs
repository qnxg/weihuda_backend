use salvo::macros::Extractible;
use serde::Deserialize;

#[derive(Deserialize, Extractible, Debug)]
pub struct GetElectricityReq {
    pub park: u8,
    pub build: String,
    pub room: String,
    pub refresh: Option<u8>,
}
