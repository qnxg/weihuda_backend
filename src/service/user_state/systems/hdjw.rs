use super::{
    super::{cache::CacheEnum, utils::SerializableHeaderMap},
    MAX_RETRY_COUNT,
    framework::{HnuSystem, NextAction},
    with_cas_token,
};
use crate::result::{AppResult, ThrowError};
use hnu_query::hdjw::login::HdjwToken;
use hnu_query::{Error as SpiderError, hdjw::error::TokenExpired};

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
    fn cache_key() -> CacheEnum {
        CacheEnum::HdjwToken
    }
    fn stu_id(&self) -> &str {
        self.stu_id.as_str()
    }
    async fn acquire_token(&mut self) -> AppResult<HdjwToken> {
        with_cas_token(self.stu_id.as_str(), async |cas_token| {
            HdjwToken::acquire_by_cas_login(cas_token).await
        })
        .await
    }
    fn serialize_token(
        &mut self,
        token: &HdjwToken,
    ) -> AppResult<String> {
        let headers_wrapped =
            SerializableHeaderMap::new(token.headers().clone());
        serde_json::to_string(&headers_wrapped)
            .throw_error("序列化 hdjw HeaderMap 失败")
    }
    fn deserialize_token(
        &mut self,
        serialized: &str,
    ) -> AppResult<HdjwToken> {
        let header =
            serde_json::from_str::<SerializableHeaderMap>(serialized)
                .throw_error("反序列化 hdjw HeaderMap 失败")?;
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
            SpiderError::NetworkError(..)
            | SpiderError::Unexpected { .. } => NextAction::Retry,
            SpiderError::ParseError { .. } => {
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
