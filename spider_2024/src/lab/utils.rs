use std::time::Duration;

use anyhow::anyhow;
use log::error;
use rand::Rng;
use reqwest::RequestBuilder;
use serde_json::Value;

use crate::{
    lab::login::lab_headers,
    utils::cache::{CACHE, CacheEnum},
};

/// 专门用于请求实验平台的函数。该函数可以自动进行错误处理和处理 cookie 失效
///
/// # Panics
///
/// - `req` 必须是可以 clone 的，否则会 panic
pub async fn request_lab(
    stu_id: &str,
    req: RequestBuilder,
) -> Result<Value, crate::Error> {
    let mut tried = 0;
    let mut err_log = String::new();
    let data;
    loop {
        if tried >= 2 {
            error!("请求实验平台失败多次，错误日志：{}", err_log);
            return Err(anyhow!("请求实验平台失败").into());
        }
        if tried > 0 {
            // 失败了就等一会儿再试
            let wait_time = rand::thread_rng().gen_range(200..500);
            tokio::time::sleep(Duration::from_millis(wait_time))
                .await;
        }
        let lab_headers = match lab_headers(stu_id).await {
            Ok(data) => data,
            Err(crate::Error::PasswordError) => {
                return Err(crate::Error::PasswordError);
            }
            Err(e) => {
                tried += 1;
                err_log.push_str(&format!(
                    "({}) 获取实验平台请求头失败: err = {}; stuid = {}",
                    tried, e, stu_id
                ));
                continue;
            }
        };
        let body = match req
            .try_clone()
            .expect("req must be cloneable")
            .headers(lab_headers)
            .send()
            .await
            .and_then(|resp| resp.error_for_status())
        {
            Ok(resp) => match resp.text().await {
                Ok(body) => body,
                Err(e) => {
                    tried += 1;
                    err_log.push_str(&format!(
                        "({}) 读取实验平台响应失败: err = {}; stuid = {}",
                        tried, e, stu_id
                    ));
                    continue;
                }
            },
            Err(e) => {
                tried += 1;
                err_log.push_str(&format!(
                    "({}) 请求实验平台失败: err = {}; stuid = {}",
                    tried, e, stu_id
                ));
                continue;
            }
        };
        match serde_json::from_str::<Value>(&body) {
            Err(e) => {
                tried += 1;
                err_log.push_str(&format!(
                    "({}) 解析实验平台响应失败: err = {}; body = {}; stuid = {}",
                    tried, e, body, stu_id
                ));
                // 这种情况（200 返回码但不是 json 格式（应该是 html 格式））大概是 cookie
                // 过期，我们清理缓存
                CACHE
                    .invalidate(&(
                        CacheEnum::LabCookie,
                        stu_id.into(),
                    ))
                    .await;
                continue;
            }
            Ok(json) => {
                data = json;
                break;
            }
        }
    }
    Ok(data)
}
