use std::convert::Infallible;

use super::{
    super::{cache::CacheEnum, utils::SerializableHeaderMap},
    default_retry_strategy,
    framework::{HnuSystem, NextAction},
};
use crate::{
    infra::captcha::LabCaptchaResolver,
    result::{AppError, AppResult, ThrowError, throw_error},
    service::{self},
};
use hnu_query::{
    Error as SpiderError,
    lab::{login::LabToken, login::LoginIssue},
};

pub struct Lab {
    stu_id: String,
    token_expired_flag: bool,
}

impl Lab {
    pub fn new(stu_id: impl Into<String>) -> Self {
        Self {
            stu_id: stu_id.into(),
            token_expired_flag: false,
        }
    }
}

impl HnuSystem for Lab {
    type Token = LabToken;
    type Error = Infallible;
    fn name() -> &'static str {
        "大物实验系统"
    }
    fn cache_key() -> CacheEnum {
        CacheEnum::LabToken
    }
    fn stu_id(&self) -> &str {
        self.stu_id.as_str()
    }
    async fn acquire_token(&mut self) -> AppResult<LabToken> {
        const MAX_TRIED: usize = 5;
        let password = service::user_info::get_lab_password(
            self.stu_id.as_str(),
        )
        .await?;
        match LabToken::acquire_by_login(
            self.stu_id.as_str(),
            &password,
            &LabCaptchaResolver,
            MAX_TRIED,
        )
        .await
        {
            Ok(token) => Ok(token),
            Err(SpiderError::Other(LoginIssue::CaptchaError)) => {
                Err(AppError::Text("登录失败，请重试".to_string()))
            }
            Err(SpiderError::Other(LoginIssue::PasswordError)) => {
                Err(AppError::PasswordError)
            }
            Err(SpiderError::Other(LoginIssue::OtherError(text))) => {
                tracing::warn!("验证码识别失败");
                Err(AppError::Text(
                    text.unwrap_or("登录时发生未知错误".to_string()),
                ))
            }
            Err(e) => Err(throw_error(e, "登录大物实验系统失败")),
        }
    }
    fn serialize_token(
        &mut self,
        token: &LabToken,
    ) -> AppResult<String> {
        let headers_wrapped =
            SerializableHeaderMap::new(token.headers().clone());
        serde_json::to_string(&headers_wrapped)
            .throw_error("序列化 lab HeaderMap 失败")
    }
    fn deserialize_token(
        &mut self,
        serialized: &str,
    ) -> AppResult<LabToken> {
        let header =
            serde_json::from_str::<SerializableHeaderMap>(serialized)
                .throw_error("反序列化 lab HeaderMap 失败")?;
        Ok(LabToken::from_headers_unchecked(
            header.into_inner(),
            self.stu_id.as_str(),
        ))
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
