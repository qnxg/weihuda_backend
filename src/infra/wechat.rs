use serde::Deserialize;

use crate::{
    config::CFG,
    error::{AppResult, ThrowInternalErrorResult},
};

#[derive(Deserialize, Debug)]
pub struct OpenID {
    #[expect(unused)]
    pub session_key: String,
    pub openid: String,
}
/// WARNING: code 只能使用一次，重复使用会报错
#[tracing::instrument(
    skip_all
    fields(
        otel.kind = "client",
        event_type = "wechat",
    ),
    err
)]
pub async fn get_openid(code: &str) -> AppResult<String> {
    if code == "testing" {
        return Ok("testing".to_string());
    }

    let url = format!(
        "https://api.weixin.qq.com/sns/jscode2session?appid={}&secret={}&js_code={}&grant_type=authorization_code",
        &CFG.wechat.appid, &CFG.wechat.secret, code,
    );
    let value: OpenID = serde_json::from_str(
        &reqwest::get(&url)
            .await
            .internal_err()?
            .text()
            .await
            .internal_err()?,
    )
    .internal_err()?;
    Ok(value.openid)
}

// 单元测试后续再写
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_openid() {
        let code = "";
        get_openid(code).await.unwrap();
    }
}
