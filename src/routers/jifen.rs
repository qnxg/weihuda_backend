use crate::result::AppError;
use crate::service::jifen::{JifenGoods, JifenRecord, JifenRule};
use crate::utils;
use crate::utils::serde::empty_string_as_none;
use crate::{result::RouterResult, service};
use anyhow::anyhow;
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
        .ok_or(AppError::Unauthorized)?;
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
    let query: GetJifenRecordReq = req.extract().await?;
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
    } = req.extract().await?;
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
    let query: GetJifenRulesReq = req.extract().await?;
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
        pub param: String,
    }
    let PostRecordReq { key, param } = req.extract().await?;
    let stu_id = utils::jwt::auth(req)?;
    let jifen_rule = service::jifen::get_jifen_rule(&key)
        .await?
        .ok_or(anyhow!("没有积分规则：{}", key))?;
    // 查询是否重复添加
    if service::jifen::get_jifen_record(&stu_id, &key, &param)
        .await?
        .is_some()
    {
        return Err("已经添加过积分记录".into());
    }
    // 查询周期内的积分记录
    let now = utils::time::now_time();
    let create_time_greater_than =
        now - chrono::Duration::days(jifen_rule.cycle as i64 - 1);
    let count = service::jifen::get_jifen_record_count(
        &stu_id,
        &key,
        create_time_greater_than,
    )
    .await?;
    if count >= jifen_rule.max_count {
        return Err("超过周期内最大次数".into());
    }
    // 添加积分
    let new_jifen = service::jifen::add_jifen(
        &stu_id,
        &key,
        &param,
        &jifen_rule.name,
        jifen_rule.jifen,
    )
    .await?;
    Ok(new_jifen.into())
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
        pub goods_id: i32,
    }
    let ExchangeGoodsReq { goods_id } = req.extract().await?;
    let stu_id = utils::jwt::auth(req)?;
    let goods = service::jifen::get_goods(goods_id as u32)
        .await?
        .ok_or(anyhow!("没有找到商品：{}", goods_id))?;
    // 检查商品库存
    if !goods.enabled {
        return Err("商品已下架".into());
    }
    if goods.count == 0 {
        return Err("商品库存不足".into());
    }
    let user_jifen = service::jifen::get_jifen(&stu_id)
        .await?
        .ok_or(AppError::Unauthorized)?;
    if user_jifen < goods.price {
        return Err("积分不足".into());
    }
    // 扣除积分并添加记录
    service::jifen::exchange_goods(&stu_id, goods).await?;
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
        req.extract().await?;
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
    let GetWebviewReq { url } = req.extract().await?;
    let stu_id = utils::jwt::auth(req)?;
    let key = String::from("yuedu");
    let param = url;
    let jifen_rule = service::jifen::get_jifen_rule(&key)
        .await?
        .ok_or(anyhow!("没有积分规则：{}", key))?;
    // 查询是否重复添加
    if service::jifen::get_jifen_record(&stu_id, &key, &param)
        .await?
        .is_some()
    {
        return Err("已经添加过积分记录".into());
    }
    // 查询周期内的积分记录
    let now = utils::time::now_time();
    let create_time_greater_than =
        now - chrono::Duration::days(jifen_rule.cycle as i64 - 1);
    let count = service::jifen::get_jifen_record_count(
        &stu_id,
        &key,
        create_time_greater_than,
    )
    .await?;
    if count >= jifen_rule.max_count {
        return Err("超过周期内最大次数".into());
    }
    // 添加积分
    let new_jifen = service::jifen::add_jifen(
        &stu_id,
        &key,
        &param,
        &jifen_rule.name,
        jifen_rule.jifen,
    )
    .await?;
    Ok(new_jifen.into())
}

#[handler]
async fn get_jifen_desc() -> RouterResult {
    let desc = service::jifen::get_jifen_desc().await?;
    Ok(desc.into())
}
