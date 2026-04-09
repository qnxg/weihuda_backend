use crate::{
    login::get_sticket,
    utils::{
        cache::{CACHE, CacheEnum::*},
        client,
        request::cookie_parser,
    },
};
use anyhow::{Result, anyhow};
use reqwest::header::{COOKIE, HeaderMap, SET_COOKIE};

const NETFLOW_URL: &str = "http://cas.hnu.edu.cn/application/sso.zf?login=B5712DC2FA281C96E053026B3E0A80A6";

/// 获取校园网流量
pub async fn netflow_headers(
    stu_id: &str,
) -> Result<HeaderMap, crate::Error> {
    let cookies = CACHE
        .try_get_with((NetflowCookie, stu_id.into()), async {
            let (s_ticket, cookies) =
                get_sticket(stu_id, NETFLOW_URL).await?;
            // 发送请求
            let res = client
                .get("http://ll.hnu.edu.cn/login/validate")
                .header(COOKIE, &cookies)
                .form(&[
                    ("s_ticket", s_ticket.as_str()),
                    ("login_id", stu_id),
                    ("password", ""),
                    ("null", ""),
                ])
                .send()
                .await?
                .error_for_status()?;
            // 获取cookies
            let cookies =
                cookie_parser(res.headers().get_all(SET_COOKIE));
            // 保留Token，有三个.ASPXAUTH，
            // 只要最后面的一个（这里就不写死是cookies[0]和cookies[3]了）
            if cookies.is_empty() {
                return Err(anyhow!("校园网流量登录失败").into());
            }
            let res = format!(
                "{}; {}",
                cookies.first().unwrap(),
                cookies.last().unwrap()
            );
            Ok(res)
        })
        .await?;
    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, cookies.parse()?);
    Ok(headers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TEST_STU_ID;

    #[tokio::test]
    async fn test_netflow() {
        let netflow_headers =
            netflow_headers(&TEST_STU_ID).await.unwrap();
        println!("{:#?}", netflow_headers);
    }
}
