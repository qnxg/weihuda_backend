use salvo::{Request, Router, handler};
use serde_json::Value;

use crate::{result::RouterResult, service, utils};

pub fn routers() -> Router {
    Router::with_path("user-settings/all")
        .get(get_all_user_settings)
        .post(post_all_user_settings)
}

#[handler]
async fn get_all_user_settings(req: &mut Request) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    let res = service::user_info::get_user_setting(&stu_id).await?;
    Ok(res.into())
}

#[handler]
async fn post_all_user_settings(req: &mut Request) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    let settings: Value = req.parse_json().await?;
    service::user_info::update_user_setting(&stu_id, &settings)
        .await?;
    Ok("设置提交成功".into())
}
