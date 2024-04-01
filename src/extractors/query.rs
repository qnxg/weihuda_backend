//! 自定义Query的解析，用于覆盖默认的解析错误处理逻辑，使Query解析时候返回的信息与程序Api格式保持一致，参照axum::extract::Query的实现。

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, Uri},
};
use serde::de::DeserializeOwned;

use crate::app_error::AppError;

/// 解析请求中的参数对信息
#[derive(Clone, Copy, Default, Debug)]
#[must_use]
pub struct Query<T>(pub T);

// 复用Axum的QueryExtractor，只是在解析失败时返回自定义的错误
#[async_trait]
impl<T, S> FromRequestParts<S> for Query<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Self::try_from_uri(&parts.uri)
    }
}

impl<T> Query<T>
where
    T: DeserializeOwned,
{
    pub fn try_from_uri(value: &Uri) -> Result<Self, AppError> {
        let query = value.query().unwrap_or_default();
        let params = serde_urlencoded::from_str(query)?;
        Ok(Query(params))
    }
}

// axum_core::__impl_deref!(Query);
