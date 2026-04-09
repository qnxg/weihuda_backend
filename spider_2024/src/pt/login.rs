use crate::{
    login::get_ticket_url,
    utils::{
        cache::{CACHE, CacheEnum::*, invalidate_stuid_cache},
        client,
        request::cookie_parser,
    },
};
use anyhow::{Result, anyhow};
use log::debug;
use reqwest::{
    StatusCode,
    header::{COOKIE, HeaderMap, SET_COOKIE},
};

// WARN 注意这个url后面必须带`/`，不然无法正常跳转
const PT_URL: &str =
    "http://cas.hnu.edu.cn/cas/login?service=https://pt.hnu.edu.cn/";

/// 个人门户密码验证结果
#[derive(Debug)]
pub enum CheckPasswordResult {
    /// 密码正确
    Success,
    /// 密码错误
    Fail,
    /// 需要更换密码
    ShouldChange,
    /// 账号被锁定
    Lock,
}

pub async fn check_password(
    stu_id: &str,
    password: &str,
) -> Result<CheckPasswordResult, crate::Error> {
    let res = pt_headers(stu_id, Some(password)).await;
    match res {
        Ok(_) => {
            // 把缓存全部重置
            invalidate_stuid_cache(stu_id).await;
            Ok(CheckPasswordResult::Success)
        }
        Err(crate::Error::PasswordError) => {
            Ok(CheckPasswordResult::Fail)
        }
        Err(crate::Error::PasswordShouldChange) => {
            Ok(CheckPasswordResult::ShouldChange)
        }
        Err(crate::Error::PasswordLocked) => {
            Ok(CheckPasswordResult::Lock)
        }
        Err(e) => Err(e),
    }
}

/// 个人门户登录
///
/// 这个函数还有个作用是可以用来进行密码检查
///
/// # Arguments
///
/// - `stu_id`: 学号
/// - `password`: 密码，可选，若不提供，则自动从数据库拉取密码。
///
/// # Returns
///
/// 后续请求个人门户所需的 HeaderMap
///
/// # Side Effects
///
/// 函数执行成功，则会把函数内部获取到的 PtCookie 进行缓存
///
/// 如果 `password` 不为 None，则该函数执行的整个过程都不会使用或者是设置 PtCookie 缓存
pub async fn pt_headers(
    stu_id: &str,
    password: Option<&str>,
) -> Result<HeaderMap, crate::Error> {
    let cached_cookies = if password.is_none() {
        CACHE.get(&(PtCookie, stu_id.into())).await
    } else {
        None
    };
    let cookies = if let Some(v) = cached_cookies {
        v
    } else {
        let ticket_url =
            get_ticket_url(stu_id, PT_URL, password).await?;
        debug!("{stu_id} 尝试通过 {} 访问个人门户", ticket_url);
        let res = client
            .get(ticket_url)
            .send()
            .await?
            .error_for_status()?;
        if res.status() != StatusCode::FOUND {
            return Err(anyhow!("获取个人门户失败").into());
        }
        let res = cookie_parser(res.headers().get_all(SET_COOKIE))
            .join("; ");
        if password.is_none() {
            CACHE
                .insert((PtCookie, stu_id.into()), res.clone())
                .await;
        }
        res
    };
    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, cookies.parse()?);
    Ok(headers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TEST_STU_ID;

    #[tokio::test]
    async fn test_pt() {
        let res = pt_headers(&TEST_STU_ID, None).await.unwrap();
        println!("{:#?}", res);
    }
}
