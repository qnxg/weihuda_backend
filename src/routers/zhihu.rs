use crate::{
    error::AppError, routers::ThrowParseError,
    utils::serde::empty_string_as_none,
};
use salvo::{Request, Router, handler, macros::Extractible};
use serde::{Deserialize, Serialize};

use crate::{
    error::RouterResult,
    service::{self, zhihu::ZhihuListItem},
    utils,
};

pub fn routers() -> Router {
    Router::with_path("zhihu")
        .get(get_zhihu_page)
        .push(Router::with_path("tags").get(get_zhihu_tags))
        .push(Router::with_path("{id}").get(get_zhihu_by_id))
}

/// 获取知湖页列表
#[handler]
async fn get_zhihu_page(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct GetZhihuPageReq {
        pub offset: u32,
        pub req_count: u32,
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub title: Option<String>,
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub typ: Option<String>,
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub tags: Option<String>,
    }
    #[derive(Serialize, Deserialize, Debug)]
    struct GetZhihuPageRes {
        pub count: u32,
        pub rows: Vec<ZhihuListItem>,
    }
    let query: GetZhihuPageReq = req.extract().await.parse_error()?;
    if query.req_count > 100 {
        return Err(AppError::customized("count不能大于100"));
    }
    let stu_id = utils::jwt::auth(req)?;
    let (total, rows) = service::zhihu::get_zhihu_list(
        query.title,
        query.typ,
        query.tags,
        &stu_id,
        query.offset,
        query.req_count,
    )
    .await?;
    Ok(GetZhihuPageRes { count: total, rows }.into())
}

#[handler]
async fn get_zhihu_by_id(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "param")))]
    struct GetZhihuByIdReq {
        pub id: u32,
    }
    let GetZhihuByIdReq { id } = req.extract().await.parse_error()?;
    if let Some(zhihu) = service::zhihu::get_zhihu_by_id(id).await? {
        Ok(zhihu.into())
    } else {
        Err(AppError::customized("找不到该知湖文章"))
    }
}

#[handler]
async fn get_zhihu_tags() -> RouterResult {
    let tags = service::zhihu::get_zhihu_tags().await?;
    Ok(tags.into())
}
