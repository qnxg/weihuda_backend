use serde::Serialize;

use crate::{infra, result::AppResult};

#[derive(Serialize, Debug)]
pub struct CardInfo {
    pub account: u32,
    pub balance: f64,
}
pub async fn get_card_info(stu_id: &str) -> AppResult<CardInfo> {
    let spider_res = infra::spider::pt::get_card_info(stu_id).await?;
    Ok(CardInfo {
        account: spider_res.account,
        balance: spider_res
            .balance
            .parse::<f64>()
            .expect("解析校园卡余额失败")
            / 100.0,
    })
}

#[derive(Serialize, Debug)]
#[expect(non_snake_case)]
pub struct CardHistory {
    pub TranCount: f64,
    pub total: f64,
    pub items: Vec<CardHistoryItem>,
}

#[derive(Serialize, Debug)]
#[expect(non_snake_case)]
pub struct CardHistoryItem {
    pub tranAmt: String,
    pub effectDate: String,
    pub jourDate: String,
    pub jourName: String,
    pub jourNum: u32,
    pub nowAmt: String,
    pub tranLocation: String,
    pub tranname: String,
}

pub async fn get_card_history(
    stu_id: &str,
    year: &str,
    month: &str,
    typ: &str,
) -> AppResult<CardHistory> {
    let spider_res =
        infra::spider::pt::get_card_history(stu_id, year, month, typ)
            .await?;
    let mut res_items = Vec::with_capacity(spider_res.items.len());
    for item in spider_res.items {
        let item = CardHistoryItem {
            tranAmt: item.fTranAmt,
            effectDate: item.effectdate,
            jourDate: item.jndatetime,
            jourName: item.jourName,
            jourNum: item.usedcardnum,
            nowAmt: item.nowAmt,
            tranLocation: item.sysname1.unwrap_or_default(),
            tranname: item.tranname,
        };
        res_items.push(item);
    }
    let res = CardHistory {
        TranCount: spider_res.TranCount,
        total: spider_res.total,
        items: res_items,
    };
    Ok(res)
}
