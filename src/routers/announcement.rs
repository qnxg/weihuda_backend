use salvo::{Router, handler};

use crate::{result::RouterResult, service};

pub fn routers() -> Router {
    Router::with_path("announcement").get(get_announcement_list) // 获取小程序公告
}

#[handler]
async fn get_announcement_list() -> RouterResult {
    let announcement =
        service::announcement::get_announcement_list(10).await?;
    Ok(announcement.into())
}
