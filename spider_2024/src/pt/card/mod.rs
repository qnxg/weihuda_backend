use crate::pt::card::raw::{
    raw_card_history_data, raw_card_info_data,
};
use anyhow::anyhow;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

mod raw;

/// 校园卡信息
#[derive(Serialize, Deserialize, Debug)]
pub struct CardInfo {
    /// 校园卡账号
    pub id: u32,
    /// 校园卡余额
    // TODO 解析成整数
    pub balance: f64,
}

/// 校园卡消费历史类型
#[derive(Serialize, Deserialize, Debug)]
pub enum CardHistoryType {
    /// 充值
    Recharge,
    /// 消费
    Consumption,
}

/// 校园卡消费历史详情
#[derive(Serialize, Deserialize, Debug)]
pub struct CardHistory {
    /// 总交易金额
    ///
    /// 如果是充值金额则是正数，如果是消费金额则是负数
    pub total: f64,
    /// 交易数量
    pub count: u32,
    /// 交易项列表
    pub items: Vec<CardHistoryItem>,
}

/// 校园卡消费历史的交易项
#[derive(Serialize, Deserialize, Debug)]
pub struct CardHistoryItem {
    /// 交易时间
    pub date_time: NaiveDateTime,
    /// 记账时间
    pub journal_time: NaiveDateTime,
    /// 交易状态，比如 `正常`
    pub status: String,
    /// 交易 id
    pub id: u32,
    /// 交易后余额
    pub now_balance: f64,
    /// 交易金额
    ///
    /// 如果是充值金额则是正数，如果是消费金额则是负数
    pub amount: f64,
    /// 交易地点
    pub location: Option<String>,
    /// 交易名称
    pub name: String,
}

pub async fn get_card_info(
    stu_id: &str,
) -> Result<CardInfo, crate::Error> {
    let res = raw_card_info_data(stu_id).await?;
    Ok(CardInfo {
        id: res.account,
        balance: res.balance.parse::<f64>().map_err(|e| {
            anyhow!(
                "解析校园卡余额失败 err = {}, data = {}",
                e,
                res.balance
            )
        })? / 100.0,
    })
}

/// 获取校园卡消费历史
///
/// # Parameters
///
/// - `stu_id`: 学号
/// - `year`: 年份
/// - `month`: 月份
/// - `history_type`: 查询充值记录还是消费记录
///
/// # Returns
///
/// 校园卡消费历史信息
pub async fn get_card_history(
    stu_id: &str,
    year: u16,
    month: u8,
    history_type: CardHistoryType,
) -> Result<CardHistory, crate::Error> {
    let trancode = match history_type {
        CardHistoryType::Consumption => "15",
        CardHistoryType::Recharge => "16",
    };
    let raw_data =
        raw_card_history_data(stu_id, year, month, trancode).await?;
    let raw_items = raw_data.webTrjnDTO.unwrap_or_default();
    let mut items = Vec::with_capacity(raw_items.len());
    for item in raw_items {
        let (
            Ok(date_time),
            Ok(journal_time),
            Ok(now_balance),
            Ok(amount),
        ) = (
            NaiveDateTime::parse_from_str(
                &item.effectdate,
                "%Y/%m/%d %H:%M:%S",
            ),
            NaiveDateTime::parse_from_str(
                &item.jndatetime,
                "%Y/%m/%d %H:%M:%S",
            ),
            item.nowAmt.parse::<f64>(),
            item.fTranAmt.parse::<f64>(),
        )
        else {
            return Err(anyhow!(
                "解析交易项目失败: data = {:?}",
                item
            )
            .into());
        };
        items.push(CardHistoryItem {
            date_time,
            journal_time,
            status: item.jourName,
            id: item.usedcardnum,
            now_balance,
            amount,
            location: item.sysname1.map(|s| s.trim().to_string()),
            name: item.tranname,
        });
    }
    let res = CardHistory {
        total: raw_data.amt / 100.0,
        count: raw_data.count as u32,
        items,
    };
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TEST_STU_ID;

    #[tokio::test]
    async fn test_get_card_info() {
        let res = get_card_info(&TEST_STU_ID).await.unwrap();
        println!("{:#?}", res);
    }

    #[tokio::test]
    async fn test_get_card_history() {
        let year = 2026;
        let month = 3;
        let res = get_card_history(
            &TEST_STU_ID,
            year,
            month,
            CardHistoryType::Consumption,
        )
        .await
        .unwrap();
        println!("{:#?}", res);
    }
}
