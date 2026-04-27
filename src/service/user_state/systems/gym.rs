use super::{
    super::{cache::CacheEnum, utils::SerializableHeaderMap},
    default_retry_strategy,
    framework::{HnuSystem, NextAction},
    with_cas_token,
};
use crate::{
    result::{AppResult, ThrowError},
    service::{self},
};
use hnu_query::{
    Error as SpiderError,
    gym::{error::TokenExpired, login::GymToken},
};

pub struct Gym {
    stu_id: String,
    token_expired_flag: bool,
}

impl Gym {
    pub fn new(stu_id: impl Into<String>) -> Self {
        Self {
            stu_id: stu_id.into(),
            token_expired_flag: false,
        }
    }
}

impl HnuSystem for Gym {
    type Token = GymToken;
    type Error = TokenExpired;
    fn name() -> &'static str {
        "体测系统"
    }
    fn cache_key() -> CacheEnum {
        CacheEnum::GymToken
    }
    fn stu_id(&self) -> &str {
        self.stu_id.as_str()
    }
    async fn acquire_token(&mut self) -> AppResult<GymToken> {
        // 体测系统支持两种登录方式，这里先试通过 cas 登录
        // 如果失败了再换体测系统本身的账号密码登录
        match with_cas_token(
            self.stu_id.as_str(),
            async |cas_token| {
                GymToken::acquire_by_cas_login(cas_token).await
            },
        )
        .await
        {
            Ok(token) => Ok(token),
            Err(_) => {
                let password = service::user_info::get_password(
                    self.stu_id.as_str(),
                )
                .await?;
                GymToken::acquire_by_direct_login(
                    self.stu_id.as_str(),
                    &password,
                )
                .await
                .throw_error("直接登录体测系统失败")
            }
        }
    }
    fn serialize_token(
        &mut self,
        token: &GymToken,
    ) -> AppResult<String> {
        let headers_wrapped =
            SerializableHeaderMap::new(token.headers().clone());
        serde_json::to_string(&headers_wrapped)
            .throw_error("序列化 gym HeaderMap 失败")
    }
    fn deserialize_token(
        &mut self,
        serialized: &str,
    ) -> AppResult<GymToken> {
        let header =
            serde_json::from_str::<SerializableHeaderMap>(serialized)
                .throw_error("反序列化 gym HeaderMap 失败")?;
        Ok(GymToken::from_headers_unchecked(header.into_inner()))
    }
    fn handle_retry(
        &mut self,
        retry_count: usize,
        error: &SpiderError<TokenExpired>,
    ) -> NextAction {
        default_retry_strategy(
            &mut self.token_expired_flag,
            retry_count,
            error,
        )
    }
}
