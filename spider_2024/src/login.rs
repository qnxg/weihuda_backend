//! 目前这个mod里，基本所有CACHE的读写方式，
//! 都允许在缓存不命中或无效时并发重复尝试获取新的待缓存项。
//! 不严格限制读写是有意的，
//! 这是因为在压力测试中我发现锁会进一步降低爬虫成功处理请求的概率。
//! 若要严格序列化读写，可使用[`moka::future::Cache::try_get_with`]或
//! [`moka::future::OwnedKeyEntrySelector::and_try_compute_with`]

use std::sync::LazyLock;

use crate::utils::{
    cache::{CACHE, CacheEnum::*},
    client,
    crypto::rsa_encrypt,
    db::get_password,
    request::cookie_parser,
};
use anyhow::{Result, anyhow};
use log::debug;
use regex::Regex;
use reqwest::{
    StatusCode,
    header::{COOKIE, LOCATION, SET_COOKIE},
};
use serde_json::Value;

// 定义请求需要用到的一些地址常量
// 注意：这里的所有地址都必须是http的
// const LOGO_URL: &str = "http://cas.hnu.edu.cn/favicon.ico";
const _LOGIN_URL: &str = "http://cas.hnu.edu.cn/cas/login";
const PUBKEY_URL: &str = "http://cas.hnu.edu.cn/cas/v2/getPubKey";
const SERVICE_URL: &str = "http://cas.hnu.edu.cn/cas/login?service=http://cas.hnu.edu.cn/system/login/login.zf"; // 这个是sso.zf跳转用到的一个链接

pub struct LoginParams {
    modulus: String,
    exponent: String,
    execution: String,
    event_id: String,
    cookies: Vec<String>,
}

pub enum GetLoginParamsRes {
    Success(LoginParams), // 成功获取到登录参数
    Skip(String),         // 已经登录成立了自动跳转
}

/// 获取统一认证登录页提交密码所需的参数
#[inline]
pub async fn get_login_params(
    service_url: &str,
    cas_cookie: Option<&str>,
) -> Result<GetLoginParamsRes> {
    // 尝试登录对应系统
    let mut login_req = client.get(service_url);
    if let Some(v) = cas_cookie {
        login_req = login_req.header(COOKIE, v)
    };
    let login_res = match login_req.send().await?.error_for_status() {
        Ok(res) => res,
        // 这种情况可能是cookie失效
        Err(e) => {
            debug!("登录失败，尝试二次登录：{}", e);
            client
                .get(service_url)
                .send()
                .await?
                .error_for_status()?
        }
    };
    // 302就提前返回
    if login_res.status() == StatusCode::FOUND {
        debug!("已经登录成功，跳转到对应服务");
        let ticket_url = login_res
            .headers()
            .get("location")
            .ok_or(anyhow!("获取ticket失败"))?
            .to_str()?;
        return Ok(GetLoginParamsRes::Skip(ticket_url.to_string()));
    }
    if login_res.status() != StatusCode::OK {
        return Err(anyhow!("访问失败"));
    }
    // 获取到登录页的set-cookie
    let mut cookies =
        cookie_parser(login_res.headers().get_all(SET_COOKIE));
    // 拿到登录表单的execution和_eventId
    let login_text = login_res.text().await?;
    static EXECUTION_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"name="execution" value="(.*?)""#).unwrap()
    });
    static EVENT_ID_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"name="_eventId" value="(.*?)""#).unwrap()
    });
    let execution = EXECUTION_RE
        .captures(&login_text)
        .and_then(|cap| cap.get(1))
        .map_or("", |m| m.as_str())
        .to_string();
    let event_id = EVENT_ID_RE
        .captures(&login_text)
        .and_then(|cap| cap.get(1))
        .map_or("", |m| m.as_str())
        .to_string();
    // 通过pubkey接口获取modulus和exponent
    let pubkey = client
        .get(PUBKEY_URL)
        .header(COOKIE, &cookies.join("; "))
        .send()
        .await?
        .error_for_status()?;
    // 获取pubkey的cookies
    cookies
        .extend(cookie_parser(pubkey.headers().get_all(SET_COOKIE)));
    let pubkey: Value = pubkey.json().await?;
    let (modulus, exponent) = (
        pubkey["modulus"]
            .as_str()
            .ok_or(anyhow!("modulus not found"))?
            .to_string(),
        pubkey["exponent"]
            .as_str()
            .ok_or(anyhow!("exponent not found"))?
            .to_string(),
    );

    let login_params = LoginParams {
        modulus,
        exponent,
        execution,
        event_id,
        cookies,
    };
    Ok(GetLoginParamsRes::Success(login_params))
}

/// 获取带有ticket的跳转链接，打开即可登录对应平台
///
/// # Arguments
///
/// - `stu_id`: 学号
/// - `service_url`: cas 回调地址
/// - `password`: 密码，可选，若不提供，则自动从数据库拉取密码。
///
/// # Returns
///
/// 跳转链接
///
/// # Side Effects
///
/// 如果函数执行成功，则会把函数内部获取到的 CasCookie 进行缓存，后续调用会尝试使用 CasCookie 减少登录流程。
///
/// 如果 `password` 不为 None，则该函数执行的整个过程都不会使用或者是设置 CasCookie 缓存
pub async fn get_ticket_url(
    stu_id: &str,
    service_url: &str,
    password: Option<&str>,
) -> Result<String, crate::Error> {
    let mut cas_cache = CACHE.get(&(CasCookie, stu_id.into())).await;
    if password.is_some() {
        // 提供密码则不适用 CasCookie 缓存
        cas_cache = None;
    }
    let login_params =
        match get_login_params(service_url, cas_cache.as_deref())
            .await?
        {
            // 如果是跳过登录的情况，就提前返回ticket_url
            GetLoginParamsRes::Skip(ticket_url) => {
                return Ok(ticket_url);
            }
            GetLoginParamsRes::Success(v) => v,
        };

    let pending_password = match password {
        Some(v) => v.to_string(),
        None => get_password(stu_id).await?,
    };
    let rsa_password = rsa_encrypt(
        &pending_password,
        &login_params.exponent,
        &login_params.modulus,
    );

    // Post登录表单
    let login = client
        .post(service_url)
        // .header(CONTENT_TYPE, "application/x-www-form-urlencoded")   //
        // 这个header会自动加上，不用手动加
        .header(COOKIE, &login_params.cookies.join("; "))
        .form(&[
            ("username", stu_id),
            ("password", &rsa_password),
            ("authcode", ""),
            ("execution", &login_params.execution),
            ("_eventId", &login_params.event_id),
        ])
        .send()
        .await?;
    if login.status() == StatusCode::FORBIDDEN {
        return Err(crate::Error::PasswordLocked);
    }
    debug!("{stu_id} 发送了登录请求");
    // login_params里面的pv0在后面的请求也会有用(netflow)
    let addition: Vec<String> = login_params
        .cookies
        .into_iter()
        .filter(|cookie| cookie.starts_with("_pv0="))
        .collect(); // 错误已在前面被处理，一定会有_pv0
    let mut cookies =
        cookie_parser(login.headers().get_all(SET_COOKIE));
    cookies.extend(addition);
    let location = login
        .headers()
        .get(LOCATION)
        .ok_or(crate::Error::PasswordError)?
        .to_str()?
        .to_string();
    const PASSWORD_SHOULD_CHANGE_PAT: &str =
        "cas.hnu.edu.cn/securitycenter/modifyPwd/index.zf";
    if location.contains(PASSWORD_SHOULD_CHANGE_PAT) {
        return Err(crate::Error::PasswordShouldChange);
    }
    let to_store = cookies.join("; ");
    let to_return = Ok(location);
    if password.is_none() {
        CACHE.insert((CasCookie, stu_id.into()), to_store).await;
    }
    to_return
}

/// 登录形如 <http://cas.hnu.edu.cn/application/sso.zf?login=B5712DC2FA281C96E053026B3E0A80A6> 这样的链接的服务
///
/// 最终将会返回一个 (s_ticket, cookies)，用于后续操作
///
/// 这种服务一般是出现在个人门户中，可以从个人门户直接免二次登录跳转过去，比如校园网和体测就是这样的
///
/// 在之前这里的代码是写在具体的 `netflow_headers` 和 `gym_headers_from_cas` 里面的，并且可能有不同的登录方式
///
/// 后来先是校园网系统需要用到这里这个函数的逻辑来进行登录，见 `5380fc7`
///
/// 然后发现体测系统也需要了，就把这个逻辑抽取出来放在这里了
///
/// 还发现，这里的重定向次数似乎是因人而异的，原因不明
pub async fn get_sticket(
    stu_id: &str,
    url: &str,
) -> Result<(String, String), crate::Error> {
    // 先请求一下，防止还没登录。拿到登录后的 cookies
    get_ticket_url(stu_id, SERVICE_URL, None).await?;
    let cas_cache = CACHE
        .get(&(CasCookie, stu_id.into()))
        .await
        .unwrap_or_default();
    // 后面可能会进行多次重定向才能拿到 s_ticket，由于目前 client
    // 关闭了跟随重定向，所以我们手动模拟
    let mut now_url = url.to_string();
    let mut cookies = cas_cache;
    let mut s_ticket = None;
    // 分析的是大概重定向 4 次就可以拿到 s_ticket，为了保险起见多循环几次（中间拿到 s_ticket
    // 就会 break）
    for _ in 0..6 {
        if now_url.starts_with(
            "https://cas.hnu.edu.cn/sprcialapp/zf_form/index.zf",
        ) {
            s_ticket = Some(
                now_url
                    .split('&')
                    .find(|s| s.starts_with("s_ticket="))
                    .ok_or(anyhow!("获取s_ticket失败"))?
                    .split('=')
                    .collect::<Vec<&str>>()[1],
            );
            break;
        }
        let res = client
            .get(now_url)
            .header(COOKIE, &cookies)
            .send()
            .await?
            .error_for_status()?;
        if res.status() != StatusCode::FOUND {
            return Err(anyhow!(
                "获取s_ticket时失败，HTTP代码 {}",
                res.status()
            )
            .into());
        }
        now_url = res
            .headers()
            .get(LOCATION)
            .ok_or(anyhow!("获取重定向链接失败"))?
            .to_str()?
            .to_string();
        cookies = format!(
            "{}; {}",
            cookies,
            cookie_parser(res.headers().get_all(SET_COOKIE))
                .join("; ")
        );
    }
    if let Some(v) = s_ticket {
        Ok((v.to_string(), cookies))
    } else {
        Err(anyhow!("获取s_ticket失败，未找到s_ticket").into())
    }
}
