use super::{
    MAX_RETRY_COUNT,
    framework::{HnuSystem, NextAction},
    with_cas_token,
};
use crate::{
    error::{AppResult, ThrowInternalErrorResult},
    service::user_state::utils::SerializableHeaderMap,
};
use hnu_query::{
    Error as SpiderError,
    iportal::{error::IPortalTokenExpired, login::IPortalToken},
};
use std::time::Duration;

pub struct IPortal {
    stu_id: String,
    token_expired_flag: bool,
}

impl IPortal {
    pub fn new(stu_id: impl Into<String>) -> Self {
        Self {
            stu_id: stu_id.into(),
            token_expired_flag: false,
        }
    }
}

impl HnuSystem for IPortal {
    type Token = IPortalToken;
    type Error = IPortalTokenExpired;
    fn name() -> &'static str {
        "新个人门户"
    }
    fn ttl() -> Duration {
        Duration::from_mins(30)
    }
    fn stu_id(&self) -> &str {
        &self.stu_id
    }
    async fn acquire_token(&mut self) -> AppResult<IPortalToken> {
        with_cas_token(self.stu_id.as_str(), async |token| {
            IPortalToken::acquire_by_cas_login(token).await
        })
        .await
    }
    fn serialize_token(
        &mut self,
        token: &IPortalToken,
    ) -> AppResult<String> {
        let headers_wrapped =
            SerializableHeaderMap::new(token.headers().clone());
        serde_json::to_string(&headers_wrapped).internal_err()
    }
    fn deserialize_token(
        &mut self,
        serialized: &str,
    ) -> AppResult<IPortalToken> {
        let header =
            serde_json::from_str::<SerializableHeaderMap>(serialized)
                .internal_err()?;
        Ok(IPortalToken::from_headers_unchecked(header.into_inner()))
    }
    fn handle_retry(
        &mut self,
        retry_count: usize,
        error: &SpiderError<IPortalTokenExpired>,
    ) -> NextAction {
        if retry_count > MAX_RETRY_COUNT {
            return NextAction::Break;
        }
        match error {
            SpiderError::Network(_) | SpiderError::Unexpected(_) => {
                NextAction::Retry
            }
            SpiderError::Parse(_) => {
                if self.token_expired_flag {
                    return NextAction::Break;
                }
                self.token_expired_flag = true;
                NextAction::Refresh
            }
            SpiderError::Other(IPortalTokenExpired) => {
                NextAction::Refresh
            }
        }
    }
}
