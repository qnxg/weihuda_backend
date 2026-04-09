use reqwest::{
    StatusCode,
    header::{COOKIE, HeaderMap, SET_COOKIE},
};

use crate::{
    login::get_ticket_url,
    utils::{
        cache::{CACHE, CacheEnum::XGXTCookie},
        client,
        request::cookie_parser,
    },
};
use anyhow::anyhow;
use log::debug;

const XGXT_URL: &str = "http://cas.hnu.edu.cn/cas/login?service=http://xgxt.hnu.edu.cn/zftal-xgxt-web/teacher/xtgl/index/check.zf";

pub async fn xgxt_headers(
    stu_id: &str,
) -> Result<HeaderMap, crate::Error> {
    let cookies = CACHE
        .try_get_with((XGXTCookie, stu_id.into()), async {
            let ticket_url =
                get_ticket_url(stu_id, XGXT_URL, None).await?;
            debug!("{stu_id} 尝试通过 {} 访问学工系统", ticket_url);
            // cas 下发的 ticket_url 是 http 的，但是学工系统要用 https
            let res = client
                .get(ticket_url.replace("http://", "https://"))
                .send()
                .await?;
            if res.status() != StatusCode::FOUND {
                return Err(anyhow!(
                    "获取学工系统失败，HTTP代码 {}",
                    res.status()
                )
                .into());
            }
            let cookies: String =
                cookie_parser(res.headers().get_all(SET_COOKIE))
                    .join("; ");
            if cookies.is_empty() {
                return Err(anyhow!(
                    "获取学工系统失败，接收到空的 cookie"
                )
                .into());
            }
            Ok(cookies)
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
    async fn test_xgxt() {
        let xgxt = xgxt_headers(&TEST_STU_ID).await;
        println!("{:#?}", xgxt.unwrap());
    }
}
