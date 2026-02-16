#![expect(unused)]
use super::get_db_pool;
use crate::{result::AppResult, utils};
use anyhow::anyhow;
use chrono::{DateTime, Local, NaiveDateTime};
use serde::Serialize;

/// 如果学号不存在，则返回 None
pub async fn get_jifen(stu_id: &str) -> AppResult<Option<i32>> {
    let res = sqlx::query_scalar!(
        r#"
        SELECT jifen FROM mini_bind WHERE stuId = ?
        "#,
        stu_id
    )
    .fetch_optional(get_db_pool().await)
    .await?;
    Ok(res)
}

/// 调用前请确保学号是存在的，否则不会产生任何影响
pub async fn update_jifen(
    stu_id: &str,
    increment: i32,
) -> AppResult<()> {
    sqlx::query!(
        r#"
        UPDATE mini_bind
        SET jifen = jifen + ?
        WHERE stuId = ?
        "#,
        increment,
        stu_id
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JifenRecord {
    pub id: u32,
    pub key: String,
    pub param: String,
    pub stu_id: String,
    pub jifen: i32,
    pub desc: String,
    pub created_at: NaiveDateTime,
}
/// key 和 param 是模糊查询。如果传 None 则表示不限制该字段
pub async fn get_jifen_record_list(
    stu_id: &str,
    page: u32,
    page_size: u32,
    key: Option<String>,
    param: Option<String>,
) -> AppResult<Vec<JifenRecord>> {
    let res = sqlx::query_as!(
        JifenRecord,
        r#"
        SELECT 
            id,
            `key`,
            param,
            stuId as stu_id,
            `desc`,
            jifen,
            createdAt as created_at
        FROM 
            jifen_records
        WHERE 
            `key` LIKE ? AND param LIKE ? AND stuId = ?
        ORDER BY 
            id DESC
        LIMIT 
            ?, ?
        "#,
        format!("%{}%", key.unwrap_or_default()),
        format!("{}%", param.unwrap_or_default()),
        stu_id,
        (page - 1) * page_size,
        page_size,
    )
    .fetch_all(get_db_pool().await)
    .await?;
    Ok(res)
}

/// 获取指定 key 和 param 的用户积分记录
pub async fn get_jifen_record(
    stu_id: &str,
    key: &str,
    param: &str,
) -> AppResult<Option<JifenRecord>> {
    let res = sqlx::query_as!(
        JifenRecord,
        r#"
        SELECT 
            id,
            `key`,
            param,
            stuId as stu_id,
            `desc`,
            jifen,
            createdAt as created_at
        FROM 
            jifen_records
        WHERE 
            `key` = ? AND param = ? AND stuId = ?
        "#,
        key,
        param,
        stu_id
    )
    .fetch_optional(get_db_pool().await)
    .await?;
    Ok(res)
}

/// 获取某个人指定时间之后的某 key 的积分记录的数量
pub async fn get_jifen_record_count(
    stu_id: &str,
    key: &str,
    since: NaiveDateTime,
) -> AppResult<u32> {
    let res = sqlx::query_scalar!(
        r#"
        SELECT 
            count(*) as count
        FROM 
            jifen_records
        WHERE 
            `key` = ? AND stuId = ? AND createdAt >= ?
        "#,
        key,
        stu_id,
        since
    )
    .fetch_one(get_db_pool().await)
    .await?;
    Ok(res as u32)
}
pub async fn add_jifen_record(
    stu_id: &str,
    key: &str,
    param: &str,
    jifen: i32,
    desc: &str,
) -> AppResult<u64> {
    let now = utils::time::now_time();
    let res = sqlx::query!(
        r#"
        INSERT INTO jifen_records
            (stuId, `key`, param, jifen, `desc`, createdAt)
        VALUES
            (?, ?, ?, ?, ?, ?)
        "#,
        stu_id,
        key,
        param,
        jifen,
        desc,
        now
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(res.last_insert_id())
}

#[derive(Serialize, Debug)]
pub struct JifenGoods {
    pub id: u32,
    pub name: String,
    pub cover: String,
    pub count: u32,
    pub price: i32,
    pub description: Option<String>,
    pub enabled: bool,
}

/// name 是模糊查询。如果传 None 则表示不限制该字段
pub async fn get_goods_list(
    name: Option<String>,
    page: u32,
    page_size: u32,
    enabled: bool,
) -> AppResult<Vec<JifenGoods>> {
    let res = sqlx::query!(
        r#"
        SELECT 
            id,
            name,
            cover,
            count,
            price,
            description,
            enabled
        FROM 
            jifen_goods
        WHERE 
            name LIKE ?
            AND deletedAt IS NULL
            AND enabled = ?
        ORDER BY 
            id DESC
        LIMIT 
            ?, ?
        "#,
        format!("%{}%", name.unwrap_or_default()),
        enabled as u32,
        (page - 1) * page_size,
        page_size,
    )
    .fetch_all(get_db_pool().await)
    .await?
    .into_iter()
    .map(|row| JifenGoods {
        id: row.id,
        name: row.name,
        cover: row.cover,
        count: row.count,
        price: row.price,
        description: row.description,
        enabled: row.enabled != 0,
    })
    .collect();
    Ok(res)
}

pub async fn get_goods(
    goods_id: u32,
) -> AppResult<Option<JifenGoods>> {
    let goods = sqlx::query!(
        r#"
        SELECT 
            id,
            name,
            cover,
            count,
            price,
            description,
            enabled
        FROM 
            jifen_goods
        WHERE 
            id = ? AND deletedAt IS NULL
        "#,
        goods_id
    )
    .fetch_optional(get_db_pool().await)
    .await?
    .map(|row| JifenGoods {
        id: row.id,
        name: row.name,
        cover: row.cover,
        count: row.count,
        price: row.price,
        description: row.description,
        enabled: row.enabled != 0,
    });
    Ok(goods)
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GoodsExchangeRecord {
    pub id: u32,
    pub stu_id: String,
    pub goods_id: u32,
    pub status: u32,
    pub receive_time: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

pub async fn get_exchange_record_list(
    stu_id: &str,
    page: u32,
    page_size: u32,
) -> AppResult<Vec<GoodsExchangeRecord>> {
    let res = sqlx::query_as!(
        GoodsExchangeRecord,
        r#"
        SELECT 
            id,
            stuId as stu_id,
            goodsId as goods_id,
            status,
            receiveTime as receive_time,
            createdAt as created_at
        FROM 
            jifen_exchange
        WHERE 
            stuId = ?
        ORDER BY id DESC
        LIMIT ?
        OFFSET ?
        "#,
        stu_id,
        page_size,
        (page - 1) * page_size,
    )
    .fetch_all(get_db_pool().await)
    .await?;
    Ok(res)
}

pub async fn add_exchange_record(
    stu_id: &str,
    goods_id: u32,
) -> AppResult<u64> {
    let now = utils::time::now_time();
    let res = sqlx::query!(
        r#"
        INSERT INTO jifen_exchange (goodsId, stuId, status, createdAt)
        VALUES (?, ?, ?, ?)
        "#,
        goods_id,
        stu_id,
        0,
        now
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(res.last_insert_id())
}

pub async fn update_goods_count(
    goods_id: u32,
    decrement: i32,
) -> AppResult<()> {
    sqlx::query!(
        r#"
        UPDATE jifen_goods
        SET count = count - ?
        WHERE id = ? AND deletedAt IS NULL
        "#,
        decrement,
        goods_id
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JifenRule {
    pub id: u32,
    pub key: String,
    pub name: String,
    pub jifen: i32,
    pub cycle: u32,
    pub max_count: u32,
    pub is_show: bool,
}
/// name 和 key 是模糊查询。如果传 None 则表示不限制该字段
pub async fn get_jifen_rule_list(
    key: Option<String>,
    name: Option<String>,
    page: u32,
    page_size: u32,
    is_show: bool,
) -> AppResult<Vec<JifenRule>> {
    let res = sqlx::query!(
        r#"
        SELECT 
            id,
            `key`,
            name,
            jifen,
            cycle,
            maxCount,
            isShow
        From 
            jifen_rules
        WHERE 
            `key` LIKE ? AND name LIKE ? AND isShow = ?
            AND deletedAt IS NULL
        ORDER BY id DESC
        LIMIT ?
        OFFSET ?
        "#,
        format!("%{}%", key.unwrap_or_default()),
        format!("%{}%", name.unwrap_or_default()),
        is_show as u32,
        page_size,
        (page - 1) * page_size,
    )
    .fetch_all(get_db_pool().await)
    .await?
    .into_iter()
    .map(|row| JifenRule {
        id: row.id,
        key: row.key,
        name: row.name,
        jifen: row.jifen,
        cycle: row.cycle,
        max_count: row.maxCount,
        is_show: row.isShow != 0,
    })
    .collect();
    Ok(res)
}

/// 获取指定 key 的积分规则
pub async fn get_jifen_rule(
    key: &str,
) -> AppResult<Option<JifenRule>> {
    let res = sqlx::query!(
        r#"
        SELECT 
            id,
            `key`,
            name,
            jifen,
            cycle,
            maxCount,
            isShow
        From 
            jifen_rules
        WHERE 
            `key` = ?
            AND deletedAt IS NULL
        "#,
        key
    )
    .fetch_optional(get_db_pool().await)
    .await?
    .map(|row| JifenRule {
        id: row.id,
        key: row.key,
        name: row.name,
        jifen: row.jifen,
        cycle: row.cycle,
        max_count: row.maxCount,
        is_show: row.isShow != 0,
    });
    Ok(res)
}
