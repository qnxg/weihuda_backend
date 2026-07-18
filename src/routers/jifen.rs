use crate::error::AppError;
use crate::routers::ThrowParseError;
use crate::service::jifen::{JifenGoods, JifenRecord, JifenRule};
use crate::utils;
use crate::utils::serde::empty_string_as_none;
use crate::{error::RouterResult, service};
use salvo::macros::Extractible;
use salvo::{Request, Router, handler};
use serde::{Deserialize, Serialize};

pub fn routers() -> Router {
    Router::with_path("jifen")
        .push(Router::with_path("total").get(get_jifen))
        .push(Router::with_path("record").get(get_jifen_record))
        .push(Router::with_path("goods").get(get_jifen_goods))
        .push(Router::with_path("rules").get(get_jifen_rules))
        .push(Router::with_path("desc").get(get_jifen_desc))
        .push(
            Router::with_path("exchange-record")
                .get(get_exchange_record_list)
                .post(exchange_goods),
        )
        .push(Router::with_path("webview-read").get(get_webview_read))
        .post(post_record)
}

#[handler]
async fn get_jifen(req: &mut Request) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    let res = service::jifen::get_jifen(&stu_id)
        .await?
        .ok_or_else(AppError::unauthorized)?;
    Ok(res.into())
}

#[handler]
async fn get_jifen_record(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(
        default_source(from = "query"),
        rename_all = "camelCase"
    ))]
    struct GetJifenRecordReq {
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub page: Option<u32>,
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub page_size: Option<u32>,
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub key: Option<String>,
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub param: Option<String>,
    }
    #[derive(Serialize, Debug)]
    struct GetJifenRecordRes {
        pub count: u32,
        pub rows: Vec<JifenRecord>,
    }
    let query: GetJifenRecordReq =
        req.extract().await.parse_error()?;
    let stu_id = utils::jwt::auth(req)?;
    let res = service::jifen::get_jifen_record_list(
        &stu_id,
        query.page.unwrap_or(1),
        query.page_size.unwrap_or(20),
        query.key,
        query.param,
    )
    .await?;
    Ok(GetJifenRecordRes {
        count: res.len() as u32,
        rows: res,
    }
    .into())
}

#[handler]
async fn get_jifen_goods(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(
        default_source(from = "query"),
        rename_all = "camelCase"
    ))]
    struct GetJifenGoodsReq {
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub page: Option<u32>,
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub page_size: Option<u32>,
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub name: Option<String>,
    }
    #[derive(Serialize, Debug)]
    struct GetJifenGoodsRes {
        pub count: u32,
        pub rows: Vec<JifenGoods>,
    }
    let GetJifenGoodsReq {
        page,
        page_size,
        name,
    } = req.extract().await.parse_error()?;
    let res = service::jifen::get_goods_list(
        name,
        page.unwrap_or(1),
        page_size.unwrap_or(10),
        true,
    )
    .await?;
    Ok(GetJifenGoodsRes {
        count: res.len() as u32,
        rows: res,
    }
    .into())
}

#[handler]
async fn get_jifen_rules(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(
        default_source(from = "query"),
        rename_all = "camelCase"
    ))]
    struct GetJifenRulesReq {
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub page: Option<u32>,
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub page_size: Option<u32>,
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub key: Option<String>,
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub name: Option<String>,
    }
    #[derive(Serialize, Debug)]
    struct GetJifenRulesRes {
        pub count: u32,
        pub rows: Vec<JifenRule>,
    }
    let query: GetJifenRulesReq =
        req.extract().await.parse_error()?;
    let res = service::jifen::get_jifen_rule_list(
        query.key,
        query.name,
        query.page.unwrap_or(1),
        query.page_size.unwrap_or(10),
        true,
    )
    .await?;
    Ok(GetJifenRulesRes {
        count: res.len() as u32,
        rows: res,
    }
    .into())
}

/// 添加积分
#[handler]
async fn post_record(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "body")))]
    struct PostRecordReq {
        pub key: String,
        #[expect(unused)]
        pub param: String,
    }
    let PostRecordReq { key, .. } =
        req.extract().await.parse_error()?;
    let stu_id = utils::jwt::auth(req)?;
    // 这里这么做主要是兼容当前前端
    let res = match key.as_str() {
        "qiandao" => service::jifen::sign_in(&stu_id).await?,
        _ => {
            return Err(AppError::customized("不支持的积分类型"));
        }
    };
    Ok(res.into())
}

// 兑换商品
#[handler]
async fn exchange_goods(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(
        default_source(from = "body"),
        rename_all = "camelCase"
    ))]
    struct ExchangeGoodsReq {
        pub goods_id: u32,
    }
    let ExchangeGoodsReq { goods_id } =
        req.extract().await.parse_error()?;
    let stu_id = utils::jwt::auth(req)?;
    service::jifen::exchange_goods(&stu_id, goods_id).await?;
    Ok("兑换成功".into())
}

#[handler]
async fn get_exchange_record_list(req: &mut Request) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(
        default_source(from = "query"),
        rename_all = "camelCase"
    ))]
    struct GetExchangeRecordListReq {
        pub page: Option<u32>,
        pub page_size: Option<u32>,
    }
    let GetExchangeRecordListReq { page, page_size } =
        req.extract().await.parse_error()?;
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(10);
    let res = service::jifen::get_exchange_record_list(
        &stu_id, page, page_size,
    )
    .await?;
    Ok(res.into())
}

#[handler]
async fn get_webview_read(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct GetWebviewReq {
        pub url: String,
    }
    let GetWebviewReq { url } = req.extract().await.parse_error()?;
    let stu_id = utils::jwt::auth(req)?;
    let res = service::jifen::read_zhihu(&stu_id, &url).await?;
    Ok(res.into())
}

#[handler]
async fn get_jifen_desc() -> RouterResult {
    let desc = service::jifen::get_jifen_desc().await?;
    Ok(desc.into())
}
