use chrono::NaiveDateTime;
use salvo::Router;
use salvo::{Request, handler};

use crate::result::RouterResult;
use crate::utils::serde::deserialize_naive_datetime;
use crate::{service, utils};
use serde::Deserialize;

pub fn routers() -> Router {
    Router::with_path("message-left").post(post_left_message)
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PostLeftMessageReq {
    pub desc: String,
    pub is_agree: i64,
    pub is_send: i64,
    #[serde(deserialize_with = "deserialize_naive_datetime")]
    pub send_time: NaiveDateTime,
}
#[handler]
async fn post_left_message(req: &mut Request) -> RouterResult {
    let (_, stu_id) = utils::jwt::auth(req)?;
    let PostLeftMessageReq {
        desc,
        is_agree,
        is_send,
        send_time,
        ..
    } = req.parse_json().await?;
    let is_agree = is_agree != 0;
    let is_send = is_send != 0;
    service::left_message::add_left_message(
        &stu_id, &desc, is_agree, send_time, is_send,
    )
    .await?;
    Ok("留言成功".into())
}
