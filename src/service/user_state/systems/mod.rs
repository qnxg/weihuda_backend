pub mod ca;
pub mod framework;
pub mod gym;
pub mod hdjw;
pub mod lab;
pub mod netflow;
pub mod pt;
pub mod xgxt;
pub mod yjsxt;

use std::sync::LazyLock;

use super::cache::{CACHE, CacheEnum};
use crate::{
    result::{AppError, AppResult, throw_error},
    service::{
        self,
        user_state::{account_tag::ACCOUNT_TAG, tfa::TFA_TOKEN},
    },
    utils::seg_lock::SegLock,
};
use framework::NextAction;
use hnu_query::cas::login::CasToken;
use hnu_query::{Error as SpiderError, cas::login::AccountIssue};

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
    async fn handle_error(
        stu_id: &str,
        e: &SpiderError<AccountIssue>,
    ) -> AppError {
        match e {
            SpiderError::Other(AccountIssue::PasswordError) => {
                ACCOUNT_TAG
                    .insert(
                        stu_id.to_string(),
                        AppError::PasswordError,
                    )
                    .await;
                AppError::PasswordError
            }
            SpiderError::Other(
                AccountIssue::PasswordShouldChange,
            ) => {
                let e: AppError =
                    "请前往个人门户修改密码后重试".into();
                ACCOUNT_TAG
                    .insert(stu_id.to_string(), e.clone())
                    .await;
                e
            }
            SpiderError::Other(AccountIssue::AccountLocked) => {
                let e: AppError =
                    "账号被锁定，请10分钟之后再试".into();
                ACCOUNT_TAG
                    .insert(stu_id.to_string(), e.clone())
                    .await;
                e
            }
            SpiderError::Other(AccountIssue::TFARequired(
                tfa_token,
            )) => {
                TFA_TOKEN
                    .insert(stu_id.to_string(), tfa_token.clone())
                    .await;
                AppError::Text("需要双因子认证".to_string())
            }
            err => throw_error(
                err,
                "with_cas_token 初始化缓存时发生错误",
            ),
        }
    }
    // TODO 每次请求都获取一次密码，会不会有性能问题？
    let password = service::user_info::get_password(stu_id).await?;
    let mut f_result = None;
    // TODO 更细颗粒度的加锁
    let _guard = USER_LOCK.lock(stu_id).await;
    // 上一次就出现了登录问题，直接返回
    if let Some(err) = ACCOUNT_TAG.get(stu_id).await {
        return Err(err);
    };
    // 需要 TFA
    if TFA_TOKEN.contains_key(stu_id) {
        return Err(AppError::Text("需要双因子认证".to_string()));
    }
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
        .await;
    let Ok(cookies) = cookies else {
        let e = cookies.expect_err("cookies 为 Ok");
        return Err(handle_error(stu_id, &e).await);
    };
    if let Some(f_result) = f_result {
        return Ok(f_result);
    }
    let mut cas_token =
        CasToken::from_cookie_unchecked(&cookies, stu_id, &password);
    // TODO 这里可能还是会出现 cas_token 过期，然后此时多个并发请求过来反复更新 cas_token 的情况
    // 后面需要进一步优化
    // https://github.com/qnxg/hnu_query/issues/26
    let f_result = f(&mut cas_token).await;
    let Ok(f_result) = f_result else {
        let e = f_result.err().expect("f_result 为 Ok");
        return Err(handle_error(stu_id, &e).await);
    };
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

pub static USER_LOCK: LazyLock<SegLock<1000>> =
    LazyLock::new(SegLock::new);
