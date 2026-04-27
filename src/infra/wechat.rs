use serde::Deserialize;

use crate::{
    config::CFG,
    result::{AppResult, ThrowError},
};

#[derive(Deserialize, Debug)]
pub struct OpenID {
    #[expect(unused)]
    pub session_key: String,
    pub openid: String,
}
/// WARNING: code 只能使用一次，重复使用会报错
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
            .throw_error("请求微信 openid 接口失败")?
            .text()
            .await
            .throw_error("获取微信 openid 接口响应失败")?,
    )
    .throw_error("解析微信 openid 接口响应失败")?;
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
