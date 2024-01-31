#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct SpiderBorrowing {
    pub author: String,
    pub callno: String,
    pub isbn: String,
    pub loanDate: String,
    pub publisher: String,
    pub returnDate: String,
    pub title: String,
    pub totalRenewNum: u32,
}

#[derive(Serialize, Debug)]
pub struct BorrowingRes {
    pub author: String,
    pub id: String,
    pub isbn: String,
    pub borrowDate: String,
    pub publisher: String,
    pub returnDate: String,
    pub title: String,
    pub renew: u32,
}

#[derive(Deserialize, Debug)]
pub struct SpiderBorrowed {
    pub author: String,
    pub callno: String,
    pub isbn: String,
    pub logtype: String, // '30001' | '30002';
    pub publisher: String,
    pub time: String,
    pub title: String,
    pub totalRenewNum: u32,
}

#[derive(Serialize, Debug)]
pub struct BorrowedRes {
    pub author: String,
    pub id: String,
    pub isbn: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub publisher: String,
    pub time: String,
    pub title: String,
    pub renew: u32,
}

#[derive(Deserialize, Debug)]
pub struct SpiderFinance {
    pub barcode: String,
    pub bookTitle: String,
    pub bookrecno: u32,
    pub cost: f64,
    pub feetype: String,
    pub local: String,
    pub paySign: String,
    pub paytype: String,
    pub regtime: String,
    pub tranid: String,
}

#[derive(Serialize, Debug)]
pub struct FinanceRes {
    pub barcode: String,
    pub title: String,
    pub bookID: String,
    pub cost: f64,
    pub feeType: String,
    pub library: String,
    pub paySign: String,
    pub payType: String,
    pub time: String,
    pub tranID: String,
}

#[derive(Serialize, Debug)]
pub struct LibraryRes {
    pub borrowed: Vec<BorrowedRes>,
    pub borrowing: Vec<BorrowingRes>,
    pub finance: Vec<FinanceRes>,
}
