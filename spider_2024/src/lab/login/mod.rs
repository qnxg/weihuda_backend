mod raw;

use crate::{
    lab::login::raw::raw_login_data,
    utils::{
        cache::{CACHE, CacheEnum::*},
        db::get_lab_password,
    },
};
use anyhow::{Result, anyhow};
use reqwest::header::{COOKIE, HeaderMap};

/// 检查密码的结果
#[derive(Debug)]
pub enum CheckPasswordResult {
    /// 密码正确
    ///
    /// 包含登录后获取的 cookie
    Success(String),
    /// 密码错误
    PasswordError,
    /// 其他错误
    ///
    /// 包含错误信息，如果为 None 则表示未知错误信息
    OtherError(Option<String>),
}

pub async fn check_password(
    stu_id: &str,
    password: &str,
) -> Result<CheckPasswordResult, crate::Error> {
    let (raw_data, cookies) =
        raw_login_data(stu_id, password).await?;
    let res = match raw_data.get("RTNCode").and_then(|v| v.as_i64()) {
        None => {
            return Err(
                anyhow!("解析响应数据失败: {:?}", raw_data).into()
            );
        }
        Some(1) => CheckPasswordResult::Success(cookies),
        Some(-1) => CheckPasswordResult::PasswordError,
        _ => CheckPasswordResult::OtherError(
            raw_data
                .get("Data")
                .and_then(|v| v.as_str().map(String::from)),
        ),
    };
    Ok(res)
}

pub async fn lab_headers(
    stu_id: &str,
) -> Result<HeaderMap, crate::Error> {
    let cookies = CACHE
        .try_get_with((LabCookie, stu_id.into()), async {
            let Some(password) = get_lab_password(stu_id).await?
            else {
                return Err(crate::Error::PasswordError);
            };
            let cookies =
                match check_password(stu_id, &password).await? {
                    CheckPasswordResult::Success(cookies) => {
                        if cookies.is_empty() {
                            return Err(anyhow!(
                                "登录实验平台失败：接收到空的 cookie"
                            )
                            .into());
                        }
                        cookies
                    }
                    CheckPasswordResult::PasswordError => {
                        return Err(crate::Error::PasswordError);
                    }
                    CheckPasswordResult::OtherError(msg) => {
                        return Err(anyhow!(
                            "登录实验平台失败：{}",
                            msg.unwrap_or("未知错误".to_string())
                        )
                        .into());
                    }
                };
            Ok(cookies)
        })
        .await?;
    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, cookies.parse()?);
    Ok(headers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TEST_STU_ID;

    #[tokio::test]
    async fn test_check_password() {
        let res =
            check_password(&TEST_STU_ID, "123456").await.unwrap();
        println!("{:#?}", res);
    }

    #[tokio::test]
    async fn test_lab_headers() {
        let lab = lab_headers(&TEST_STU_ID).await;
        println!("{:#?}", lab.unwrap());
    }
}
