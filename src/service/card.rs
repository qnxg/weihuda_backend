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

/// 判断是否属于 消费/充值 账单
fn matches_history_type(
    transaction_type: TransactionType,
    transaction_amount: i64,
    history_type: CardHistoryType,
) -> bool {
    if matches!(
        transaction_type,
        TransactionType::EAccountOpening
            | TransactionType::HolderAccountOpening
    ) {
        return false;
    }

    match history_type {
        CardHistoryType::Consumption => transaction_amount < 0,
        CardHistoryType::Recharge => transaction_amount > 0,
    }
}

/// 计算月度累计数据时去掉 领取补助 类型，不然会有重复计费
fn monthly_summary(records: &[TransactionRecord]) -> (i64, usize) {
    records
        .iter()
        .filter(|item| {
            item.transaction_type != TransactionType::MoneyTransfer
        })
        .fold((0, 0), |(total, count), item| {
            (total + item.transaction_amount, count + 1)
        })
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

    records.retain(|item| {
        matches_history_type(
            item.transaction_type,
            item.transaction_amount,
            history_type,
        )
    });
    records.sort_by_key(|item| Reverse(item.pay_time));

    let (total_cents, tran_count) = monthly_summary(&records);
    let res_items = records
        .into_iter()
        .map(CardHistoryItem::from)
        .collect::<Vec<_>>();
    let res = CardHistory {
        TranCount: tran_count as f64,
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

    fn transaction_record(
        transaction_type: TransactionType,
        transaction_amount: i64,
    ) -> TransactionRecord {
        TransactionRecord {
            transaction_name: String::new(),
            merchant_name: String::new(),
            merchant_account: 0,
            pay_time: NaiveDate::from_ymd_opt(2026, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            transaction_amount,
            card_balance: 0,
            transaction_type,
        }
    }

    #[test]
    fn test_matches_history_type() {
        for transaction_type in [
            TransactionType::Consumption,
            TransactionType::Recharge,
            TransactionType::Benefit,
            TransactionType::MoneyTransfer,
            TransactionType::QRCodeConsumption,
            TransactionType::Withholding,
        ] {
            assert!(matches_history_type(
                transaction_type,
                -1,
                CardHistoryType::Consumption
            ));
            assert!(matches_history_type(
                transaction_type,
                1,
                CardHistoryType::Recharge
            ));
            assert!(!matches_history_type(
                transaction_type,
                1,
                CardHistoryType::Consumption
            ));
            assert!(!matches_history_type(
                transaction_type,
                -1,
                CardHistoryType::Recharge
            ));
        }

        for transaction_type in [
            TransactionType::EAccountOpening,
            TransactionType::HolderAccountOpening,
        ] {
            assert!(!matches_history_type(
                transaction_type,
                -1,
                CardHistoryType::Consumption
            ));
            assert!(!matches_history_type(
                transaction_type,
                1,
                CardHistoryType::Recharge
            ));
        }

        assert!(!matches_history_type(
            TransactionType::Consumption,
            0,
            CardHistoryType::Consumption
        ));
        assert!(!matches_history_type(
            TransactionType::Recharge,
            0,
            CardHistoryType::Recharge
        ));
    }

    #[test]
    fn test_monthly_summary_excludes_money_transfer() {
        let records = [
            transaction_record(TransactionType::Consumption, -100),
            transaction_record(TransactionType::MoneyTransfer, -4),
            transaction_record(TransactionType::Withholding, -4),
            transaction_record(TransactionType::MoneyTransfer, 1000),
            transaction_record(TransactionType::Recharge, 200),
        ];

        assert_eq!(monthly_summary(&records), (96, 3));
    }

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
