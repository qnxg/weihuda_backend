#![allow(non_snake_case)]
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SpiderEmail {
    pub unReadCount: Option<u32>,
}
