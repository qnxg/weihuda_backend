use crate::service::jifen::{JifenGoods, JifenRecord, JifenRule};
use crate::utils;
use crate::utils::serde::empty_string_as_none;
use crate::{result::RouterResult, service};
use anyhow::anyhow;
use salvo::{Request, Router, handler};
use serde::{Deserialize, Serialize};
use serde_json::json;

pub fn routers() -> Router {
    Router::with_path("jifen")
        .push(Router::with_path("total").get(get_jifen))
        .push(Router::with_path("record").get(get_jifen_record))
        .push(Router::with_path("goods").get(get_jifen_goods))
        .push(Router::with_path("rules").get(get_jifen_rules))
        .push(
            Router::with_path("goods-record")
                .get(get_jifen_record)
                .post(post_goods),
        )
        .push(Router::with_path("webview-read").get(get_webview_read))
        .post(post_record)
}

#[handler]
async fn get_jifen(req: &mut Request) -> RouterResult {
    let (_, stu_id) = utils::jwt::auth(req)?;
    // 这里确保用户是存在的了
    let res = service::jifen::get_jifen(&stu_id)
        .await?
        .expect("通过鉴权，但是获取积分时用户不存在");
    // 为了与原接口兼容，这里还需要获取当前用户的 id，虽然这貌似并没有什么必要
    let mini_bind = service::auth::user::check_by_stu_id(&stu_id)
        .await?
        .expect("通过鉴权，但是获取 mini_bind 时用户不存在");
    // 兼容旧接口
    Ok(json!({
        "jifen": res,
        "stuID": &stu_id,
        "id": mini_bind.id,
    })
    .into())
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
struct GetJifenRecordReq {
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub page: Option<u32>,
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub pageSize: Option<u32>,
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
#[handler]
async fn get_jifen_record(req: &mut Request) -> RouterResult {
    let GetJifenRecordReq {
        page,
        pageSize,
        key,
        param,
    } = req.parse_queries()?;
    let (_, stu_id) = utils::jwt::auth(req)?;
    let res = service::jifen::get_jifen_record_list(
        &stu_id,
        page.unwrap_or(1),
        pageSize.unwrap_or(20),
        key,
        param,
    )
    .await?;
    Ok(GetJifenRecordRes {
        count: res.len() as u32,
        rows: res,
    }
    .into())
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
struct GetJifenGoodsReq {
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub page: Option<u32>,
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub pageSize: Option<u32>,
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub name: Option<String>,
}
#[derive(Serialize, Debug)]
struct GetJifenGoodsRes {
    pub count: u32,
    pub rows: Vec<JifenGoods>,
}
#[handler]
async fn get_jifen_goods(req: &mut Request) -> RouterResult {
    let GetJifenGoodsReq {
        page,
        pageSize,
        name,
    } = req.parse_queries()?;
    let res = service::jifen::get_goods_list(
        name,
        page.unwrap_or(1),
        pageSize.unwrap_or(10),
    )
    .await?;
    Ok(GetJifenGoodsRes {
        count: res.len() as u32,
        rows: res,
    }
    .into())
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
struct GetJifenRulesReq {
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub page: Option<u32>,
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub pageSize: Option<u32>,
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
#[handler]
async fn get_jifen_rules(req: &mut Request) -> RouterResult {
    let GetJifenRulesReq {
        page,
        pageSize,
        key,
        name,
    } = req.parse_queries()?;
    let res = service::jifen::get_jifen_rule_list(
        key,
        name,
        page.unwrap_or(1),
        pageSize.unwrap_or(10),
    )
    .await?;
    Ok(GetJifenRulesRes {
        count: res.len() as u32,
        rows: res,
    }
    .into())
}

#[derive(Deserialize, Debug)]
struct PostRecordReq {
    pub key: String,
    pub param: String,
}
#[derive(Serialize, Debug)]
struct PostRecordRes {
    pub jifen: u32,
}
/// 添加积分
#[handler]
async fn post_record(req: &mut Request) -> RouterResult {
    let PostRecordReq { key, param } = req.parse_json().await?;
    let (_, stu_id) = utils::jwt::auth(req)?;
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
    let now = chrono::Local::now();
    let create_time_greater_than =
        now - chrono::Duration::days(jifen_rule.cycle as i64 - 1);
    let count = service::jifen::get_jifen_record_count(
        &stu_id,
        &key,
        create_time_greater_than,
    )
    .await?;
    if count as i32 >= jifen_rule.maxCount {
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
    Ok(PostRecordRes { jifen: new_jifen }.into())
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
struct PostGoodsReq {
    pub goodsId: i32,
}
// 兑换商品
#[handler]
async fn post_goods(req: &mut Request) -> RouterResult {
    let PostGoodsReq { goodsId } = req.parse_json().await?;
    let (_, stu_id) = utils::jwt::auth(req)?;
    let goods = service::jifen::get_goods(goodsId as u32)
        .await?
        .ok_or(anyhow!("没有找到商品：{}", goodsId))?;
    // 检查商品库存
    if goods.enabled.is_none_or(|v| v == 0) {
        return Err("商品已下架".into());
    }
    if goods.count <= 0 {
        return Err("商品库存不足".into());
    }
    // 用户一定存在，因此不可能返回 None
    let user_jifen = service::jifen::get_jifen(&stu_id)
        .await?
        .expect("通过鉴权后用户不存在");
    if user_jifen < goods.price as u32 {
        return Err("积分不足".into());
    }
    // 检查积分是否足够
    let current_jifen = service::jifen::get_jifen(&stu_id)
        .await?
        .expect("通过鉴权后用户不存在");
    if current_jifen < goods.price as u32 {
        return Err("积分不足，无法兑换".into());
    }
    // 扣除积分并添加记录
    service::jifen::exchange_goods(&stu_id, goods).await?;
    Ok("兑换成功".into())
}

#[derive(Deserialize, Debug)]
struct GetWebviewReq {
    pub url: String,
}
#[derive(Serialize, Debug)]
struct GetWebviewRes {
    pub jifen: u32,
}
#[handler]
async fn get_webview_read(req: &mut Request) -> RouterResult {
    let GetWebviewReq { url } = req.parse_queries()?;
    let (_, stu_id) = utils::jwt::auth(req)?;
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
    let now = chrono::Local::now();
    let create_time_greater_than =
        now - chrono::Duration::days(jifen_rule.cycle as i64 - 1);
    let count = service::jifen::get_jifen_record_count(
        &stu_id,
        &key,
        create_time_greater_than,
    )
    .await?;
    if count as i32 >= jifen_rule.maxCount {
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
    Ok(GetWebviewRes { jifen: new_jifen }.into())
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test() {
        // 查询周期内的积分记录
        let now = chrono::Local::now();
        let create_time_greater_than =
            now - chrono::Duration::days(1 + 1 - 1);
        let create_time_greater_than = create_time_greater_than
            .date_naive()
            .and_hms_opt(16, 0, 0)
            .unwrap();
        let create_time_greater_than_str = create_time_greater_than
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        println!("{}", create_time_greater_than);
        println!("{}", create_time_greater_than_str);
        let dt = chrono::NaiveDateTime::parse_from_str(
            "2024-03-02 11:04:49",
            "%Y-%m-%d %H:%M:%S",
        )
        .unwrap();
        dbg!(
            dt.checked_sub_offset(
                chrono::FixedOffset::east_opt(8 * 3600).unwrap()
            )
            .unwrap()
        );
    }
}
