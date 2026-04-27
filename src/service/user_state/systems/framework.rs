use super::super::cache::{CACHE, CacheEnum};
use crate::result::{AppError, AppResult, throw_error};
use hnu_query::Error as SpiderError;

/// 重试时遇到错误时的下一个动作
pub enum NextAction {
    /// 继续重试
    Retry,
    /// 中断重试，将最后一次错误返回
    Break,
    /// 刷新令牌
    Refresh,
}

pub trait HnuSystem {
    type Token: Clone;
    /// 该系统在请求时需要特殊处理的错误类型
    type Error: std::error::Error;
    /// 系统名称，用于显示在日志中
    fn name() -> &'static str;
    /// 对应的缓存 key
    fn cache_key() -> CacheEnum;
    /// 当前实例对应的学号
    fn stu_id(&self) -> &str;
    /// 获取令牌
    async fn acquire_token(&mut self) -> AppResult<Self::Token>;
    /// 序列化令牌
    fn serialize_token(
        &mut self,
        token: &Self::Token,
    ) -> AppResult<String>;
    /// 反序列化令牌
    fn deserialize_token(
        &mut self,
        s: &str,
    ) -> AppResult<Self::Token>;
    /// 重试的处理
    ///
    /// # Arguments
    ///
    /// - `tried_count`: 已经重试的次数
    /// - `error`: 本次导致重试的错误
    fn handle_retry(
        &mut self,
        tried_count: usize,
        error: &SpiderError<Self::Error>,
    ) -> NextAction;
}

pub async fn with_token<S, F, R>(mut system: S, f: F) -> AppResult<R>
where
    S: HnuSystem,
    F: AsyncFn(S::Token) -> Result<R, SpiderError<S::Error>>
        + 'static,
{
    let serialized_token = CACHE
        .try_get_with(
            (S::cache_key(), system.stu_id().to_string()),
            async {
                let token = system.acquire_token().await?;
                let serialized = system.serialize_token(&token)?;
                Ok::<_, AppError>(serialized)
            },
        )
        .await?;
    let mut token = system.deserialize_token(&serialized_token)?;
    let mut retry_count = 0;
    loop {
        match f(token.clone()).await {
            Ok(res) => {
                if retry_count > 0 {
                    tracing::warn!(
                        "在第 {} 次重试后成功请求 {}",
                        retry_count,
                        S::name()
                    );
                }
                return Ok(res);
            }
            Err(e) => match system.handle_retry(retry_count, &e) {
                NextAction::Retry => {
                    tracing::warn!(
                        error = ?e,
                        "在第 {} 次重试后请求 {} 失败",
                        retry_count,
                        S::name()
                    );
                    retry_count += 1;
                    continue;
                }
                NextAction::Break => {
                    return Err(throw_error(
                        e,
                        &format!("请求 {} 失败", S::name()),
                    ));
                }
                NextAction::Refresh => {
                    tracing::warn!(
                        error = ?e,
                        "在第 {} 次重试后请求 {} 失败，尝试刷新令牌",
                        retry_count,
                        S::name()
                    );
                    let new_token = system.acquire_token().await?;
                    let serialized =
                        system.serialize_token(&new_token)?;
                    CACHE
                        .insert(
                            (
                                S::cache_key(),
                                system.stu_id().to_string(),
                            ),
                            serialized,
                        )
                        .await;
                    token = new_token;
                    continue;
                }
            },
        }
    }
}
