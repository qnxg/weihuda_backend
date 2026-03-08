use salvo::prelude::Extractible;
use serde::Deserialize;

#[derive(Deserialize, Extractible, Debug)]
pub struct GymReq {
    pub stuid: String,
    pub xn: u16,
}
