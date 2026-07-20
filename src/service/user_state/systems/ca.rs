use std::convert::Infallible;

use super::{
    super::utils::SerializableHeaderMap,
    default_retry_strategy,
    framework::{HnuSystem, NextAction},
    with_cas_token,
};
use crate::error::{AppResult, ThrowInternalErrorResult};
use hnu_query::{Error as SpiderError, ca::login::CaToken};

pub struct Ca {
    stu_id: String,
    token_expired_flag: bool,
}

impl Ca {
    pub fn new(stu_id: impl Into<String>) -> Self {
        Self {
            stu_id: stu_id.into(),
            token_expired_flag: false,
        }
    }
}

impl HnuSystem for Ca {
    type Token = CaToken;
    type Error = Infallible;
    fn name() -> &'static str {
        "可信电子凭证系统"
    }
    fn stu_id(&self) -> &str {
        self.stu_id.as_str()
    }
    async fn acquire_token(&mut self) -> AppResult<CaToken> {
        with_cas_token(self.stu_id.as_str(), async |token| {
            CaToken::acquire_by_cas_login(token).await
        })
        .await
    }
    fn serialize_token(
        &mut self,
        token: &CaToken,
    ) -> AppResult<String> {
        let headers_wrapped =
            SerializableHeaderMap::new(token.headers().clone());
        serde_json::to_string(&headers_wrapped).internal_err()
    }
    fn deserialize_token(
        &mut self,
        serialized: &str,
    ) -> AppResult<CaToken> {
        let header =
            serde_json::from_str::<SerializableHeaderMap>(serialized)
                .internal_err()?;
        Ok(CaToken::from_headers_unchecked(header.into_inner()))
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
