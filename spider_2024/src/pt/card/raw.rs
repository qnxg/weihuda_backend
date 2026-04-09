use crate::{pt::login::pt_headers, utils::client};
use anyhow::anyhow;
use serde::Deserialize;
use serde_json::Value;

const CARD_INFO_URL: &str =
    "https://pt.hnu.edu.cn/api/hndxYkt/getCardUserInfo/info";
const CSRF_TOKEN_URL: &str =
    "https://pt.hnu.edu.cn/api/security/token";
const CARD_HISTORY_URL: &str =
    "https://pt.hnu.edu.cn/api/hndxYkt/getAccHisConsubDzzfLog/detail";

#[derive(Deserialize, Debug)]
pub struct CardInfo {
    pub account: u32,
    pub balance: String,
}

/// 由代码推断的校园卡交易历史接口返回值
#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct CardHistory {
    /// 总额
    // TODO 浮点数？
    pub amt: f64,
    /// 交易数量
    // TODO 浮点数？
    pub count: f64,
    /// 交易项列表
    pub webTrjnDTO: Option<Vec<CardHistoryItem>>,
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct CardHistoryItem {
    pub fTranAmt: String,
    pub jndatetime: String,
    pub effectdate: String,
    pub jourName: String,
    pub usedcardnum: u32,
    pub nowAmt: String,
    pub sysname1: Option<String>,
    pub tranname: String,
}

pub async fn raw_card_info_data(
    stu_id: &str,
) -> Result<CardInfo, crate::Error> {
    let pt_headers = pt_headers(stu_id, None).await?;
    let raw_res = client
        .get(CARD_INFO_URL)
        .headers(pt_headers)
        .send()
        .await?
        .error_for_status()?
        .json::<Value>()
        .await?;
    let res: CardInfo = raw_res
        .get("data")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .ok_or(anyhow!("解析校园卡信息失败: {:?}", raw_res))?;
    Ok(res)
}

pub async fn raw_card_history_data(
    stu_id: &str,
    year: u16,
    month: u8,
    trancode: &str,
) -> Result<CardHistory, crate::Error> {
    let pt_headers = pt_headers(stu_id, None).await?;
    // 字符串格式化默认是左对齐，这里要手动改成右对齐，并且两位宽左侧补0
    let begin_date = format!("{}-{:0>2}-01", year, month);
    // 这里没有必要精确查询日历好像是？直接取31号
    let end_date = format!("{}-{:0>2}-31", year, month);
    let token: Value = client
        .get(CSRF_TOKEN_URL)
        .headers(pt_headers.clone())
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let token = token
        .get("data")
        .and_then(|v| v.as_str())
        .ok_or(anyhow!("获取token失败"))?;
    let form_data = [
        ("beginDate", begin_date.as_str()),
        ("endDate", end_date.as_str()),
        ("pageSize", "100000"),
        ("trancode", trancode),
    ];

    let raw_res = client
        .post(CARD_HISTORY_URL)
        .headers(pt_headers)
        .header("X-XSRF-TOKEN", token)
        .form(&form_data)
        // .timeout(Duration::from_secs(5)) // 这个请求消费数据比较多的时候比较慢，单独设置5s超时
        .send()
        .await?
        .error_for_status()?
        .json::<Value>()
        .await?;
    let res: CardHistory = raw_res
        .get("data")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .ok_or(anyhow!("解析校园卡交易历史失败: {:?}", raw_res))?;
    Ok(res)
}
