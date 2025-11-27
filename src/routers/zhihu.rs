use crate::utils::serde::empty_string_as_none;
use salvo::{Request, Router, handler, macros::Extractible};
use serde::{Deserialize, Serialize};

use crate::{
    result::{AppResult, RouterResult},
    service::{self, zhihu::ZhihuListItem},
    utils,
};

pub fn routers() -> Router {
    Router::with_path("zhihu")
        .get(get_zhihu_page)
        .post(post_zhihu)
        .push(
            Router::with_path("{id}")
                .get(get_zhihu_by_id)
                .put(put_zhihu)
                .delete(delete_zhihu),
        )
}

/// 获取知湖页列表
#[derive(Deserialize, Debug)]
struct GetZhihuPageReq {
    pub offset: u32,
    pub req_count: u32,
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub title: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    #[serde(rename = "type")]
    pub _type: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub tags: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
struct GetZhihuPageRes {
    pub count: u32,
    pub rows: Vec<ZhihuListItem>,
}
#[handler]
async fn get_zhihu_page(req: &mut Request) -> RouterResult {
    let GetZhihuPageReq {
        offset,
        req_count,
        title,
        _type,
        tags,
    } = req.parse_queries()?;
    if req_count > 100 {
        return Err("count不能大于100".into());
    }
    let (_, stu_id) = utils::jwt::auth(req)?;
    let (total, rows) = service::zhihu::get_zhihu_list(
        title, _type, tags, &stu_id, offset, req_count,
    )
    .await?;
    Ok(GetZhihuPageRes { count: total, rows }.into())
}

#[derive(Deserialize, Debug)]
struct GetZhihuByIdReq {
    pub id: u32,
}
#[handler]
async fn get_zhihu_by_id(req: &mut Request) -> RouterResult {
    let GetZhihuByIdReq { id } = req.parse_params()?;
    if let Some(zhihu) = service::zhihu::get_zhihu_by_id(id).await? {
        Ok(zhihu.into())
    } else {
        Err("找不到该知湖文章".into())
    }
}

fn check_zhihu_item(item: ZhihuListItem) -> AppResult<ZhihuListItem> {
    if let Some(_type) = item._type.clone() {
        // data必须是article和link之一
        if !["article", "link"].contains(&_type.as_str()) {
            return Err("类型必须为'article', 'link'中的一个".into());
        }
    } else {
        return Err("type不能为空".into());
    }

    if item.content.is_none() {
        return Err("content不能为空".into());
    }

    if let Some(status) = item.status {
        if ![0, 1].contains(&status) {
            return Err("status必须为0或1".into());
        }
    } else {
        return Err("status不能为空".into());
    }

    Ok(item)
}

#[handler]
async fn post_zhihu(req: &mut Request) -> RouterResult {
    let zhihu: ZhihuListItem = req.parse_json().await?;
    let zhihu = check_zhihu_item(zhihu)?;
    let new_id = service::zhihu::add_zhihu(zhihu).await?;
    Ok(new_id.into())
}

#[derive(Deserialize, Debug, Extractible)]
struct PutZhihuReq {
    #[salvo(extract(source(from = "param")))]
    pub id: u32,
    #[salvo(extract(flatten))]
    pub item: ZhihuListItem,
}
#[handler]
async fn put_zhihu(req: &mut Request) -> RouterResult {
    let PutZhihuReq { id, item } = req.extract().await?;
    let zhihu = check_zhihu_item(item)?;
    service::zhihu::update_zhihu(id, zhihu).await?;
    Ok(().into())
}

#[derive(Deserialize, Debug)]
struct DeleteZhihuReq {
    pub id: u32,
}
#[handler]
async fn delete_zhihu(req: &mut Request) -> RouterResult {
    let DeleteZhihuReq { id } = req.parse_params()?;
    service::zhihu::delete_zhihu(id).await?;
    Ok(().into())
}
