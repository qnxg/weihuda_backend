use serde::Deserialize;

use spider_2024::dtos::pt::CasPasswordStatus;

use crate::result::AppResult;

#[derive(Deserialize, Debug)]
pub struct VerifyResult {
    pub code: u32,
    #[expect(unused)]
    pub status: String,
    pub message: String,
}
pub async fn verify_password(
    stu_id: &str,
    password: &str,
) -> AppResult<VerifyResult> {
    let password_check_status =
        spider_2024::pt::check_password_handler(stu_id, password)
            .await?;

    match password_check_status {
        CasPasswordStatus::Success => Ok(VerifyResult {
            code: 0,
            status: "success".to_string(),
            message: "密码正确".to_string(),
        }),
        CasPasswordStatus::Fail => Ok(VerifyResult {
            code: 1,
            status: "error".to_string(),
            message: "密码错误".to_string(),
        }),
        CasPasswordStatus::ShouldChange => Ok(VerifyResult {
            code: 1,
            status: "error".to_string(),
            message: "请前往个人门户修改密码后重试".to_string(),
        }),
        CasPasswordStatus::Lock => Ok(VerifyResult {
            code: 1,
            status: "error".to_string(),
            message: "账号被锁定，请10分钟之后再试".to_string(),
        }),
    }
}
