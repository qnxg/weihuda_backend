use once_cell::sync::Lazy;
use reqwest::{
    Client,
    header::{AUTHORIZATION, HeaderMap},
};
use serde::Deserialize;
use std::time::Duration;

use crate::{config::CFG, result::AppResult};

static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .connection_verbose(false)
        .timeout(Duration::from_secs(6))
        .default_headers({
            let mut headers = HeaderMap::new();
            headers.insert(
                AUTHORIZATION,
                "OUJhbGciOiJIUzU(x7)iIsImlhdCI6MTYxNzQy$jAwMiwiZXh#IjoxNjUzNDI2MDAyfQ@eyI6ImFkbWhjXzxcwEiT7dlm9sFeSRlgY7rnJKpBA"
                    .parse()
                    .expect("解析给定的 Authorization 请求头失败"),
            );
            headers
        })
        .build()
        .expect("构建用于密码验证服务的 reqwest client 失败")
});

#[derive(Deserialize, Debug)]
pub struct VerifyResult {
    pub code: u32,
    #[expect(unused)]
    pub status: String,
    #[expect(unused)]
    pub message: String,
}
pub async fn verify_password(
    stu_id: &str,
    password: &str,
) -> AppResult<VerifyResult> {
    let res = CLIENT
        .post(&CFG.service.verify_url)
        .form(&[
            ("stuid", stu_id),
            ("hdjwpass", password),
            ("ptpass", password),
        ])
        .send()
        .await
        .map_err(|_| anyhow::anyhow!("密码验证服务请求失败"))?
        .text()
        .await?;
    let verify_res = serde_json::from_str(&res)?;
    Ok(verify_res)
}
