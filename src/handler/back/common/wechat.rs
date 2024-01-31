use crate::{config::CFG, schema::back::auth::OpenID};

pub async fn get_openid(code: &str) -> Result<String, anyhow::Error> {
    let url = format!(
        "https://api.weixin.qq.com/sns/jscode2session?appid={}&secret={}&js_code={}&grant_type=authorization_code",
        &CFG.wechat.appid, &CFG.wechat.secret, code,
    );
    let res = reqwest::get(&url).await?.text().await?;
    let openid_res: OpenID = serde_json::from_str(&res)?;

    Ok(openid_res.openid)
}

// 单元测试后续再写
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_openid() {
        let code = "";
        let _res = get_openid(code).await;
    }
}
