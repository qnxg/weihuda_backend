use crate::{login::get_ticket_url, utils::client};
use anyhow::{Result, anyhow};
use log::debug;
use reqwest::header::HeaderMap;
use serde_json::Value;

const CA_URL: &str = "http://cas.hnu.edu.cn/cas/login?service=https://ca.hnu.edu.cn/student/";

/// 可信电子凭证登录
pub async fn ca_headers(
    stu_id: &str,
) -> Result<HeaderMap, crate::Error> {
    let ticket_url = get_ticket_url(stu_id, CA_URL, None).await?;
    debug!("{stu_id} 尝试通过 {} 访问可信电子凭证", ticket_url);
    client.get(&ticket_url).send().await?.error_for_status()?;
    let ticket =
        ticket_url.split("ticket=").collect::<Vec<&str>>()[1];
    let res: Value =
        client.get(format!("https://ca.hnu.edu.cn/student/cas/client/validateLogin?ticket={ticket}%23%2F&service=https:%2F%2Fca.hnu.edu.cn%2Fstudent%2F"))
        .send().await?
        .error_for_status()?
        .json().await?;

    if res["message"] != "登录成功" {
        return Err(anyhow!("登录失败").into());
    }
    let token = res["result"]["token"].as_str().unwrap();
    let cookie = format!("X-Access-Token={token}");
    let mut headers = HeaderMap::new();
    headers.insert("X-Access-Token", token.parse()?);
    headers.insert("Cookie", cookie.parse()?);
    Ok(headers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TEST_STU_ID;

    #[tokio::test]
    async fn test_ca() {
        let ca = ca_headers(&TEST_STU_ID).await;
        println!("{:#?}", ca.unwrap());
    }
}
