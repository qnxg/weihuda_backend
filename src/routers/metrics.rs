use salvo::{Request, Response, Router, handler};

use crate::middlewares::prometheus::render_metrics;

pub fn routers() -> Router {
    Router::with_path("metrics").get(metrics_handler)
}

#[handler]
async fn metrics_handler(_req: &mut Request, res: &mut Response) {
    render_metrics(res).await;
}
