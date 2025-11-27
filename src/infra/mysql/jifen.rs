use super::get_db_pool;
use crate::result::AppResult;
use chrono::{DateTime, Local, NaiveDateTime};
use serde::Serialize;

pub async fn get_jifen(stu_id: &str) -> AppResult<Option<u32>> {
    let res = sqlx::query_scalar!(
        "SELECT jifen FROM mini_bind WHERE stuID = ?",
        stu_id
    )
    .fetch_optional(get_db_pool().await)
    .await?;
    Ok(res.flatten())
}

pub async fn update_jifen(
    stu_id: &str,
    increment: i32,
) -> AppResult<()> {
    sqlx::query!(
        r#"
        UPDATE mini_bind
        SET jifen = jifen + ?
        WHERE stuID = ?
        "#,
        increment,
        stu_id
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}

#[derive(Serialize, Debug)]
#[expect(non_snake_case)]
pub struct JifenRecord {
    pub id: u32,
    pub key: String,
    pub param: String,
    pub stuId: String,
    pub jifen: i32,
    pub desc: String,
    pub createTime: NaiveDateTime,
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
            stuId,
            `desc`,
            jifen,
            createTime
        FROM 
            jifen_records
        WHERE 
            (`key` LIKE ? AND param LIKE ? AND stuId = ?)
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
            stuId,
            `desc`,
            jifen,
            createTime
        FROM 
            jifen_records
        WHERE 
            `key` = ? AND param = ? AND stuId = ?
            AND deletedAt IS NULL
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
    since: DateTime<Local>,
) -> AppResult<u32> {
    let res = sqlx::query_scalar!(
        r#"
        SELECT 
            count(*) as count
        FROM 
            jifen_records
        WHERE 
            `key` = ? AND stuId = ? AND createTime >= ?
            AND deletedAt IS NULL
        "#,
        key,
        stu_id,
        since.format("%Y-%m-%d").to_string()
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
    let now = chrono::Local::now();
    let res = sqlx::query!(
        r#"
        INSERT INTO jifen_records
            (stuId, `key`, param, jifen, `desc`, createTime, createdAt, updatedAt)
        VALUES
            (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        stu_id,
        key,
        param,
        jifen,
        desc,
        now,
        now,
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
    pub count: i32,
    pub price: i32,
    pub description: Option<String>,
    pub enabled: Option<i8>,
}
/// name 是模糊查询。如果传 None 则表示不限制该字段
pub async fn get_goods_list(
    name: Option<String>,
    page: u32,
    page_size: u32,
) -> AppResult<Vec<JifenGoods>> {
    let res = sqlx::query_as!(
        JifenGoods,
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
        ORDER BY 
            id DESC
        LIMIT 
            ?, ?
        "#,
        format!("%{}%", name.unwrap_or_default()),
        (page - 1) * page_size,
        page_size,
    )
    .fetch_all(get_db_pool().await)
    .await?;
    Ok(res)
}

pub async fn get_goods(
    goods_id: u32,
) -> AppResult<Option<JifenGoods>> {
    let goods = sqlx::query_as!(
        JifenGoods,
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
            id = ?
        "#,
        goods_id
    )
    .fetch_optional(get_db_pool().await)
    .await?;
    Ok(goods)
}

pub async fn add_goods_record(
    stu_id: &str,
    goods_id: u32,
    desc: &str,
) -> AppResult<u64> {
    let now = chrono::Local::now();
    let res = sqlx::query!(
        r#"
        INSERT INTO goods_records (goodsId, stuId, exchangeTime, status, comment, createdAt, updatedAt)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
        goods_id,
        stu_id,
        now,
        0,
        desc,
        now,
        now
    ).execute(get_db_pool().await).await?;
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
        WHERE id = ?
        "#,
        decrement,
        goods_id
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}

#[derive(Serialize, Debug)]
#[expect(non_snake_case)]
pub struct JifenRule {
    pub id: u32,
    pub key: String,
    pub name: String,
    pub jifen: i32,
    pub cycle: i32,
    pub maxCount: i32,
}
/// name 和 key 是模糊查询。如果传 None 则表示不限制该字段
pub async fn get_jifen_rule_list(
    key: Option<String>,
    name: Option<String>,
    page: u32,
    page_size: u32,
) -> AppResult<Vec<JifenRule>> {
    let res = sqlx::query_as!(
        JifenRule,
        r#"
        SELECT 
            id,
            `key`,
            name,
            jifen,
            cycle,
            maxCount
        From 
            jifen_rules
        WHERE 
            `key` LIKE ? AND name LIKE ? AND enabled = 1
            AND deletedAt IS NULL
        ORDER BY 
            id DESC
        LIMIT 
            ?, ?
        "#,
        format!("%{}%", key.unwrap_or_default()),
        format!("%{}%", name.unwrap_or_default()),
        (page - 1) * page_size,
        page_size
    )
    .fetch_all(get_db_pool().await)
    .await?;
    Ok(res)
}

/// 获取指定 key 的积分规则
pub async fn get_jifen_rule(
    key: &str,
) -> AppResult<Option<JifenRule>> {
    let res = sqlx::query_as!(
        JifenRule,
        r#"
        SELECT 
            id,
            `key`,
            name,
            jifen,
            cycle,
            maxCount
        From 
            jifen_rules
        WHERE 
            `key` = ?
            AND deletedAt IS NULL
        "#,
        key
    )
    .fetch_optional(get_db_pool().await)
    .await?;
    Ok(res)
}
