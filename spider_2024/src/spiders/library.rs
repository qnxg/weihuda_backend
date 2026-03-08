use crate::{
    app_result::AppResult,
    dtos::library::{Borrowed, Borrowing, Finance},
    spiders::login::library_headers,
    utils::client,
};
use anyhow::anyhow;
use serde_json::{Value, json};
use std::collections::HashMap;

const CURRENT_LOAN_LIST_URL: &str =
    "http://opac.hnu.edu.cn/opac/loan/currentLoanList";
const HISTORY_LOAN_LIST_URL: &str =
    "http://opac.hnu.edu.cn/opac/loan/historyLoanList";
const FINANCE_LIST_URL: &str =
    "http://opac.hnu.edu.cn/opac/finance/financeList";

pub async fn get_current_list(stu_id: &str) -> AppResult<Value> {
    let library_headers = library_headers(stu_id).await?;
    // 只请求一页，如果真有神人数据达到了2000条以上，那么2000条之后的也不爬了，
    // 反正小程序也显示不下（）
    let mut form_data = HashMap::new();
    form_data.insert("page", 1);
    form_data.insert("rows", 2000);
    let html = client
        .post(CURRENT_LOAN_LIST_URL)
        .form(&form_data)
        .headers(library_headers)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let document = scraper::Html::parse_document(&html);
    let selector =
        scraper::Selector::parse("#right_div #content #contentTable")
            .map_err(|_e| anyhow!("html解析失败"))?;
    let mut list: Vec<Borrowing> = Vec::new();
    if let Some(table) = document.select(&selector).next() {
        for tr in table
            .select(
                &scraper::Selector::parse("tr")
                    .map_err(|_e| anyhow!("html解析失败"))?,
            )
            .skip(1)
        {
            let tds = tr
                .select(
                    &scraper::Selector::parse("td")
                        .map_err(|_e| anyhow!("html解析失败"))?,
                )
                .collect::<Vec<_>>();
            let title = tds
                .get(1)
                .ok_or(anyhow!("获取书名错误"))?
                .child_elements()
                .next()
                .ok_or(anyhow!("获取书名错误"))?
                .inner_html()
                .trim()
                .to_string();
            let isbn = tds
                .get(2)
                .ok_or(anyhow!("获取isbn错误"))?
                .inner_html()
                .trim()
                .to_string();
            let author = tds
                .get(3)
                .ok_or(anyhow!("获取作者错误"))?
                .inner_html()
                .trim()
                .to_string();
            let library = tds
                .get(5)
                .ok_or(anyhow!("获取图书馆信息错误"))?
                .inner_html()
                .trim()
                .to_string();
            let borrow_date = tds
                .get(7)
                .ok_or(anyhow!("获取借出日期错误"))?
                .inner_html()
                .trim()
                .to_string();
            let return_date = tds
                .get(8)
                .ok_or(anyhow!("获取应还日期错误"))?
                .inner_html()
                .trim()
                .to_string();
            list.push(Borrowing {
                author,
                borrow_date,
                isbn,
                library,
                return_date,
                title,
            });
        }
    }
    Ok(json!(list))
}

pub async fn get_history_list(stu_id: &str) -> AppResult<Value> {
    let library_headers = library_headers(stu_id).await?;
    let mut form_data = HashMap::new();
    form_data.insert("page", 1);
    form_data.insert("rows", 2000);
    let html = client
        .post(HISTORY_LOAN_LIST_URL)
        .form(&form_data)
        .headers(library_headers)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let document = scraper::Html::parse_document(&html);
    let selector =
        scraper::Selector::parse("#right_div #content #contentTable")
            .map_err(|_e| anyhow!("html解析失败"))?;
    let mut list: Vec<Borrowed> = Vec::new();
    if let Some(table) = document.select(&selector).next() {
        for tr in table
            .select(
                &scraper::Selector::parse("tr")
                    .map_err(|_e| anyhow!("html解析失败"))?,
            )
            .skip(1)
        {
            let tds = tr
                .select(
                    &scraper::Selector::parse("td")
                        .map_err(|_e| anyhow!("html解析失败"))?,
                )
                .collect::<Vec<_>>();
            let _type = tds
                .first()
                .ok_or(anyhow!("获取操作类型错误"))?
                .child_elements()
                .nth(1)
                .ok_or(anyhow!("获取签名类型错误"))?
                .inner_html()
                .trim()
                .to_string();
            let title = tds
                .get(2)
                .ok_or(anyhow!("获取书名错误"))?
                .child_elements()
                .next()
                .ok_or(anyhow!("获取书名错误"))?
                .inner_html()
                .trim()
                .to_string();
            let isbn = tds
                .get(3)
                .ok_or(anyhow!("获取isbn错误"))?
                .inner_html()
                .trim()
                .to_string();
            let author = tds
                .get(4)
                .ok_or(anyhow!("获取作者错误"))?
                .inner_html()
                .trim()
                .to_string();
            let library = tds
                .get(6)
                .ok_or(anyhow!("获取图书馆信息错误"))?
                .inner_html()
                .trim()
                .to_string();
            let time = tds
                .get(8)
                .ok_or(anyhow!("获取时间"))?
                .inner_html()
                .trim()
                .to_string();
            list.push(Borrowed {
                author,
                isbn,
                library,
                time,
                title,
                borrowed_type: _type,
            });
        }
    }
    Ok(json!(list))
}

pub async fn get_finance_list(stu_id: &str) -> AppResult<Value> {
    let library_headers = library_headers(stu_id).await?;
    let mut form_data = HashMap::new();
    form_data.insert("page", 1);
    form_data.insert("rows", 2000);
    let html = client
        .post(FINANCE_LIST_URL)
        .form(&form_data)
        .headers(library_headers)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let document = scraper::Html::parse_document(&html);
    let selector =
        scraper::Selector::parse("#right_div #content #contentTable")
            .map_err(|_e| anyhow!("html解析失败"))?;
    let mut list: Vec<Finance> = Vec::new();
    if let Some(table) = document.select(&selector).next() {
        for tr in table
            .select(
                &scraper::Selector::parse("tr")
                    .map_err(|_e| anyhow!("html解析失败"))?,
            )
            .skip(1)
        {
            let tds = tr
                .select(
                    &scraper::Selector::parse("td")
                        .map_err(|_e| anyhow!("html解析失败"))?,
                )
                .collect::<Vec<_>>();
            let fee_type = tds
                .first()
                .ok_or(anyhow!("获取费用类型错误"))?
                .inner_html()
                .trim()
                .to_string();
            let cost = tds
                .get(1)
                .ok_or(anyhow!("获取费用错误"))?
                .inner_html()
                .trim()
                .to_string();
            let time = tds
                .get(2)
                .ok_or(anyhow!("获取时间错误"))?
                .inner_html()
                .trim()
                .to_string();
            // let library = tds
            //     .get(4)
            //     .ok_or(anyhow!("获取发生馆错误"))?
            //     .inner_html()
            //     .trim()
            //     .to_string();
            let place = tds
                .get(6)
                .ok_or(anyhow!("获取发生地错误"))?
                .inner_html()
                .trim()
                .to_string();
            let barcode = tds
                .get(7)
                .ok_or(anyhow!("获取条形码错误"))?
                .inner_html()
                .trim()
                .to_string();
            let pay_sign = tds
                .get(9)
                .ok_or(anyhow!("获取支付标识错误"))?
                .inner_html()
                .trim()
                .to_string();
            list.push(Finance {
                barcode,
                cost,
                fee_type,
                pay_sign,
                place,
                time,
            })
        }
    }
    Ok(json!(list))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::request::STU_ID;

    #[tokio::test]
    async fn test_get_current_list() {
        dbg!(get_current_list(&STU_ID).await.unwrap());
    }

    #[tokio::test]
    async fn test_get_history_list() {
        dbg!(get_history_list(&STU_ID).await.unwrap());
    }

    #[tokio::test]
    async fn test_get_finance_list() {
        dbg!(get_finance_list(&STU_ID).await.unwrap());
    }
}
