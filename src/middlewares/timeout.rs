use std::time::Duration;

use salvo::{
    Depot, FlowCtrl, Request, Response, handler,
    http::headers::{Connection, HeaderMapExt},
};

use crate::result::AppError;

#[handler]
pub async fn timeout_middleware(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    let timeout = match req.uri().path() {
        "/hdjw/grade-rank-from-ca" => Duration::from_secs(10),
        _ => Duration::from_secs(6),
    };
    tokio::select! {
        _ = ctrl.call_next(req, depot, res) => {},
        _ = tokio::time::sleep(timeout) => {
            res.headers_mut().typed_insert(Connection::close());
            res.render(AppError::TimeoutError);
            ctrl.skip_rest();
        }
    }
}
