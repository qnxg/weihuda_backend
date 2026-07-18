pub mod ca;
pub mod framework;
pub mod gym;
pub mod hdjw;
pub mod lab;
pub mod netflow;
pub mod pt;
pub mod xgxt;
pub mod yjsxt;

use std::{sync::LazyLock, time::Instant};

use super::cache::{CACHE, CacheEnum};
use crate::{
    error::{AppError, AppResult, ThrowInternalError},
    service::{
        self,
        user_state::{account_tag::ACCOUNT_TAG, tfa::TFA_TOKEN},
    },
    utils::{self, seg_lock::SegLock},
};
use framework::NextAction;
use hnu_query::{
    Error as SpiderError,
    cas::{
        error::TokenExpired as CasTokenExpired,
        login::{AccountIssue, CasToken},
    },
};

pub const MAX_RETRY_COUNT: usize = 3;

/// 使用 CasToken 进行请求
///
/// # Arguments
///
/// - `stu_id`: 学号
/// - `f`: 处理函数，该函数应使用 CasToken 进行一些登录相关的请求，
///   返回的错误为 [`SpiderError<CasTokenExpired>`]
#[tracing::instrument(
    skip(f),
    fields(
        name = "with_cas_token",
        otel.kind = "client",
        event_type = "hnu_call",
        // 是否复用了上一次的 token。为 false 说明本次调用刷新了 token
        token_hit = true,
        // 是否是由于 tag 导致直接响应错误的
        tag_hit = false,
        // 获取锁的等待时间，单位：毫秒
        lock_wait = tracing::field::Empty,
        // 持有锁的时间，只在函数成功执行后记录，单位：毫秒
        lock_hold = tracing::field::Empty,
    ),
    err
)]
#[expect(clippy::too_many_lines)]
async fn with_cas_token<F, R>(stu_id: &str, f: F) -> AppResult<R>
where
    F: AsyncFn(&CasToken) -> Result<R, SpiderError<CasTokenExpired>>,
{
    // 获取新的 CasToken，返回 cookie
    async fn get_cas_token(stu_id: &str) -> AppResult<String> {
        utils::record!(token_hit = false);
        let password =
            service::user_info::get_password(stu_id).await?;
        let cas_token =
            CasToken::acquire_by_login(stu_id, &password).await;
        match cas_token {
            Ok(cas_token) => Ok(cas_token.cookie().to_string()),
            Err(e) => match e {
                SpiderError::Other(AccountIssue::PasswordError) => {
                    ACCOUNT_TAG
                        .insert(
                            stu_id.to_string(),
                            AppError::password_error(),
                        )
                        .await;
                    Err(AppError::password_error())
                }
                SpiderError::Other(
                    AccountIssue::PasswordShouldChange,
                ) => {
                    let e: AppError = AppError::customized(
                        "请前往个人门户修改密码后重试",
                    );
                    ACCOUNT_TAG
                        .insert(stu_id.to_string(), e.clone())
                        .await;
                    Err(e)
                }
                SpiderError::Other(AccountIssue::AccountLocked) => {
                    let e: AppError = AppError::customized(
                        "账号被锁定，请10分钟之后再试",
                    );
                    ACCOUNT_TAG
                        .insert(stu_id.to_string(), e.clone())
                        .await;
                    Err(e)
                }
                SpiderError::Other(AccountIssue::TFARequired(
                    tfa_token,
                )) => {
                    TFA_TOKEN
                        .insert(stu_id.to_string(), tfa_token.clone())
                        .await;
                    Err(AppError::customized("需要双因子认证"))
                }
                err => Err(err.internal_err().into()),
            },
        }
    }
    // TODO 更细颗粒度的加锁
    // 这里对学号加锁，确保同一学号同一时刻只有一个请求
    // 这样可以确保不会有多个请求反复触发 AccountIssue（比如反复触发密码错误，导致账号被锁定）
    // 同时可以确保 CasToken 过期的话，不会反复刷新 CasToken
    let timer = Instant::now();
    let _guard = USER_LOCK.lock(stu_id).await;
    utils::record!(lock_wait = timer.elapsed().as_millis());
    let timer = Instant::now();
    // 上一次就出现了登录问题，直接返回
    if let Some(err) = ACCOUNT_TAG.get(stu_id).await {
        utils::record!(tag_hit = true);
        return Err(err);
    };
    // 需要 TFA
    if TFA_TOKEN.contains_key(stu_id) {
        return Err(AppError::customized("需要双因子认证"));
    }
    // 这里只是单纯用一下 moka 的 get_with 如果缓存不命中则刷新的作用，
    // 由于这里对学号加锁，所以 get_with 的同步作用这里没有利用到
    let cookies = CACHE
        .try_get_with(
            (CacheEnum::CasToken, stu_id.to_string()),
            async {
                let cookies = get_cas_token(stu_id).await?;
                Ok(cookies)
            },
        )
        .await?;
    let cas_token = CasToken::from_cookie_unchecked(&cookies, stu_id);
    let f_result = f(&cas_token).await;
    match f_result {
        Ok(res) => {
            utils::record!(lock_hold = timer.elapsed().as_millis());
            return Ok(res);
        }
        Err(e) => match e {
            SpiderError::Other(CasTokenExpired) => {
                // 下面继续刷新
            }
            err => {
                return Err(err.internal_err().into());
            }
        },
    }
    // 走到这里说明旧的 cas_token 过期了，需要刷新
    let cookies = get_cas_token(stu_id).await?;
    CACHE
        .insert(
            (CacheEnum::CasToken, stu_id.to_string()),
            cookies.clone(),
        )
        .await;
    let cas_token = CasToken::from_cookie_unchecked(&cookies, stu_id);
    let f_result = f(&cas_token).await;
    match f_result {
        Ok(res) => {
            utils::record!(lock_hold = timer.elapsed().as_millis());
            Ok(res)
        }
        Err(e) => match e {
            SpiderError::Other(CasTokenExpired) => {
                Err(e.internal_err().into())
            }
            err => Err(err.internal_err().into()),
        },
    }
}

/// 默认请求重试策略
///
/// 使用该策略的 HnuSystem 应该在内部维护一个 `token_expired_flag`，用于后面判断是否已经刷新过令牌
///
/// 该策略假定 HnuSystem 不需要处理 [framework::HnuSystem::Error]
///
/// * 最多重试 [MAX_RETRY_COUNT] 次
/// * 如果遇到 [SpiderError::Parse]，则可能是令牌过期导致返回了爬虫库暂时无法识别出来的内容
///   所以就先大胆假设成令牌过期，刷新令牌重试。如果又遇到 [SpiderError::Parse]，则大概率是
///   真的解析错误。解析错误一般也没必要重试，直接返回。
/// * 对于 [SpiderError::Network] 和 [SpiderError::Unexpected]，有重试的必要，直接重试。
fn default_retry_strategy<E: std::error::Error>(
    token_expired_flag: &mut bool,
    tried_count: usize,
    error: &SpiderError<E>,
) -> NextAction {
    if tried_count > MAX_RETRY_COUNT {
        return NextAction::Break;
    }
    match error {
        SpiderError::Parse(_) => {
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
