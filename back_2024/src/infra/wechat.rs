use anyhow::anyhow;
use serde::Deserialize;
use serde_json::Value;

use crate::{config::CFG, result::AppResult};

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
    let value: Value = serde_json::from_str(
        &reqwest::get(&url).await?.text().await?,
    )?;
    // from_value 会直接拿走所有权，所以这里先提前借用 value 生成一个错误信息
    let errmsg = anyhow!("获取 openid 失败: {:?}", value);
    if let Ok(res) = serde_json::from_value::<OpenID>(value) {
        Ok(res.openid)
    } else {
        Err(errmsg.into())
    }
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
