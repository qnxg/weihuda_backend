use anyhow::anyhow;
use once_cell::sync::Lazy;
use reqwest::{
    header::{HeaderMap, AUTHORIZATION, LOCATION},
    redirect::Policy,
    Client, StatusCode,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::time::Duration;

use crate::{app_error::AppError, config::CFG};

#[expect(non_upper_case_globals)]
pub static client: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .connection_verbose(false)
        .timeout(Duration::from_secs(6))
        .default_headers({
            let mut headers = HeaderMap::new();
            headers.insert(
                AUTHORIZATION,
                "OUJhbGciOiJIUzU(x7)iIsImlhdCI6MTYxNzQy$jAwMiwiZXh#IjoxNjUzNDI2MDAyfQ@eyI6ImFkbWhjXzxcwEiT7dlm9sFeSRlgY7rnJKpBA"
                    .parse()
                    .unwrap(),
            );
            headers
        })
        .redirect(Policy::none())   // 不使用默认的重定向策略，因为重定向需要保留请求头，而reqwest默认的重定向策略会清除请求头
        .build()
        .unwrap()
});

/// 提取爬虫返回值的data字段
#[inline]
pub async fn spider_data<T: Serialize, U: DeserializeOwned>(
    path: &str,
    params: &T,
) -> Result<U, AppError> {
    let url = format!("{}{}", CFG.service.spider_url, path);

    let mut res =
        client.get(&url).query(params).send().await.map_err(|e| {
            tracing::error!(
                "请求爬虫失败，错误信息：{}，请求目标：{}",
                e,
                url
            );
            AppError::SpiderRequestError(
                "内部错误：内部请求失败".to_string(),
            )
        })?;

    while res.status().is_redirection() {
        let redirect_url =
            res.headers().get(LOCATION).unwrap().to_str().unwrap();
        res = client.get(redirect_url).send().await.map_err(|e| {
            tracing::error!(
                "请求爬虫失败，错误信息：{}，请求目标：{}",
                e,
                url
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
                    "解析爬虫响应体失败，错误信息：{}，请求目标：{}",
                    e,
                    url
                );
                AppError::SpiderRequestError(
                    "内部错误：内部请求失败。".to_string(),
                )
            })?;

            let mut res_obj: Value =
                serde_json::from_str(&res_obj).map_err(|e| {
                    tracing::error!(
                    "解析爬虫响应内容到 json 失败，错误信息：{}，请求目标：{}",
                    e,
                    url
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
                    let res: U = serde_json::from_value(res_obj["data"].take())
                        .map_err(|e| {
                            tracing::error!(
                                "解析爬虫信息失败，错误信息：{}，请求目标：{}",
                                e,
                                url
                            );
                            anyhow!("解析爬虫信息失败")
                        })?;
                    Ok(res)
                }
                _ => {
                    tracing::error!(
                        "爬虫请求失败，状态码：{}，错误信息：{}，请求目标：{}",
                        status,
                        msg,
                        url
                    );
                    Err(AppError::SpiderRequestError(msg.to_string()))
                }
            }
        }
    }
}

/// 直接返回爬虫返回的json数据
// #[deprecated(note = "请使用spider_data，新爬虫的返回格式完全与本后端的一致")]
#[inline]
pub async fn spider<T: Serialize, U: DeserializeOwned>(
    path: &str,
    params: &T,
) -> Result<U, anyhow::Error> {
    let url = format!("{}{}", CFG.service.spider_url, path);

    let mut res = client.get(url).query(params).send().await?;

    while res.status().is_redirection() {
        let redirect_url =
            res.headers().get(LOCATION).unwrap().to_str().unwrap();
        res = client.get(redirect_url).send().await?;
    }

    let res = res.text().await?;

    let json_res: Value = serde_json::from_str(&res)?;

    let res: U = serde_json::from_value(json_res).map_err(|e| {
        anyhow::anyhow!(format!("数据获取失败: {}", e))
    })?;

    Ok(res)
}

#[expect(dead_code)]
///访问地址完全自定义
#[inline]
pub async fn spider_data_url<T: Serialize, U: DeserializeOwned>(
    url: &str,
    params: &T,
) -> Result<U, anyhow::Error> {
    let mut res = client.get(url).query(params).send().await?;

    while res.status().is_redirection() {
        let redirect_url =
            res.headers().get(LOCATION).unwrap().to_str().unwrap();
        res = client.get(redirect_url).send().await?;
    }

    let res = res.text().await?;

    let mut json_res: Value = serde_json::from_str(&res)?;

    if json_res.get("data").is_none_or(|v| v.is_null()) {
        return Err(anyhow::anyhow!("数据获取失败"));
    }

    let res: U = serde_json::from_value(json_res["data"].take())?; // take()方法将json_res的所有权转移给res

    Ok(res)
}
