use crate::result::AppError;
use crate::result::RouterResult;
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

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
struct GetNoticeReq {
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub page: Option<u32>,
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub pageSize: Option<u32>,
}
#[derive(Serialize, Debug)]
struct NoticeRes {
    pub count: u32,
    pub rows: Vec<Notice>,
}
#[handler]
async fn get_notice(req: &mut Request) -> RouterResult {
    let (_, stu_id) = utils::jwt::auth(req)?;
    let GetNoticeReq { page, pageSize, .. } = req.parse_queries()?;
    let page = page.unwrap_or(1);
    let page_size = pageSize.unwrap_or(10);
    let res =
        service::notice::get_notice_list(&stu_id, page, page_size)
            .await?;
    Ok(NoticeRes {
        count: res.len() as u32,
        rows: res,
    }
    .into())
}

#[derive(Deserialize, Debug, Extractible)]
#[salvo(extract(default_source(from = "body")))]
struct PutNoticeByIdReq {
    #[salvo(extract(source(from = "param")))]
    pub id: u32,
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub result: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub status: Option<i32>,
}
#[handler]
async fn put_notice_by_id(req: &mut Request) -> RouterResult {
    let PutNoticeByIdReq { id, result, status } =
        req.extract().await?;
    if let Some(status) = status
        && status != 0
        && status != 1
    {
        return Err(AppError::ParseError());
    }
    service::notice::update_notice(id, result.as_ref(), status)
        .await?;
    Ok("更新通知状态成功".into())
}
