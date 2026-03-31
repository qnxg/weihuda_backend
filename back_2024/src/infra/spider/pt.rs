use spider_2024::dtos::pt::{
    CardHistoryReq, CardHistoryReturn, CardInfoRes, UnreadEmailRes,
};

use crate::result::AppResult;

pub async fn get_card_info(stu_id: &str) -> AppResult<CardInfoRes> {
    let spider_res =
        spider_2024::pt::get_card_info_handler(stu_id).await?;
    Ok(spider_res)
}

pub async fn get_card_history(
    stu_id: &str,
    year: &str,
    month: &str,
    typ: &str,
) -> AppResult<CardHistoryReturn> {
    let spider_res =
        spider_2024::pt::get_card_history_handler(CardHistoryReq {
            stu_id: stu_id.to_string(),
            year: year.to_string(),
            month: month.to_string(),
            typ: typ.to_string(),
        })
        .await?;
    Ok(spider_res)
}

pub async fn get_email(stu_id: &str) -> AppResult<UnreadEmailRes> {
    let spider_res =
        spider_2024::pt::get_unread_email_handler(stu_id).await?;
    Ok(spider_res)
}
