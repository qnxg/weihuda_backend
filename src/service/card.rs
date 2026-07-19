use crate::{
    error::AppResult,
    service::user_state::{Pt, with_token},
};
use serde::Serialize;

pub use hnu_query::pt::card::CardHistoryType;

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
    year: u16,
    month: u8,
    history_type: CardHistoryType,
) -> AppResult<CardHistory> {
    let spider_res =
        with_token(Pt::new(stu_id), |token| async move {
            hnu_query::pt::get_card_history(
                &token,
                year,
                month,
                history_type,
            )
            .await
        })
        .await?;
    let mut res_items = Vec::with_capacity(spider_res.items.len());
    for item in spider_res.items {
        let date_time =
            item.date_time.format("%Y-%m-%d %H:%M:%S").to_string();
        let journal_time =
            item.journal_time.format("%Y-%m-%d %H:%M:%S").to_string();
        let item = CardHistoryItem {
            tranAmt: format!("{:.2}", item.amount),
            effectDate: date_time,
            jourDate: journal_time,
            jourName: item.status,
            jourNum: item.id,
            nowAmt: format!("{:.2}", item.now_balance),
            tranLocation: item.location.unwrap_or("未知".to_string()),
            tranname: item.name,
        };
        res_items.push(item);
    }
    let res = CardHistory {
        TranCount: spider_res.count as f64,
        total: spider_res.total,
        items: res_items,
    };
    Ok(res)
}

#[derive(Serialize, Debug)]
pub struct CardInfo {
    pub account: u32,
    pub balance: f64,
}

pub async fn get_card_info(stu_id: &str) -> AppResult<CardInfo> {
    let spider_res =
        with_token(Pt::new(stu_id), async move |token| {
            hnu_query::pt::get_card_info(&token).await
        })
        .await?;
    let res = CardInfo {
        account: spider_res.id,
        balance: spider_res.balance,
    };
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TEST_STU_ID;

    #[tokio::test]
    async fn test_get_card_info() {
        let card_info = get_card_info(&TEST_STU_ID).await.unwrap();
        println!("{:#?}", card_info);
    }
}
