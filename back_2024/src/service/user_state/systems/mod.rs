pub mod ca;
pub mod framework;
pub mod gym;
pub mod hdjw;
pub mod lab;
pub mod netflow;
pub mod pt;
pub mod xgxt;

use super::cache::{CACHE, CacheEnum};
use crate::{
    result::{AppError, AppResult, throw_error},
    service::{self},
};
use framework::NextAction;
use spider_2024::cas::login::CasToken;
use spider_2024::{Error as SpiderError, cas::login::AccountIssue};

pub const MAX_RETRY_COUNT: usize = 3;

/// 使用 CasToken 进行请求
///
/// # Arguments
///
/// - `stu_id`: 学号
/// - `f`: 处理函数，该函数应使用 CasToken 进行一些登录相关的请求，
///   返回的错误为 [`SpiderError<AccountIssue>`]
async fn with_cas_token<F, R>(stu_id: &str, f: F) -> AppResult<R>
where
    F: AsyncFn(&mut CasToken) -> Result<R, SpiderError<AccountIssue>>,
{
    fn map_e(e: &SpiderError<AccountIssue>) -> AppError {
        match e {
            SpiderError::Other(AccountIssue::PasswordError) => {
                AppError::PasswordError
            }
            SpiderError::Other(
                AccountIssue::PasswordShouldChange,
            ) => "请前往个人门户修改密码后重试".into(),
            SpiderError::Other(AccountIssue::AccountLocked) => {
                "账号被锁定，请10分钟之后再试".into()
            }
            err => throw_error(
                err,
                "with_cas_token 初始化缓存时发生错误",
            ),
        }
    }
    let password = service::user_info::get_password(stu_id).await?;
    let mut f_result = None;
    let cookies = CACHE
        .try_get_with((CacheEnum::CasToken, stu_id.to_string()), async {
            // 初始时对应的 Cookie 必然是空字符串
            let mut cas_token =
                CasToken::from_cookie_unchecked("", stu_id, &password);
            // 现在缓存中没有 CasToken，目前传进来的函数 f 理论上应该会使用这个 CasToken 进行请求
            // 从而会刷新 CasToken 内部的 cookie
            // 所以我们在当前计算缓存的代码块中调用 f，然后再把 CasToken 内刷新的 cookie 写回缓存
            f_result = Some(f(&mut cas_token).await?);
            // TODO 这里如果遇到 AccountIssue，应该进行相应的处理
            let Some(cookie) = cas_token.cookie() else {
                tracing::warn!("调用函数 f 成功后 CasToken 内的 cookie 仍为空");
                // 不写回缓存，这样可以保证缓存内的 CasToken 都是有有效 Cookie 的
                // 这里加一个类型注释来帮助编译器进行类型推导
                return Ok::<_, SpiderError<AccountIssue>>(String::new());
            };
            Ok(cookie.to_string())
        })
        .await
        .map_err(|e| map_e(e.as_ref()))?;
    if let Some(f_result) = f_result {
        return Ok(f_result);
    }
    let mut cas_token =
        CasToken::from_cookie_unchecked(&cookies, stu_id, &password);
    // TODO 这里可能还是会出现 cas_token 过期，然后此时多个并发请求过来反复更新 cas_token 的情况
    // 后面需要进一步优化
    let f_result = f(&mut cas_token).await.map_err(|e| map_e(&e))?;
    // 可能刷新了 CasToken 内部的 cookie，所以需要写回缓存
    CACHE
        .insert(
            (CacheEnum::CasToken, stu_id.to_string()),
            cas_token.cookie().unwrap_or_default().to_string(),
        )
        .await;
    Ok(f_result)
}

/// 默认请求重试策略
///
/// 使用该策略的 HnuSystem 应该在内部维护一个 `token_expired_flag`，用于后面判断是否已经刷新过令牌
///
/// 该策略假定 HnuSystem 不需要处理 [framework::HnuSystem::Error]
///
/// * 最多重试 [MAX_RETRY_COUNT] 次
/// * 如果遇到 [SpiderError::ParseError]，则可能是令牌过期导致返回了爬虫库暂时无法识别出来的内容
///   所以就先大胆假设成令牌过期，刷新令牌重试。如果又遇到 [SpiderError::ParseError]，则大概率是
///   真的解析错误。解析错误一般也没必要重试，直接返回。
/// * 对于 [SpiderError::NetworkError] 和 [SpiderError::Unexpected]，有重试的必要，直接重试。
fn default_retry_strategy<E: std::error::Error>(
    token_expired_flag: &mut bool,
    tried_count: usize,
    error: &SpiderError<E>,
) -> NextAction {
    if tried_count > MAX_RETRY_COUNT {
        return NextAction::Break;
    }
    match error {
        SpiderError::ParseError { .. } => {
            if *token_expired_flag {
                NextAction::Break
            } else {
                *token_expired_flag = true;
                NextAction::Retry
            }
        }
        _ => NextAction::Retry,
    }
}
