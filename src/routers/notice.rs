use crate::error::RouterResult;
use crate::routers::ThrowParseError;
use crate::service::notice::Notice;
use crate::utils::serde::empty_string_as_none;
use crate::{service, utils};
use salvo::Router;
use salvo::macros::Extractible;
use salvo::{Request, handler};
use serde::Deserialize;
use serde::Serialize;

pub fn routers() -> Router {
    Router::with_path("notice")
        .get(get_notice)
        .push(Router::with_path("{id}").put(put_notice_by_id))
}

#[handler]
async fn get_notice(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(
        default_source(from = "query"),
        rename_all = "camelCase"
    ))]
    struct GetNoticeReq {
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub page: Option<u32>,
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub page_size: Option<u32>,
    }
    #[derive(Serialize, Debug)]
    struct NoticeRes {
        pub count: u32,
        pub rows: Vec<Notice>,
    }
    let stu_id = utils::jwt::auth(req)?;
    let GetNoticeReq { page, page_size } =
        req.extract().await.parse_error()?;
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(10);
    let res =
        service::notice::get_notice_list(&stu_id, page, page_size)
            .await?;
    Ok(NoticeRes {
        count: res.len() as u32,
        rows: res,
    }
    .into())
}

#[handler]
async fn put_notice_by_id(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "body")))]
    struct PutNoticeByIdReq {
        #[salvo(extract(source(from = "param")))]
        pub id: u32,
        pub status: u32,
    }
    let PutNoticeByIdReq { id, status } =
        req.extract().await.parse_error()?;
    service::notice::update_notice(id, status).await?;
    Ok("更新通知状态成功".into())
}
