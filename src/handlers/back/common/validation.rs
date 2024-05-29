use crate::{
    config::CFG,
    dtos::back::user::{CryptoResult, VerifyResult},
};

pub async fn verify_password(
    client: &reqwest::Client,
    stu_id: &str,
    hdjw_pass: &str,
    stu_pass: &str,
) -> Result<VerifyResult, anyhow::Error> {
    let res = client
        .post(&CFG.service.verify_url)
        .form(&[("stuid", stu_id), ("hdjwpass", hdjw_pass), ("ptpass", stu_pass)])
        .send()
        .await
        .map_err(|_| anyhow::anyhow!("密码验证服务请求失败"))?
        .text()
        .await?;
    let verify_res = serde_json::from_str(&res)?;
    Ok(verify_res)
}

pub async fn crypto_password(
    client: &reqwest::Client,
    hdjw_pass: &str,
    stu_pass: &str,
) -> Result<CryptoResult, anyhow::Error> {
    let res = client
        .post(&CFG.service.crypto_url)
        .form(&[("password", hdjw_pass), ("ptPassword", stu_pass)])
        .send()
        .await
        .map_err(|_| anyhow::anyhow!("密码加密服务请求失败"))?
        .text()
        .await?;
    let crypto_res = serde_json::from_str(&res)?;
    Ok(crypto_res)
}
