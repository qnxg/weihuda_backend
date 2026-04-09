use anyhow::anyhow;
use reqwest::Response;
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::Value;

use crate::utils::cache::{CACHE, CacheEnum};

#[derive(Deserialize, Debug)]
pub struct BadGymResponse {
    pub data: Value,
    pub info: String,
    pub status: i64,
}

pub trait GymResponseExtractor {
    async fn extract_data<T: DeserializeOwned>(
        self,
    ) -> Result<Result<T, BadGymResponse>, crate::Error>;
}

impl GymResponseExtractor for Response {
    async fn extract_data<T: DeserializeOwned>(
        self,
    ) -> Result<Result<T, BadGymResponse>, crate::Error> {
        let body = self.text().await?;
        let res: BadGymResponse = serde_json::from_str(&body)
            .map_err(|e| {
                anyhow!(
                    "解析体测平台响应失败: body = {}, error = {:?}",
                    body,
                    e
                )
            })?;
        if res.status == 1 {
            match serde_json::from_value::<T>(res.data) {
                Ok(data) => return Ok(Ok(data)),
                Err(e) => {
                    return Err(anyhow!(
                        "解析体测平台响应失败: body = {}, error = {:?}", body, e
                    ).into());
                }
            }
        }
        Ok(Err(res))
    }
}

pub trait GymResponse<T> {
    async fn check_cache(self, stu_id: &str) -> Self;
    fn into_result(self) -> Result<T, crate::Error>;
}

impl<T> GymResponse<T> for Result<T, BadGymResponse> {
    /// 检查该响应是否表明 cookie 过期，如果是的话则将 cookie 缓存清除
    async fn check_cache(self, stu_id: &str) -> Self {
        // 典型的异常response body：
        // {"data":[],"info":"登录失效","status":-1}
        if let Err(ref bad_resp) = self
            && bad_resp.info.contains("登录失效")
        {
            CACHE
                .invalidate(&(CacheEnum::GymCookie, stu_id.into()))
                .await;
        }
        self
    }
    fn into_result(self) -> Result<T, crate::Error> {
        match self {
            Ok(value) => Ok(value),
            Err(bad_resp) => {
                Err(anyhow!("体测平台响应失败: {:?}", bad_resp)
                    .into())
            }
        }
    }
}
