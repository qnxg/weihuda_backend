use salvo::{Request, Router, handler};
use serde_json::Value;

use crate::{error::RouterResult, service, utils};

pub fn routers() -> Router {
    Router::with_path("pt/email").get(get_campus_email_unread_count)
}

#[handler]
async fn get_campus_email_unread_count(
    req: &mut Request,
) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    let res = service::email::get_unread_email_count(&stu_id).await;
    match res {
        Err(_) => Ok(Value::Null.into()),
        Ok(res) => match res {
            None => Ok(Value::Null.into()),
            Some(count) => Ok(count.into()),
        },
    }
}
