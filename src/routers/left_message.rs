use chrono::NaiveDateTime;
use salvo::Router;
use salvo::{Request, handler, macros::Extractible};
use serde::Deserialize;

use crate::result::RouterResult;
use crate::utils::serde::deserialize_naive_datetime;
use crate::utils::serde::empty_string_as_none;
use crate::{service, utils};

pub fn routers() -> Router {
    Router::with_path("message-left").post(post_left_message)
}

#[handler]
async fn post_left_message(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(
        default_source(from = "body"),
        rename_all = "camelCase"
    ))]
    struct PostLeftMessageReq {
        pub desc: String,
        pub is_agree: i64,
        pub is_send: i64,
        #[serde(deserialize_with = "deserialize_naive_datetime")]
        pub send_time: NaiveDateTime,
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub email: Option<String>,
    }
    let body: PostLeftMessageReq = req.extract().await?;
    let stu_id = utils::jwt::auth(req)?;
    let is_agree = body.is_agree != 0;
    let is_send = body.is_send != 0;
    service::left_message::add_left_message(
        &stu_id,
        &body.desc,
        body.email.as_deref(),
        is_agree,
        body.send_time,
        is_send,
    )
    .await?;
    Ok("留言成功".into())
}
