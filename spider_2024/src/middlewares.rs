use std::{
    panic::AssertUnwindSafe, sync::atomic::AtomicU64, time::Duration,
};

use crate::{
    app_error::AppError,
    utils::cache::{CACHE, CacheEnum},
};
use anyhow::anyhow;
use futures::FutureExt;
use log::{error, info};
use salvo::{
    Depot, FlowCtrl, Request, Response, Writer, handler,
    http::{
        ResBody,
        headers::{Connection, HeaderMapExt},
    },
    prelude::StatusCode,
    writing::Json,
};
use tokio::time::Instant;

/// 提供授权的中间件
#[handler]
pub async fn auth_middleware(req: &mut Request, res: &mut Response) {
    let auth_header: Option<String> = req.header("Authorization");
    if let Some(auth_header) = auth_header {
        if auth_header
            != "OUJhbGciOiJIUzU(x7)iIsImlhdCI6MTYxNzQy$jAwMiwiZXh#IjoxNjUzNDI2MDAyfQ@eyI6ImFkbWhjXzxcwEiT7dlm9sFeSRlgY7rnJKpBA"
        {
            res.stuff(
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "code": 401,
                    "data": serde_json::Value::Null,
                    "msg": "授权失败",
                })),
            );
        }
    } else {
        res.stuff(
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "code": 401,
                "data": serde_json::Value::Null,
                "msg": "未经授权",
            })),
        );
    }
}

/// 提供根据code记录日志的中间件
#[handler]
pub async fn log_middleware(
    &self,
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    info!("Request {} {}", req.method(), req.uri(),);
    let start = Instant::now();
    ctrl.call_next(req, depot, res).await;
    let status = res.status_code.unwrap_or(match &res.body {
        ResBody::None => StatusCode::NOT_FOUND,
        ResBody::Error(e) => e.code,
        _ => StatusCode::OK,
    });
    let end = Instant::now();
    let duration = (end - start).as_millis();
    if status == StatusCode::OK {
        info!("Response {duration}ms {status} {}", req.uri(),);
    } else {
        let msg = match &res.body {
            ResBody::Once(body) => {
                Some(String::from_utf8_lossy(body))
            }
            _ => None,
        };
        error!(
            "Response {duration}ms {status} {} {}",
            req.uri(),
            msg.unwrap_or("没有报错信息".into()),
        );
    }
}

/// 负责提前删除失败的token（连续5次请求失败）（只针对bks教务系统的请求）
/// 压测后认为，这个值设置为5及以上时一定不会是性能瓶颈
#[handler]
pub async fn hdjw_reset_middleware(
    req: &mut Request,
    depot: &mut Depot,
    ctrl: &mut FlowCtrl,
    res: &mut Response,
) {
    const FAILURE_LIMIT: u64 = 5;
    let stu_id: Option<String> = req.query("stuid");
    if stu_id.is_none() {
        ctrl.call_next(req, depot, res).await;
        return;
    }
    let stu_id = stu_id.unwrap();
    ctrl.call_next(req, depot, res).await;

    if res.status_code.unwrap_or(StatusCode::OK).is_server_error() {
        CACHE
            .entry((CacheEnum::HdjwFailureRecord, stu_id.to_owned()))
            .and_upsert_with(|entry| async {
                match entry {
                    None => "1".into(),
                    Some(entry) => {
                        let Ok(count) = entry.value().parse::<u64>()
                        else {
                            error!("异常的hdjw错误计数");
                            return "0".into();
                        };
                        if count >= FAILURE_LIMIT {
                            CACHE
                                .invalidate(&(
                                    CacheEnum::Hdjw,
                                    stu_id,
                                ))
                                .await;
                            return "0".into();
                        }
                        (count + 1).to_string()
                    }
                }
            })
            .await;
    } else {
        CACHE
            .invalidate(&(CacheEnum::HdjwFailureRecord, stu_id))
            .await;
    }
}

/// handle panics
#[handler]
pub async fn catch_panic(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    if let Err(e) = AssertUnwindSafe(ctrl.call_next(req, depot, res))
        .catch_unwind()
        .await
    {
        res.render(AppError::AnyHow(anyhow!(
            "Panic occurred on server: {e:#?}"
        )));
    }
}

/// handler timeout
#[handler]
pub async fn timeout(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    let timeout_secs = match req.uri().path() {
        "/bks/grade-from-ca" => 60,
        _ => 6,
    };
    tokio::select! {
        _ = ctrl.call_next(req, depot, res) => {},
        _ = tokio::time::sleep(Duration::from_secs(timeout_secs)) => {
            res.headers_mut().typed_insert(Connection::close());
            res.render(AppError::Timeout);
            ctrl.skip_rest();
        }
    }
}

/// 限流中间件
/// 压测后认为，限流会降低爬虫处理请求的成功率和单位时间的成功数。
#[handler]
pub async fn throttle(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    const RUNNING_HANDLER_LIMIT: u64 = 500;
    static RUNNING_HANDLER_CNT: AtomicU64 = AtomicU64::new(0);
    if RUNNING_HANDLER_CNT.load(std::sync::atomic::Ordering::Acquire)
        > RUNNING_HANDLER_LIMIT
    {
        res.headers_mut().typed_insert(Connection::close());
        res.render(AppError::OtherErr(
            StatusCode::SERVICE_UNAVAILABLE,
            "服务器高负载".into(),
        ));
        ctrl.skip_rest();
    }
    RUNNING_HANDLER_CNT
        .fetch_add(1, std::sync::atomic::Ordering::Release);
    struct A;
    let a = A;
    impl Drop for A {
        fn drop(&mut self) {
            RUNNING_HANDLER_CNT
                .fetch_sub(1, std::sync::atomic::Ordering::Release);
        }
    }
    ctrl.call_next(req, depot, res).await;
    let _b = &a;
}

/// 处理HTTP空响应体，例如404状态或者405状态或者路由出了什么问题的的情况
#[handler]
pub async fn default_response(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    ctrl.call_next(req, depot, res).await;
    let body_size = res.body.size().unwrap_or(0);
    if body_size > 0 {
        return;
    }

    match res.status_code {
        None => {
            AppError::AnyHow(anyhow!("服务器错误：未返回有效信息"))
        }
        Some(status_code) => AppError::OtherErr(
            status_code,
            status_code
                .canonical_reason()
                .unwrap_or(&format!(
                    "空结果与未知返回状态: {}",
                    status_code.as_str()
                ))
                .into(),
        ),
    }
    .write(req, depot, res)
    .await;
}
