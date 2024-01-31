#![allow(non_snake_case)]
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct BindReq {
    pub code: String,
    pub stuId: String,
    pub stuPassword: String,
    pub hdjwPassword: String,
    // pub mode: u32,
    // pub platform: u32,
}

#[derive(Deserialize, Debug)]

pub struct VerifyResult {
    pub code: u32,
    pub status: String,
    pub message: String,
}

#[derive(Deserialize, Debug)]

pub struct CryptoResult {
    pub data: CryptoResultData,
    pub errorMessage: String,
    pub errorCode: String,
}

#[derive(Deserialize, Debug)]

pub struct CryptoResultData {
    pub hdjw_encrypted: String,
    pub pt_encrypted: String,
}
