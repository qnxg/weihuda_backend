use std::time::Duration;

use crate::{
    hdjw::login::hdjw_headers,
    utils::cache::{CACHE, CacheEnum},
};
use anyhow::anyhow;
use log::error;
use rand::Rng;
use reqwest::RequestBuilder;
use serde_json::Value;

/// 专门用于请求教务系统的函数。该函数可以自动进行错误处理和处理 cookie 失效
///
/// # Panics
///
/// - `req` 必须是可以 clone 的，否则会 panic
pub async fn request_hdjw(
    stu_id: &str,
    req: RequestBuilder,
) -> Result<Value, crate::Error> {
    if stu_id.starts_with('S') {
        // 研究生就先别凑热闹了
        return Err(anyhow!("暂不支持研究生教务系统").into());
    }
    let mut tried = 0;
    let mut err_log = String::new();
    let data;
    loop {
        if tried >= 2 {
            error!("请求教务系统失败多次，错误日志：{}", err_log);
            return Err(anyhow!("请求教务系统失败").into());
        }
        if tried > 0 {
            // 失败了就等一会儿再试
            let wait_time = rand::thread_rng().gen_range(200..500);
            tokio::time::sleep(Duration::from_millis(wait_time))
                .await;
        }
        let hdjw_headers = match hdjw_headers(stu_id).await {
            Ok(data) => data,
            // 账号异常直接返回，不重试了
            Err(crate::Error::PasswordError) => {
                return Err(crate::Error::PasswordError);
            }
            Err(crate::Error::PasswordShouldChange) => {
                return Err(crate::Error::PasswordShouldChange);
            }
            Err(crate::Error::PasswordLocked) => {
                return Err(crate::Error::PasswordLocked);
            }
            Err(e) => {
                tried += 1;
                err_log.push_str(&format!(
                    "({}) 获取教务系统请求头失败: err = {}; stuid = {}",
                    tried, e, stu_id
                ));
                continue;
            }
        };
        let resp = match req
            .try_clone()
            .expect("req must be cloneable")
            .headers(hdjw_headers)
            .send()
            .await
            .and_then(|resp| resp.error_for_status())
        {
            Ok(resp) => resp,
            Err(e) => {
                tried += 1;
                err_log.push_str(&format!(
                    "({}) 请求教务系统失败: err = {}; stuid = {}",
                    tried, e, stu_id
                ));
                continue;
            }
        };
        let body = match resp.text().await {
            Ok(body) => body,
            Err(e) => {
                tried += 1;
                err_log.push_str(&format!(
                    "({}) 读取教务系统响应失败: err = {}; stuid = {}",
                    tried, e, stu_id
                ));
                continue;
            }
        };
        if body.contains("window.initQzTable") {
            // 说明是课程分数详情的响应，我们特殊处理对待一下
            return Ok(Value::String(body));
        }
        let json = match serde_json::from_str::<Value>(&body) {
            Ok(json) => json,
            Err(e) => {
                tried += 1;
                err_log.push_str(&format!(
                    "({}) 解析教务系统响应失败: err = {}; body = {}; stuid = {}",
                    tried, e, body, stu_id
                ));
                // 这种情况（200 返回码但不是 json 格式（应该是 html 格式））大概是 cookie
                // 过期，我们清理缓存
                CACHE
                    .invalidate(&(CacheEnum::Hdjw, stu_id.into()))
                    .await;
                continue;
            }
        };
        // 典型的 cookie 失效时的 response body：
        // {"flag1":2,"msgContent":"è¯·å…ˆç™»å½•ç³»ç»Ÿ"}
        // 这里只判断 flag1 字段，因为 msgContent 是乱码，不好说
        if let Some(Value::Number(flag1)) = json.get("flag1")
            && flag1.as_i64() == Some(2)
        {
            CACHE.invalidate(&(CacheEnum::Hdjw, stu_id.into())).await;
            tried += 1;
            err_log.push_str(&format!(
                "({}) 教务系统 cookie 失效; stuid = {}",
                tried, stu_id
            ));
            continue;
        }
        data = json;
        break;
    }
    Ok(data)
}
