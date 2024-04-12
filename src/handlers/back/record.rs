use axum::{extract::State, Extension};

use crate::{
    app_result::{AppResult, AppState},
    dtos::back::record::{
        GetRecordGoodsReq, GetRecordReq, GetRecordRulesReq, GetWebviewReq, PostRecordReq,
    },
    entities::back::record::{
        GoodsReq, MiniBindRecord, PostRecordRes, Record, RecordGoods, RecordGoodsRes, RecordRes,
        RecordRules, RecordRulesRes,
    },
    extractors::{Json, Query},
    utils::jwt::parse_stu_id,
};

#[allow(non_snake_case)]
pub async fn get_record_total_handler(
    State(data): AppState,
    Extension(token): Extension<String>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;

    let res = sqlx::query_as!(
        MiniBindRecord,
        r#"
        SELECT stuID, jifen, id FROM mini_bind WHERE stuID = ?
        "#,
        stu_id
    )
    .fetch_one(&data.db)
    .await?;

    Ok(res.into())
}

#[allow(non_snake_case)]
pub async fn get_record_handler(
    State(data): AppState,
    Extension(token): Extension<String>,
    Query(req): Query<GetRecordReq>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let page = req.page.unwrap_or(1);
    let page_size = req.pageSize.unwrap_or(20);

    let offset = (page - 1) * page_size;
    let key = format!("%{}%", req.key.unwrap_or_default());
    let param = format!("{}%", req.param.unwrap_or_default());
    // let stu_id = format!("%{}%", stu_id);

    let res = sqlx::query_as!(
        Record,
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
        key,
        param,
        stu_id,
        offset,
        page_size,
    )
    .fetch_all(&data.db)
    .await?;

    let res = RecordRes { count: res.len() as u32, rows: res };
    Ok(res.into())
}

pub async fn get_record_goods_handler(
    State(data): AppState,
    Query(req): Query<GetRecordGoodsReq>,
) -> AppResult {
    let page = req.page.unwrap_or(1);
    let page_size = req.pageSize.unwrap_or(10);
    let name = req.name.unwrap_or_default();

    let offset = (page - 1) * page_size;
    let name = format!("%{}%", name);

    let res = sqlx::query_as!(
        RecordGoods,
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
        name,
        offset,
        page_size,
    )
    .fetch_all(&data.db)
    .await?;

    let res = RecordGoodsRes { count: res.len() as u32, rows: res };

    Ok(res.into())
}

#[allow(non_snake_case)]
pub async fn get_record_rules_handler(
    State(data): AppState,
    Query(req): Query<GetRecordRulesReq>,
) -> AppResult {
    let page = req.page.unwrap_or(1);
    let pageSize = req.pageSize.unwrap_or(10);
    let offset = (page - 1) * pageSize;

    let key = format!("%{}%", req.key.unwrap_or_default());
    let name = format!("%{}%", req.name.unwrap_or_default());

    let res = sqlx::query_as!(
        RecordRules,
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
        key,
        name,
        offset,
        pageSize
    )
    .fetch_all(&data.db)
    .await?;

    let res = RecordRulesRes { count: res.len() as u32, rows: res };
    Ok(res.into())
}

#[allow(non_snake_case)]
pub async fn post_record_handler(
    State(data): AppState,
    Extension(token): Extension<String>,
    Json(req): Json<PostRecordReq>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;

    // 查询积分规则
    let record_rule: RecordRules = sqlx::query_as!(
        RecordRules,
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
        req.key
    )
    .fetch_one(&data.db)
    .await?;

    // 查询是否重复添加
    let record = sqlx::query!(
        r#"
        SELECT 
            id
        FROM 
            jifen_records
        WHERE 
            `key` = ? AND param = ? AND stuId = ?
            AND deletedAt IS NULL
        "#,
        req.key,
        req.param,
        stu_id
    )
    .fetch_one(&data.db)
    .await;

    if record.is_ok() {
        return Err("已经添加过积分记录".into());
    }

    // 查询周期内的积分记录
    let now = chrono::Local::now();
    let create_time_greater_than = now - chrono::Duration::days(record_rule.cycle as i64 - 1);
    let create_time_greater_than_str = create_time_greater_than.format("%Y-%m-%d").to_string();

    // 统计数量
    let records = sqlx::query!(
        r#"
        SELECT 
            id
        FROM 
            jifen_records
        WHERE 
            `key` = ? AND stuId = ? AND createTime >= ?
            AND deletedAt IS NULL
        "#,
        req.key,
        stu_id,
        create_time_greater_than_str
    )
    .fetch_all(&data.db)
    .await?;

    if records.len() as i32 >= record_rule.maxCount {
        return Err("超过周期内最大次数".into());
    }

    // 添加积分记录
    let now = chrono::Local::now();
    let _ = sqlx::query!(
        r#"
        INSERT INTO jifen_records (`key`, param, stuId, `desc`, jifen, createTime, createdAt, updatedAt)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        req.key,
        req.param,
        stu_id,
        record_rule.name,
        record_rule.jifen,
        now,
        now,
        now
    ).execute(&data.db).await?;

    // 增加积分
    let _ = sqlx::query!(
        r#"
        UPDATE mini_bind
        SET jifen = jifen + ?
        WHERE stuID = ?
        "#,
        record_rule.jifen,
        stu_id
    )
    .execute(&data.db)
    .await?;

    // 获取增加后的积分数值
    let mini_bind = sqlx::query_as!(
        MiniBindRecord,
        r#"
        SELECT stuID, jifen, id FROM mini_bind WHERE stuID = ?
        "#,
        stu_id
    )
    .fetch_one(&data.db)
    .await?;

    let res = PostRecordRes { jifen: mini_bind.jifen.unwrap() as i32 };

    Ok(res.into())
}

#[allow(non_snake_case)]
pub async fn post_goods_handler(
    State(data): AppState,
    Extension(token): Extension<String>,
    Json(req): Json<GoodsReq>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    // 通过goodsId查询商品
    let goods: RecordGoods = sqlx::query_as!(
        RecordGoods,
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
        req.goodsId
    )
    .fetch_one(&data.db)
    .await?;
    // 通过stuId查找学生
    let mini_bind: MiniBindRecord = sqlx::query_as!(
        MiniBindRecord,
        r#"
        SELECT stuID, jifen, id FROM mini_bind WHERE stuID = ?
        "#,
        stu_id
    )
    .fetch_one(&data.db)
    .await?;
    // 检查商品库存
    if goods.enabled.is_none() || goods.enabled.unwrap() == 0 {
        return Err("商品已下架".into());
    }
    if goods.count <= 0 {
        return Err("商品库存不足".into());
    }
    if mini_bind.jifen.unwrap() < goods.price as u32 {
        return Err("积分不足".into());
    }
    // 减少库存，通过update方式操作
    let _ = sqlx::query!(
        r#"
        UPDATE jifen_goods
        SET count = count - 1
        WHERE id = ?
        "#,
        req.goodsId
    )
    .execute(&data.db)
    .await?;
    // 减少学生的积分
    let _ = sqlx::query!(
        r#"
        UPDATE mini_bind
        SET jifen = jifen - ?
        WHERE stuID = ?
        "#,
        goods.price,
        stu_id
    )
    .execute(&data.db)
    .await?;
    // 添加积分记录
    let now = chrono::Local::now();
    let key = "exchange".to_string();
    let param = req.goodsId.to_string();
    let desc = format!("兑换商品{}", goods.name);
    let _ = sqlx::query!(
        r#"
        INSERT INTO jifen_records (`key`, param, stuId, `desc`, jifen, createTime, createdAt, updatedAt)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        key,
        param,
        stu_id,
        desc,
        -goods.price,
        now,
        now,
        now
    ).execute(&data.db).await?;
    // 添加商品记录
    let _ = sqlx::query!(
        r#"
        INSERT INTO goods_records (goodsId, stuId, exchangeTime, status, comment, createdAt, updatedAt)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
        req.goodsId,
        stu_id,
        now,
        0,
        desc,
        now,
        now
    ).execute(&data.db).await?;

    Ok("兑换成功".into())
}

#[allow(non_snake_case)]
pub async fn get_webview_read_handler(
    State(data): AppState,
    Query(req): Query<GetWebviewReq>,
    Extension(token): Extension<String>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let key = "yuedu".to_string();
    let param = req.url;

    // 查询积分规则
    let record_rule: RecordRules = sqlx::query_as!(
        RecordRules,
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
        "#,
        key
    )
    .fetch_one(&data.db)
    .await?;

    // 查询是否重复添加
    let record: Result<Record, _> = sqlx::query_as!(
        Record,
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
        "#,
        key,
        param,
        stu_id
    )
    .fetch_one(&data.db)
    .await;

    if record.is_ok() {
        return Ok("已经添加过积分记录".into());
    }

    // 查询周期内的积分记录
    let now = chrono::Local::now();
    let create_time_greater_than = now - chrono::Duration::days(record_rule.cycle as i64 - 1);
    let create_time_greater_than_naive_datetime = create_time_greater_than.date_naive().and_hms_opt(0, 0, 0).unwrap();  // 设置时分秒为0

    let count = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) as count
        FROM 
            jifen_records
        WHERE 
            `key` = ? AND stuId = ? AND createTime >= ?
        "#,
        key,
        stu_id,
        create_time_greater_than_naive_datetime
    )
    .fetch_one(&data.db)
    .await?
    .count;

    if count as i32 >= record_rule.maxCount {
        return Ok("超过周期内最大次数".into());
    }

    // 添加积分记录
    let now = chrono::Local::now();
    let _ = sqlx::query!(
        r#"
        INSERT INTO jifen_records (`key`, param, stuId, `desc`, jifen, createTime, createdAt, updatedAt)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        key,
        param,
        stu_id,
        record_rule.name,
        record_rule.jifen,
        now,
        now,
        now
    ).execute(&data.db).await?;

    // 增加积分
    let _ = sqlx::query!(
        r#"
        UPDATE mini_bind
        SET jifen = jifen + ?
        WHERE stuID = ?
        "#,
        record_rule.jifen,
        stu_id
    )
    .execute(&data.db)
    .await?;

    // 获取增加后的积分数值
    let mini_bind = sqlx::query_as!(
        MiniBindRecord,
        r#"
        SELECT stuID, jifen, id FROM mini_bind WHERE stuID = ?
        "#,
        stu_id
    )
    .fetch_one(&data.db)
    .await?;

    let res = PostRecordRes { jifen: mini_bind.jifen.unwrap() as i32 };

    Ok(res.into())
}

// #[allow(non_snake_case)]
// 获取积分记录列表
// pub async fn get_record_goods_list_handler(
//     State(data): State<Arc<Pool>>,
//     Extension(token): Extension<String>,
//     Query(req): Query<GetRecordReq>,
// ) -> AppResult {
//     let stu_id = parse_stu_id(&token)?;
//     let page = req.page.unwrap_or(1);
//     let pageSize = req.pageSize.unwrap_or(10);
//     if pageSize > 20 {
//         return Err("pageSize不能大于20".into())
//     }
//     let offset = (page - 1) * pageSize;
//     let key = format!("%{}%", req.key.unwrap_or_default());
//     let param = format!("%{}%", req.param.unwrap_or_default());
//     let stu_id = format!("%{}%", stu_id);

//     let res: Vec<RecordGoodsList> = sqlx::query_as!(
//         RecordGoodsList,
//         r#"
//         SELECT
//             id,
//             `key`,
//             param,
//             stuId,
//             description,
//             jifen,
//             createTime
//         FROM
//             jifen_records
//         WHERE
//             (`key` LIKE ? AND param LIKE ? AND stuId LIKE ?)
//         ORDER BY
//             id DESC
//         LIMIT
//             ?, ?
//         "#,
//         key, param, stu_id, offset, pageSize
//     ).fetch_all(&data.db).await?;

//     let res = RecordGoodsRes {
//         count: res.len() as u32,
//         rows: res,
//     };

//     Ok(res.into())
// }


#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test() {
        // 查询周期内的积分记录
        let now = chrono::Local::now();
        let create_time_greater_than = now - chrono::Duration::days(1 - 1);
        let create_time_greater_than = create_time_greater_than.date_naive().and_hms_opt(0, 0, 0).unwrap();
        let create_time_greater_than_str = create_time_greater_than.format("%Y-%m-%d %H:%M:%S").to_string();
        println!("{}", create_time_greater_than);
        println!("{}", create_time_greater_than_str);
        let dt = chrono::NaiveDateTime::parse_from_str("2024-03-02 11:04:49", "%Y-%m-%d %H:%M:%S").unwrap();
        dbg!(dt.checked_sub_offset(chrono::FixedOffset::east_opt(8 * 3600).unwrap()).unwrap());
    }
}