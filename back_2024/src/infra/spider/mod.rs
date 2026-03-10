use crate::{
    config::CFG,
    result::{AppError, AppResult},
};
use anyhow::anyhow;
use once_cell::sync::Lazy;
use reqwest::{
    Client,
    header::{AUTHORIZATION, HeaderMap},
    redirect::Policy,
};
use reqwest::{StatusCode, header::LOCATION};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::time::Duration;

pub mod ca;
pub mod electricity;
pub mod gymos;
pub mod hdjw;
pub mod lab;
pub mod netflow;
pub mod pt;
pub mod xgxt;

static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .connection_verbose(false)
        .timeout(Duration::from_secs(6))
        .default_headers({
            let mut headers = HeaderMap::new();
            headers.insert(
                AUTHORIZATION,
                "OUJhbGciOiJIUzU(x7)iIsImlhdCI6MTYxNzQy$jAwMiwiZXh#IjoxNjUzNDI2MDAyfQ@eyI6ImFkbWhjXzxcwEiT7dlm9sFeSRlgY7rnJKpBA"
                    .parse()
                    .expect("解析给定的 Authorization 请求头失败"),
            );
            headers
        })
        .redirect(Policy::none())   // 不使用默认的重定向策略，因为重定向需要保留请求头，而reqwest默认的重定向策略会清除请求头
        .build()
        .expect("构建用于爬虫的 reqwest client 失败")
});

async fn spider_data<T: Serialize, U: DeserializeOwned>(
    path: &str,
    params: &T,
) -> AppResult<U> {
    spider_data_with_timeout(path, params, Duration::from_secs(6))
        .await
}

/// 提取爬虫返回值的data字段
#[inline]
async fn spider_data_with_timeout<
    T: Serialize,
    U: DeserializeOwned,
>(
    path: &str,
    params: &T,
    timeout: Duration,
) -> AppResult<U> {
    let url = format!("{}{}", CFG.service.spider_url, path);

    let mut res = CLIENT
        .get(&url)
        .query(params)
        .timeout(timeout)
        .send()
        .await
        .map_err(|e| {
            tracing::error!(
                error = ?e,
                url = %url,
                "请求爬虫失败",
            );
            AppError::SpiderRequestError(
                "内部错误：内部请求失败".to_string(),
            )
        })?;

    while res.status().is_redirection() {
        let redirect_url = res
            .headers()
            .get(LOCATION)
            .ok_or(AppError::SpiderRequestError(
                format!("爬虫请求要求重定向，但未找到重定向目标地址。请求目标:{}", url)
            ))?
            .to_str()
            .map_err(|e| AppError::SpiderRequestError(format!("爬虫请求要求重定向，但重定向目标地址解析失败。错误信息:{}, 请求目标:{}", e, url)))?;
        res = CLIENT.get(redirect_url).send().await.map_err(|e| {
            tracing::error!(
                error = ?e,
                url = %url,
                "请求爬虫失败",
            );
            AppError::SpiderRequestError(
                "内部错误：内部请求失败".to_string(),
            )
        })?;
    }

    match res.status() {
        StatusCode::UNAUTHORIZED => Err(AppError::PasswordError),
        status => {
            let res_obj = res.text().await.map_err(|e| {
                tracing::error!(
                    error = ?e,
                    url = %url,
                    "解析爬虫响应体失败",
                );
                AppError::SpiderRequestError(
                    "内部错误：内部请求失败。".to_string(),
                )
            })?;

            let mut res_obj: Value = serde_json::from_str(&res_obj)
                .map_err(|e| {
                tracing::error!(
                    error = ?e,
                    url = %url,
                    response = %res_obj,
                    "解析爬虫响应内容到 json 失败",
                );
                AppError::SpiderRequestError(
                    "内部错误：内部请求失败。".to_string(),
                )
            })?;

            let msg = res_obj
                .get("msg")
                .and_then(|v| v.as_str())
                .unwrap_or("未知错误");
            match status {
                StatusCode::OK => {
                    // 目前的实验平台爬虫接口会有意地利用 null 来表示实验平台密码没有绑定/密码错误
                    // if res_obj.get("data").is_none_or(|v| v.is_null()) {
                    //     tracing::error!(
                    //         "爬虫响应状态正常，但是返回没有携带 data 字段，请求目标：{}",
                    //         url
                    //     );
                    //     return Err(AppError::SpiderRequestError(
                    //         "内部错误：内部请求失败。".to_string(),
                    //     ));
                    // }
                    // take()方法将json_res的所有权转移给res
                    let res: U = serde_json::from_value(
                        res_obj["data"].take(),
                    )
                    .map_err(|e| {
                        tracing::error!(
                            error = ?e,
                            url = %url,
                            response = %res_obj,
                            "解析爬虫信息失败",
                        );
                        anyhow!("解析爬虫信息失败")
                    })?;
                    Ok(res)
                }
                _ => {
                    tracing::error!(
                        error = ?msg,
                        url = %url,
                        "爬虫请求失败",
                    );
                    Err(AppError::SpiderRequestError(msg.to_string()))
                }
            }
        }
    }
}
