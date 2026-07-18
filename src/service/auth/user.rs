use crate::utils;
use crate::{error::AppResult, infra};

pub use infra::mysql::user::clear_openid;
pub use infra::mysql::user::get_by_openid as check_by_openid;
pub use infra::mysql::user::get_by_stu_id as check_by_stu_id;
pub use infra::wechat::get_openid;

/// password 提供明文即可，该函数会自动加密
/// TODO QQ 的绑定
pub async fn bind(
    stu_id: &str,
    openid: &str,
    password: &str,
) -> AppResult<()> {
    infra::mysql::user::add_user(
        stu_id,
        &utils::crypto::encrypt(password),
        Some(openid),
        None,
    )
    .await?;
    Ok(())
}
