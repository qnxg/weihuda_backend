use crate::result::AppError;
use anyhow::anyhow;
use futures::FutureExt;
use salvo::{Depot, FlowCtrl, Request, Response, handler};
use std::panic::AssertUnwindSafe;

#[handler]
pub async fn catch_panic_middleware(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    if let Err(e) = AssertUnwindSafe(ctrl.call_next(req, depot, res))
        .catch_unwind()
        .await
    {
        tracing::error!(panic = ?e, "Panic occurred on server");
        res.render(AppError::AnyHow(anyhow!(
            "服务器发生意料之外异常"
        )));
    }
}
