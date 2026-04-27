use crate::result::throw_error;
use crate::utils;
use crate::{infra, result::AppResult};

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

pub enum VerifyPasswordResult {
    Success,
    Fail,
    ShouldChange,
    Lock,
}

/// 检查密码是否正确
pub async fn verify_password(
    stu_id: &str,
    password: &str,
) -> AppResult<VerifyPasswordResult> {
    // 尝试登陆一下个人门户来检查代码
    let mut cas_token =
        spider_2024::cas::login::CasToken::new(stu_id, password);
    let res = match spider_2024::pt::login::PtToken::acquire_by_cas_login(
        &mut cas_token,
    )
    .await
    {
        Ok(_) => VerifyPasswordResult::Success,
        Err(spider_2024::Error::Other(issue)) => match issue {
            spider_2024::cas::login::AccountIssue::AccountLocked => {
                VerifyPasswordResult::Lock
            }
            spider_2024::cas::login::AccountIssue::PasswordError => {
                VerifyPasswordResult::Fail
            }
            spider_2024::cas::login::AccountIssue::PasswordShouldChange => {
                VerifyPasswordResult::ShouldChange
            }
        },
        Err(e) => return Err(throw_error(e, "验证密码失败")),
    };
    Ok(res)
}
