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

/// 获取学校对应系统的令牌并进行请求。框架会将自动处理令牌缓存，缓存过期等问题。
///
/// # Arguments
///
/// - `system`：学校对应系统的示例
/// - `f`：请求函数，该参数是一个普通闭包，闭包需要接受学校对应系统的令牌 [HnuSystem::Token]，
///   并返回一个 Future。
///   - 请求函数本身和返回的 Future 需要是 Send 的，因为 salvo 要求 router 层的 handler
///     需要是 Send 的。
///   - Future 的 Output 需要是一个 Result，其中 Err 的错误类型需要是 [SpiderError]<[HnuSystem::Error]>。
///     一般闭包内直接把调用 hnu_query 的函数抛出的错误再往上抛就行了。
///   - 请求函数需要是 [Fn] 闭包，因为请求函数抛出错误后，该框架会调用对应系统的错误处理逻辑，
///     来决定接下来的行为，可能会重试，因此闭包会被多次调用。
///
/// # Returns
///
/// 该函数会把 `f` 返回的 Future 的输出返回出去。
///
/// # Errors
///
/// - [HnuSystem::serialize_token]、[HnuSystem::deserialize_token] 失败时会抛出错误
/// - [HnuSystem::acquire_token] 失败时会抛出错误
/// - [HnuSystem::handle_retry] 返回 [NextAction::Break] 时会把最后一次 `f` 返回的错误抛出
///
/// # Examples
///
/// ```rust,ignore
/// let spider_res = with_token(Hdjw::new(stu_id), |token| {
///     let jx0404id_value = &jx0404id_value;
///     async move {
///         hnu_query::hdjw::get_grade_detail(
///             &token,
///             jx0404id_value.as_str(),
///         )
///         .await
///     }
/// });
/// ```
///
/// - 返回的 async block 必须是 `async move` 的，因为闭包返回的 async block 不能引用 `token`，
///   必须直接拿其所有权。
/// - 如果 async block 中需要捕获外部的非 Copy 的值（比如这里的 `jx0404id_value`，是个 String）
///   那么需要在返回 async block 前搞一个 `let jx0404id_value = &jx0404id_value;` 这种代码，
///   来提示 Rust 外层的这个闭包需要按引用捕获外部的值。
///
/// # Notes
///
/// `f` 的类型并没有选择 [AsyncFn]，这是因为 [AsyncFn] 有 GAT，Rust 在判断其是否是 Send 时，
/// 会按高阶生命周期约束，要求 [AsyncFn] 对于任意的生命周期都是 Send 的。当 [AsyncFn] 捕获了
/// 外部的引用时，Rust 会认为该 [AsyncFn] 的生命周期只能在外部的引用的生命周期下才是 Send 的，
/// 在其他更大的生命周期下不是 Send 的，于是判断整个 [AsyncFn] 不是 Send 的。因此，这意味着
/// 我们的 `f` 不能捕获外部的引用，必须 Clone + 传所有权。
///
/// [AsyncFn] 出现这样的问题应该是 Rust 编译器自身的缺陷。
///
/// 我们现在的这个返回 Future 的方案中，由于 [Fn] 没有 GAT，所以 Rust 不会用高阶生命周期推导。
///
/// # References
///
/// 可以参考 Rust 的闭包机制，async block，GAT，高阶生命周期约束等内容。
pub async fn with_token<S, F, Fut, R>(
    mut system: S,
    f: F,
) -> AppResult<R>
where
    S: HnuSystem,
    F: Fn(S::Token) -> Fut + Send,
    Fut: Future<Output = Result<R, SpiderError<S::Error>>> + Send,
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
