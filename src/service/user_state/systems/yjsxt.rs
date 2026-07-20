use super::{
    MAX_RETRY_COUNT,
    framework::{HnuSystem, NextAction},
    with_cas_token,
};
use crate::error::AppResult;
use hnu_query::yjsxt::login::YjsxtToken;
use hnu_query::{Error as SpiderError, yjsxt::error::TokenExpired};

pub struct Yjsxt {
    stu_id: String,
    token_expired_flag: bool,
}

impl Yjsxt {
    pub fn new(stu_id: impl Into<String>) -> Self {
        Self {
            stu_id: stu_id.into(),
            token_expired_flag: false,
        }
    }
}

impl HnuSystem for Yjsxt {
    type Token = YjsxtToken;
    type Error = TokenExpired;
    fn name() -> &'static str {
        "研究生系统"
    }
    fn stu_id(&self) -> &str {
        self.stu_id.as_str()
    }
    async fn acquire_token(&mut self) -> AppResult<YjsxtToken> {
        with_cas_token(self.stu_id.as_str(), async |cas_token| {
            YjsxtToken::acquire_by_cas_login(cas_token).await
        })
        .await
    }
    fn serialize_token(
        &mut self,
        token: &YjsxtToken,
    ) -> AppResult<String> {
        Ok(token.id().to_string())
    }
    fn deserialize_token(
        &mut self,
        serialized: &str,
    ) -> AppResult<YjsxtToken> {
        Ok(YjsxtToken::from_id_unchecked(serialized))
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
                if self.token_expired_flag {
                    return NextAction::Break;
                }
                self.token_expired_flag = true;
                NextAction::Refresh
            }
            SpiderError::Other(TokenExpired) => NextAction::Refresh,
        }
    }
}
