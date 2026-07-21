use std::{convert::Infallible, time::Duration};

use super::{
    super::utils::SerializableHeaderMap,
    default_retry_strategy,
    framework::{HnuSystem, NextAction},
};
use crate::{
    error::{
        AppError, AppResult, ThrowInternalError,
        ThrowInternalErrorResult,
    },
    infra::captcha::LabCaptchaResolver,
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
    fn ttl() -> Duration {
        Duration::from_mins(10)
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
                Err(AppError::customized("验证码错误"))
            }
            Err(SpiderError::Other(LoginIssue::PasswordError)) => {
                Err(AppError::password_error())
            }
            Err(SpiderError::Other(LoginIssue::OtherError(text))) => {
                Err(AppError::customized(
                    text.unwrap_or("登录时发生未知错误".to_string()),
                ))
            }
            Err(e) => Err(e.internal_err().into()),
        }
    }
    fn serialize_token(
        &mut self,
        token: &LabToken,
    ) -> AppResult<String> {
        let headers_wrapped =
            SerializableHeaderMap::new(token.headers().clone());
        serde_json::to_string(&headers_wrapped).internal_err()
    }
    fn deserialize_token(
        &mut self,
        serialized: &str,
    ) -> AppResult<LabToken> {
        let header =
            serde_json::from_str::<SerializableHeaderMap>(serialized)
                .internal_err()?;
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
