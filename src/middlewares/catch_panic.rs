use crate::error::{AppError, ThrowInternalErrorMsg};
use crate::utils;
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
    if AssertUnwindSafe(ctrl.call_next(req, depot, res))
        .catch_unwind()
        .await
        .is_err()
    {
        // panic 的内容和产生位置由 panic_hook 记录，放到出现 panic 的 span 的 span event 中
        utils::record!(panic = true, otel.status_code = "error");
        res.render(Into::<AppError>::into(
            "thread panicked".internal_err(),
        ));
    }
}
