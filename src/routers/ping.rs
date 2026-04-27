use salvo::{Router, handler};

use crate::result::RouterResult;

static MESSAGE: &str = "I am fine!";

pub fn routers() -> Router {
    Router::with_path("ping").get(health_checker)
}

#[handler]
async fn health_checker() -> RouterResult {
    Ok(MESSAGE.into())
}
