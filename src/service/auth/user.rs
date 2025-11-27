use crate::{infra, result::AppResult};

pub use infra::mysql::user::get_by_openid as check_by_openid;
pub use infra::mysql::user::get_by_stu_id as check_by_stu_id;
pub use infra::verify::verify_password;
pub use infra::wechat::get_openid;

/// 需要保证提供的 mini_bind 是正确的，该函数会直接 expect
pub async fn bind(
    openid: &str,
    stu_id: &str,
    stu_pass: &str,
    hdjw_pass: &str,
) -> AppResult<()> {
    infra::mysql::user::add_user(openid, stu_id, stu_pass, hdjw_pass)
        .await?;
    // 需要清一下缓存，因为爬虫那边会缓存用户密码到 redis 中
    infra::redis::clear_stuid_cache(stu_id).await
}

pub use infra::mysql::user::delete_user as unbind;
