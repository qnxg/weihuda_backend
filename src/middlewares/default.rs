use crate::utils;
use salvo::{
    Depot, FlowCtrl, Request, Response, handler, writing::Json,
};
use serde_json::json;

/// 中间件，处理任何无返回体的结果
///
/// 当请求没有进入 router 层时，响应体的内容为空
#[handler]
pub async fn default_middleware(
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
        // 这种情况下，status code 不可能为 None
        None => panic!("status code should not be none"),
        Some(status_code) => {
            let status_message =
                status_code.canonical_reason().unwrap_or("未知错误");
            res.stuff(
                status_code,
                Json(json!({
                    "code": status_code.as_u16(),
                    "data": null,
                    "msg": status_message,
                })),
            );
            utils::record!(
                otel.status_code = "error",
                otel.status_description = %status_message,
            );
        }
    }
}
