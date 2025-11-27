use crate::utils::serde::empty_string_as_none;
use salvo::{Request, Router, handler};
use serde::{Deserialize, Serialize};

use crate::{
    result::RouterResult,
    service::{self, config::Config},
};

pub fn routers() -> Router {
    Router::with_path("config").get(get_config)
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
struct GetConfigReq {
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub page: Option<u32>,
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub pageSize: Option<u32>,
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub key: Option<String>,
}
#[derive(Serialize, Debug)]
struct GetConfigRes {
    pub count: usize,
    pub rows: Vec<Config>,
}
#[handler]
async fn get_config(req: &mut Request) -> RouterResult {
    let GetConfigReq {
        page,
        pageSize,
        key,
    } = req.parse_queries()?;
    let page = page.unwrap_or(1);
    let page_size = pageSize.unwrap_or(10);
    let like = format!("%{}%", key.unwrap_or_default());
    let offset = (page - 1) * page_size;

    let res =
        service::config::get_config(&like, offset, page_size).await?;

    Ok(GetConfigRes {
        count: res.len(),
        rows: res,
    }
    .into())
}
