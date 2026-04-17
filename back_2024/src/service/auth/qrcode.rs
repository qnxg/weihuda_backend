use crate::utils::cache::CacheEnum::AuthQrCode;
use crate::{result::AppResult, utils::cache::CACHE};
use rand::{Rng, distributions::Alphanumeric};
use serde::{Deserialize, Serialize};

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
    CACHE.insert((AuthQrCode, code.clone()), "-1".into()).await;
    Ok(code)
}

/// 获取 auth_qrcode 状态
/// 如果不存在则返回 None
pub async fn get_auth_qrcode_status(
    code: &str,
) -> AppResult<Option<AuthQrCodeStatus>> {
    if let Some(stu_id) = CACHE.get(&(AuthQrCode, code.into())).await
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
    CACHE
        .insert((AuthQrCode, code.to_string()), stu_id.to_string())
        .await;
    Ok(())
}

/// 获取 auth_qrcode 的 stu_id
/// 如果不存在或者没绑定则返回 None
/// 成功获取后会删除该二维码
pub async fn consume_auth_qrcode(
    code: &str,
) -> AppResult<Option<String>> {
    if let Some(stu_id) = CACHE.get(&(AuthQrCode, code.into())).await
        && stu_id != "-1"
    {
        CACHE.invalidate(&(AuthQrCode, code.into())).await;
        Ok(Some(stu_id))
    } else {
        Ok(None)
    }
}
