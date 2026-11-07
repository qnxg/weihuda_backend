use crate::{
    error::{AppError, AppResult, ThrowInternalErrorResult},
    service::user_state::{IPortal, with_token},
};
use chrono::NaiveDate;
use hnu_query::iportal::card::{TransactionRecord, TransactionType};
use serde::Serialize;
use std::cmp::Reverse;

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
    pub nowAmt: String,
    pub tranLocation: String,
    pub tranname: String,
}

impl From<TransactionRecord> for CardHistoryItem {
    fn from(item: TransactionRecord) -> Self {
        Self {
            tranAmt: format!(
                "{:.2}",
                item.transaction_amount as f64 / 100.0
            ),
            effectDate: item
                .pay_time
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            nowAmt: format!(
                "{:.2}",
                item.card_balance as f64 / 100.0
            ),
            tranLocation: item.merchant_name,
            tranname: item.transaction_name,
        }
    }
}

const TRANSACTION_PAGE_SIZE: u32 = 50;
const TRANSACTION_MAX_PAGE: u32 = 5;

fn month_range(
    year: u16,
    month: u8,
) -> AppResult<(NaiveDate, NaiveDate)> {
    let start =
        NaiveDate::from_ymd_opt(i32::from(year), u32::from(month), 1)
            .ok_or_else(|| AppError::customized("无效的年月"))?;
    let (next_year, next_month) = if month == 12 {
        (i32::from(year) + 1, 1)
    } else {
        (i32::from(year), u32::from(month) + 1)
    };
    let end = NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .and_then(|date| date.pred_opt())
        .ok_or_else(|| AppError::customized("无效的年月"))?;
    Ok((start, end))
}

pub async fn get_card_history(
    stu_id: &str,
    year: u16,
    month: u8,
    history_type: CardHistoryType,
) -> AppResult<CardHistory> {
    let (start, end) = month_range(year, month)?;
    let mut records =
        with_token(IPortal::new(stu_id), |token| async move {
            let card =
                hnu_query::iportal::get_card_info(&token).await?;
            let mut records = Vec::new();
            for page in 1..=TRANSACTION_MAX_PAGE {
                let details =
                    hnu_query::iportal::get_card_transaction_records(
                        &token,
                        card.account.clone(),
                        start,
                        end,
                        TRANSACTION_PAGE_SIZE,
                        page,
                    )
                    .await?;
                let has_next_page = details.has_next_page;
                records.extend(details.records);
                if !has_next_page {
                    break;
                }
            }
            Ok(records)
        })
        .await?;

    let transaction_type = match history_type {
        CardHistoryType::Consumption => TransactionType::Consumption,
        CardHistoryType::Recharge => TransactionType::Recharge,
    };
    records.retain(|item| item.transaction_type == transaction_type);
    records.sort_by_key(|item| Reverse(item.pay_time));

    let total_cents: i64 =
        records.iter().map(|item| item.transaction_amount).sum();
    let res_items = records
        .into_iter()
        .map(CardHistoryItem::from)
        .collect::<Vec<_>>();
    let res = CardHistory {
        TranCount: res_items.len() as f64,
        total: total_cents as f64 / 100.0,
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
        with_token(IPortal::new(stu_id), async move |token| {
            hnu_query::iportal::get_card_info(&token).await
        })
        .await?;
    let res = CardInfo {
        account: spider_res.account.parse::<u32>().internal_err()?,
        balance: spider_res.balance,
    };
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TEST_STU_ID;

    #[test]
    fn test_month_range() {
        assert_eq!(
            month_range(2026, 11).unwrap(),
            (
                NaiveDate::from_ymd_opt(2026, 11, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 11, 30).unwrap(),
            )
        );
        assert_eq!(
            month_range(2026, 12).unwrap(),
            (
                NaiveDate::from_ymd_opt(2026, 12, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
            )
        );
        assert!(month_range(2026, 13).is_err());
    }

    #[test]
    fn test_transaction_record_mapping() {
        let item = CardHistoryItem::from(TransactionRecord {
            transaction_name: "持卡人消费".to_string(),
            merchant_name: "天马二食堂二楼".to_string(),
            merchant_account: 1000006,
            pay_time: NaiveDate::from_ymd_opt(2026, 9, 13)
                .unwrap()
                .and_hms_opt(11, 47, 30)
                .unwrap(),
            transaction_amount: -200,
            card_balance: 8181,
            transaction_type: TransactionType::Consumption,
        });

        assert_eq!(item.tranAmt, "-2.00");
        assert_eq!(item.effectDate, "2026-09-13 11:47:30");
        assert_eq!(item.nowAmt, "81.81");
        assert_eq!(item.tranLocation, "天马二食堂二楼");
        assert_eq!(item.tranname, "持卡人消费");
    }

    #[tokio::test]
    async fn test_get_card_info() {
        let card_info = get_card_info(&TEST_STU_ID).await.unwrap();
        println!("{:#?}", card_info);
    }
}
