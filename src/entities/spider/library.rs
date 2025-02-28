#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Borrowing {
    /// 作者
    pub author: String,
    /// 借阅时间
    pub borrow_date: String,
    /// 图书的isbn
    pub isbn: String,
    /// 所在图书馆
    pub library: String,
    /// 应还时间
    pub return_date: String,
    /// 书名
    pub title: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Borrowed {
    /// 作者
    pub author: String,
    /// 图书的isbn
    pub isbn: String,
    /// 所在图书馆
    pub library: String,
    /// 操作时间
    pub time: String,
    /// 书名
    pub title: String,
    /// 操作类型，借书/还书
    #[serde(rename = "type")]
    pub borrowed_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Finance {
    /// 条型号
    pub barcode: String,
    /// 花费
    pub cost: String,
    /// 财经类型
    pub fee_type: String,
    /// 支付状态
    pub pay_sign: String,
    /// 发生地，发生馆-发生地点
    pub place: String,
    /// 时间
    pub time: String,
}

#[derive(Serialize, Debug)]
pub struct LibraryRes {
    pub borrowed: Vec<Borrowed>,
    pub borrowing: Vec<Borrowing>,
    pub finance: Vec<Finance>,
}
