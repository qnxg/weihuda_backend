use anyhow::anyhow;
use serde::Deserialize;
use serde_json::Value;

use crate::{pt::login::pt_headers, utils::client};

const UNREAD_EMAIL_URL: &str =
    "https://pt.hnu.edu.cn/api/v1/email/unRead/count";

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct UnreadEmail {
    pub unReadCount: Option<u32>,
}

pub async fn raw_unread_email_data(
    stu_id: &str,
) -> Result<UnreadEmail, crate::Error> {
    let pt_headers = pt_headers(stu_id, None).await?;
    let raw_res = client
        .get(UNREAD_EMAIL_URL)
        .headers(pt_headers)
        .send()
        .await?
        .error_for_status()?
        .json::<Value>()
        .await?;
    let res: UnreadEmail = raw_res
        .get("data")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .ok_or(anyhow!("解析未读邮件数失败: {:?}", raw_res))?;
    Ok(res)
}
