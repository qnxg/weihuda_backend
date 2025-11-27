use crate::{infra, result::AppResult};
use rand::{Rng, distributions::Alphanumeric};
use serde::{Deserialize, Serialize};

const AUTH_QRCODE_KEY_PREFIX: &str = "auth_qrcode_";
const AUTH_QRCODE_EXPIRE_SECONDS: u64 = 600; // 10分钟

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AuthQrCodeStatus {
    // 刚创建，还没扫描
    #[serde(rename = "unused")]
    Unused,
    // 扫描了，但还没确认。目前这个状态没啥用
    #[serde(rename = "using")]
    Using,
    // 确认使用了
    #[serde(rename = "used")]
    Used,
}

/// 生成一个 auth_qrcode
pub async fn generate_auth_qrcode() -> AppResult<String> {
    let code: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();
    infra::redis::set_with_expire(
        &format!("{}{}", AUTH_QRCODE_KEY_PREFIX, code),
        "-1",
        AUTH_QRCODE_EXPIRE_SECONDS,
    )
    .await?;
    Ok(code)
}

/// 获取 auth_qrcode 状态
/// 如果不存在则返回 None
pub async fn get_auth_qrcode_status(
    code: &str,
) -> AppResult<Option<AuthQrCodeStatus>> {
    if let Some(stu_id) = infra::redis::get(&format!(
        "{}{}",
        AUTH_QRCODE_KEY_PREFIX, code
    ))
    .await?
    {
        if stu_id == "-1" {
            Ok(Some(AuthQrCodeStatus::Unused))
        } else {
            Ok(Some(AuthQrCodeStatus::Used))
        }
    } else {
        Ok(None)
    }
}

/// 确认 auth_qrcode 使用
/// 使用前需要确保 code 是存在的且没有已经被绑定
pub async fn confirm_auth_qrcode(
    code: &str,
    stu_id: &str,
) -> AppResult<()> {
    infra::redis::set_with_expire(
        &format!("{}{}", AUTH_QRCODE_KEY_PREFIX, code),
        stu_id,
        AUTH_QRCODE_EXPIRE_SECONDS,
    )
    .await
}

/// 获取 auth_qrcode 的 stu_id
/// 如果不存在或者没绑定则返回 None
/// 成功获取后会删除该二维码
pub async fn consume_auth_qrcode(
    code: &str,
) -> AppResult<Option<String>> {
    if let Some(stu_id) = infra::redis::get(&format!(
        "{}{}",
        AUTH_QRCODE_KEY_PREFIX, code
    ))
    .await?
        && stu_id != "-1"
    {
        infra::redis::del(&format!(
            "{}{}",
            AUTH_QRCODE_KEY_PREFIX, code
        ))
        .await?;
        Ok(Some(stu_id))
    } else {
        Ok(None)
    }
}
