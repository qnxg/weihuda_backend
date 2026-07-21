use super::{
    super::utils::SerializableHeaderMap,
    MAX_RETRY_COUNT,
    framework::{HnuSystem, NextAction},
    with_cas_token,
};
use crate::error::{AppResult, ThrowInternalErrorResult};
use hnu_query::hdjw::login::HdjwToken;
use hnu_query::{Error as SpiderError, hdjw::error::TokenExpired};
use std::sync::LazyLock;
use std::{collections::VecDeque, time::Duration};
use tokio::sync::Mutex;

/// 号池最大容量
const POOL_MAX_SIZE: usize = 5;

/// 存放最近登录成功过的学号
///
/// 考虑到号池不大，直接使用 VecDeque
pub static TOKEN_POOL: LazyLock<Mutex<VecDeque<String>>> =
    LazyLock::new(|| Mutex::new(VecDeque::new()));

pub struct Hdjw {
    stu_id: String,
    token_expired_flag: bool,
}

impl Hdjw {
    pub fn new(stu_id: impl Into<String>) -> Self {
        Self {
            stu_id: stu_id.into(),
            token_expired_flag: false,
        }
    }
}

impl HnuSystem for Hdjw {
    type Token = HdjwToken;
    type Error = TokenExpired;
    fn name() -> &'static str {
        "教务系统"
    }
    fn ttl() -> Duration {
        Duration::from_mins(30)
    }
    fn stu_id(&self) -> &str {
        self.stu_id.as_str()
    }
    async fn acquire_token(&mut self) -> AppResult<HdjwToken> {
        let res =
            with_cas_token(self.stu_id.as_str(), async |cas_token| {
                // 把 HdjwToken 缓存
                HdjwToken::acquire_by_cas_login(cas_token).await
            })
            .await;
        let mut pool = TOKEN_POOL.lock().await; // 拿锁
        // 如果账号已经在号池内就不添加了
        if res.is_ok() && !pool.iter().any(|x| x == &self.stu_id) {
            // 为了维护号池内账号始终是新的，淘汰旧账号以添加新账号
            if pool.len() >= POOL_MAX_SIZE {
                pool.pop_front(); // 前出
            }
            pool.push_back(self.stu_id.clone()); // 后进
        }
        res
    }
    fn serialize_token(
        &mut self,
        token: &HdjwToken,
    ) -> AppResult<String> {
        let headers_wrapped =
            SerializableHeaderMap::new(token.headers().clone());
        serde_json::to_string(&headers_wrapped).internal_err()
    }
    fn deserialize_token(
        &mut self,
        serialized: &str,
    ) -> AppResult<HdjwToken> {
        let header =
            serde_json::from_str::<SerializableHeaderMap>(serialized)
                .internal_err()?;
        Ok(HdjwToken::from_headers_unchecked(header.into_inner()))
    }
    fn handle_retry(
        &mut self,
        retry_count: usize,
        error: &SpiderError<TokenExpired>,
    ) -> NextAction {
        if retry_count > MAX_RETRY_COUNT {
            return NextAction::Break;
        }
        match error {
            SpiderError::Network(_) | SpiderError::Unexpected(_) => {
                NextAction::Retry
            }
            SpiderError::Parse(_) => {
                // 解析错误可能是由于 token 过期
                if self.token_expired_flag {
                    // 已经过期过了，说明不太可能是令牌过期导致解析错误的
                    return NextAction::Break;
                }
                self.token_expired_flag = true;
                NextAction::Refresh
            }
            SpiderError::Other(TokenExpired) => NextAction::Refresh,
        }
    }
}
