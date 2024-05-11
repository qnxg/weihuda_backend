use once_cell::sync::Lazy;
use reqwest::{
    header::{HeaderMap, AUTHORIZATION, LOCATION},
    redirect::Policy,
    Client,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

use crate::config::CFG;

#[allow(non_upper_case_globals)]
pub static client: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .connection_verbose(false)
        // .timeout(Duration::from_secs(6)) // timeout直接使用后端中间件的超时时间，不再单独设置
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
) -> Result<U, anyhow::Error> {
    let url = format!("{}{}", CFG.service.spider_url, path);

    let mut res = client.get(url).query(params).send().await?;

    while res.status().is_redirection() {
        let redirect_url = res.headers().get(LOCATION).unwrap().to_str().unwrap();
        res = client.get(redirect_url).send().await?;
    }

    let res = res.text().await?;

    let mut json_res: Value = serde_json::from_str(&res)?;

    if json_res.get("data").map_or(true, |v| v.is_null()) {
        return Err(anyhow::anyhow!("请检查个人门户密码或5分钟后再次尝试"));
    }

    let res: U = serde_json::from_value(json_res["data"].take())?; // take()方法将json_res的所有权转移给res

    Ok(res)
}

/// 直接返回爬虫返回的json数据
#[inline]
pub async fn spider<T: Serialize, U: DeserializeOwned>(
    path: &str,
    params: &T,
) -> Result<U, anyhow::Error> {
    let url = format!("{}{}", CFG.service.spider_url, path);

    let mut res = client.get(url).query(params).send().await?;

    while res.status().is_redirection() {
        let redirect_url = res.headers().get(LOCATION).unwrap().to_str().unwrap();
        res = client.get(redirect_url).send().await?;
    }

    let res = res.text().await?;

    let json_res: Value = serde_json::from_str(&res)?;

    let res: U = serde_json::from_value(json_res)
        .map_err(|_| anyhow::anyhow!("请检查个人门户密码或5分钟后再次尝试"))?;

    Ok(res)
}

#[allow(dead_code)]
///访问地址完全自定义
#[inline]
pub async fn spider_data_url<T: Serialize, U: DeserializeOwned>(
    url: &str,
    params: &T,
) -> Result<U, anyhow::Error> {
    let mut res = client.get(url).query(params).send().await?;

    while res.status().is_redirection() {
        let redirect_url = res.headers().get(LOCATION).unwrap().to_str().unwrap();
        res = client.get(redirect_url).send().await?;
    }

    let res = res.text().await?;

    let mut json_res: Value = serde_json::from_str(&res)?;

    if json_res.get("data").map_or(true, |v| v.is_null()) {
        return Err(anyhow::anyhow!("请检查个人门户密码或5分钟后再次尝试"));
    }

    let res: U = serde_json::from_value(json_res["data"].take())?; // take()方法将json_res的所有权转移给res

    Ok(res)
}
