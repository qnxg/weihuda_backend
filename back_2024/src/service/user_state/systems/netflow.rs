use std::convert::Infallible;

use super::{
    super::{cache::CacheEnum, utils::SerializableHeaderMap},
    default_retry_strategy,
    framework::{HnuSystem, NextAction},
    with_cas_token,
};
use crate::result::{AppResult, ThrowError};
use spider_2024::{
    Error as SpiderError, netflow::login::NetflowToken,
};

pub struct Netflow {
    stu_id: String,
    token_expired_flag: bool,
}

impl Netflow {
    pub fn new(stu_id: impl Into<String>) -> Self {
        Self {
            stu_id: stu_id.into(),
            token_expired_flag: false,
        }
    }
}

impl HnuSystem for Netflow {
    type Token = NetflowToken;
    type Error = Infallible;
    fn name() -> &'static str {
        "校园网流量系统"
    }
    fn cache_key() -> CacheEnum {
        CacheEnum::NetflowToken
    }
    fn stu_id(&self) -> &str {
        self.stu_id.as_str()
    }
    async fn acquire_token(&mut self) -> AppResult<NetflowToken> {
        with_cas_token(self.stu_id.as_str(), async |token| {
            NetflowToken::acquire_by_cas_login(token).await
        })
        .await
    }
    fn serialize_token(
        &mut self,
        token: &NetflowToken,
    ) -> AppResult<String> {
        let headers_wrapped =
            SerializableHeaderMap::new(token.headers().clone());
        serde_json::to_string(&headers_wrapped)
            .throw_error("序列化 netflow HeaderMap 失败")
    }
    fn deserialize_token(
        &mut self,
        serialized: &str,
    ) -> AppResult<NetflowToken> {
        let header =
            serde_json::from_str::<SerializableHeaderMap>(serialized)
                .throw_error("反序列化 netflow HeaderMap 失败")?;
        Ok(NetflowToken::from_headers_unchecked(header.into_inner()))
    }
    fn handle_retry(
        &mut self,
        retry_count: usize,
        error: &SpiderError<Infallible>,
    ) -> NextAction {
        default_retry_strategy(
            &mut self.token_expired_flag,
            retry_count,
            error,
        )
    }
}
