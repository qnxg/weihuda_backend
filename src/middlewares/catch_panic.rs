use crate::result::AppError;
use futures::FutureExt;
use salvo::{Depot, FlowCtrl, Request, Response, handler};
use std::panic::{AssertUnwindSafe, PanicHookInfo};

#[handler]
pub async fn catch_panic_middleware(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    if AssertUnwindSafe(ctrl.call_next(req, depot, res))
        .catch_unwind()
        .await
        .is_err()
    {
        // 该中间件主要是用来 panic 时返回给用户恰当的错误信息。输出相关日志在 [panic_hook] 中进行。
        res.render(AppError::Text("服务器内部错误".to_string()));
    }
}

pub fn panic_hook(info: &PanicHookInfo) {
    let msg = info
        .payload()
        .downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| info.payload().downcast_ref::<String>().cloned())
        .unwrap_or(format!(
            "Unknown panic, type_id: {:?}",
            info.payload().type_id()
        ));
    let location = info
        .location()
        .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()));
    tracing::error!(%msg, ?location, "thread panicked");
}
