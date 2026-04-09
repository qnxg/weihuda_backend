use crate::{
    login::get_ticket_url,
    utils::{
        cache::{CACHE, CacheEnum::*},
        client,
        request::cookie_parser,
    },
};
use anyhow::{Result, anyhow};
use log::debug;
use reqwest::{
    StatusCode,
    header::{COOKIE, HeaderMap, LOCATION, SET_COOKIE},
};

const HDJW_FROM_CAS_URL: &str = "http://cas.hnu.edu.cn/cas/login?service=http://hdjw.hnu.edu.cn/gld/sso.jsp";
const HDJW_ENTER_URL: &str = "http://hdjw.hnu.edu.cn/gld/sso.jsp";

/// 教务系统登录
pub async fn hdjw_headers(
    stu_id: &str,
) -> Result<HeaderMap, crate::Error> {
    let cookies = CACHE
        .try_get_with((Hdjw, stu_id.into()), async {
            // 需要先请求 hdjw 的登录页面，获取到相关的 cookie
            let cookies = cookie_parser(
                client
                    .get(HDJW_ENTER_URL)
                    .send()
                    .await?
                    .error_for_status()?
                    .headers()
                    .get_all(SET_COOKIE),
            )
            .join("; ");
            let ticket_url =
                get_ticket_url(stu_id, HDJW_FROM_CAS_URL, None)
                    .await?;
            debug!("{stu_id} 尝试通过 {} 访问教务系统", ticket_url);
            // 这里需要带着之前拿到的 cookies 去访问 ticket_url，不然会返回 500 internal server
            // error
            client
                .get(ticket_url)
                .header(COOKIE, &cookies)
                .send()
                .await?
                .error_for_status()?;
            // 上面的请求会重定向到 HDJW_ENTER_URL，我们再访问一下。
            let res = client
                .get(HDJW_ENTER_URL)
                .header(COOKIE, &cookies)
                .send()
                .await?
                .error_for_status()?;
            // 随后又会被重定向到一个新的链接，再请求一下就会得到 hdjw 鉴权的 cookie
            if res.status() != StatusCode::FOUND {
                return Err(anyhow!(
                    "获取教务系统失败，HTTP代码 {} {}",
                    res.status(),
                    res.text().await.unwrap_or_default()
                )
                .into());
            }
            let target_url = res
                .headers()
                .get(LOCATION)
                .ok_or(anyhow!("获取重定向链接失败"))?
                .to_str()?;
            let new_cookies = cookie_parser(
                client
                    .get(target_url)
                    .header(COOKIE, &cookies)
                    .send()
                    .await?
                    .error_for_status()?
                    .headers()
                    .get_all(SET_COOKIE),
            )
            .join("; ");
            // 保险起见，将两次 cookie 合并一下
            Ok(format!("{}; {}", cookies, new_cookies))
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
    async fn test_hdjw() {
        let headers = hdjw_headers(&TEST_STU_ID).await.unwrap();
        println!("{:#?}", headers);
    }
}
