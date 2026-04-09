use crate::utils::{
    self, captcha::CaptchaType, client, request::cookie_parser,
};
use anyhow::anyhow;
use log::debug;
use reqwest::header::SET_COOKIE;
use serde_json::Value;

const LOGIN_URL: &str =
    "http://10.62.106.112/BaseInfo/Login/ValidateLogin";
const CAPTCHA_URL: &str =
    "http://10.62.106.112/Ashx/CheckCode.ashx?t=0.29911677684547566";

/// 获取原始登录数据，自动处理了验证码
///
/// # Parameters
///
/// - `stu_id`: 学号
/// - `password`: 大物实验平台密码
///
/// # Returns
///
/// 返回一个二元组，第一个元素响应数据，第二个元素是获取的所有 cookie
pub async fn raw_login_data(
    stu_id: &str,
    password: &str,
) -> Result<(Value, String), crate::Error> {
    let password = utils::crypto::lab_encrypt(password);
    let mut tried = 0;
    let mut checkcode = String::new();
    let mut all_cookies = String::new();
    while tried < 5 {
        let res = client
            .post(LOGIN_URL)
            .form(&[
                ("uname", stu_id),
                ("pwd", &password),
                ("checkcode", &checkcode),
            ])
            .header("Cookie", &all_cookies)
            .send()
            .await?
            .error_for_status()?;
        let cookies =
            cookie_parser(res.headers().get_all(SET_COOKIE));
        if !cookies.is_empty() {
            all_cookies
                .push_str(&format!("; {}", cookies.join("; ")));
        }
        let data: Value = res.json().await?;
        if let Some(code) = data["RTNCode"].as_i64() {
            if code == -2 {
                // 需要验证码
                let res = client
                    .get(CAPTCHA_URL)
                    .header("Cookie", &all_cookies)
                    .send()
                    .await?
                    .error_for_status()?;
                let img_bytes = res.bytes().await?;
                checkcode = utils::captcha::captcha_solve(
                    &img_bytes,
                    CaptchaType::Default,
                )
                .await?;
                tried += 1;
            } else {
                debug!("经过 {} 次尝试后成功登录实验平台", tried + 1);
                return Ok((data, all_cookies));
            }
        } else {
            return Err(anyhow!("意料之外的响应: {}", data))?;
        }
    }
    // 尝试多次后仍然无法通过，返回错误
    Err(anyhow!("解析验证码失败").into())
}
