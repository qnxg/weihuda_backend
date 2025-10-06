use axum::{
    extract::{Path, State},
    Extension,
};

use crate::{
    app_result::{AppResult, AppState},
    dtos::back::zhihu::{CrudZhihuByIdReq, GetZhihuPageReq},
    entities::back::zhihu::{ZhihuListItem, ZhihuPage},
    extractors::{Json, Query},
    utils::jwt::parse_stu_id,
};

/// 获取知湖文章列表
pub async fn get_zhihu_page_handler(
    State(data): AppState,
    Query(req): Query<GetZhihuPageReq>,
    Extension(token): Extension<String>,
) -> AppResult {
    let offset = req.offset;
    let req_count = req.req_count;

    if req_count > 100 {
        return Err("count不能大于100".into());
    }

    let title = format!("%{}%", req.title.unwrap_or_default());
    let _type = format!("%{}%", req._type.unwrap_or_default());
    let tags = format!("%{}%", req.tags.unwrap_or_default());
    let stu_id = parse_stu_id(&token)?;

    let res: Vec<ZhihuListItem> = sqlx::query_as!(
        ZhihuListItem,
        r#"
        SELECT 
            id, 
            title, 
            type AS _type, 
            tags, 
            cover, 
            IF(type = 'link', content, NULL) AS content, 
            status, 
            publishTime, 
            stuId 
        FROM 
            zhihus 
        WHERE 
            (title LIKE ? AND type LIKE ? AND tags LIKE ?) 
            AND (status = 1 OR stuId = ?)
            AND deletedAt IS NULL
        ORDER BY 
            id DESC 
        LIMIT 
            ?, ?;
        "#,
        title,
        _type,
        tags,
        stu_id,
        offset,
        req_count
    )
    .fetch_all(&data.db)
    .await?;

    let count = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) AS count
        FROM 
            zhihus 
        WHERE 
            (title LIKE ? AND type LIKE ? AND tags LIKE ?) 
            AND (status = 1 OR stuId = ?)
            AND deletedAt IS NULL
        "#,
        title,
        _type,
        tags,
        stu_id
    )
    .fetch_one(&data.db)
    .await?
    .count;

    let res: ZhihuPage = ZhihuPage {
        count: count as u32,
        rows: res,
    };

    Ok(res.into())
}

/// 通过id获取知湖文章信息
pub async fn get_zhihu_by_id_handler(
    State(data): AppState,
    Path(req): Path<CrudZhihuByIdReq>,
) -> AppResult {
    let res: ZhihuListItem = sqlx::query_as!(
        ZhihuListItem,
        r#"
        SELECT 
            id, 
            title, 
            type AS _type, 
            tags, 
            cover, 
            content,
            status, 
            publishTime, 
            stuId 
        FROM 
            zhihus 
        WHERE 
            id = ? AND deletedAt IS NULL;
        "#,
        req.id
    )
    .fetch_one(&data.db)
    .await?;

    Ok(res.into())
}

pub async fn post_zhihu_handler(
    State(data): AppState,
    Json(json): Json<ZhihuListItem>,
) -> AppResult {
    if let Some(_type) = json._type.clone() {
        // data必须是article和link之一
        if !["article", "link"].contains(&_type.as_str()) {
            return Err("类型必须为'article', 'link'中的一个".into());
        }
    } else {
        return Err("type不能为空".into());
    }

    if json.content.is_none() {
        return Err("content不能为空".into());
    }

    if let Some(status) = json.status {
        if ![0, 1].contains(&status) {
            return Err("status必须为0或1".into());
        }
    } else {
        return Err("status不能为空".into());
    }

    let now = chrono::Local::now();

    // 插入到数据库中
    let _ = sqlx::query!(
        r#"
        INSERT INTO zhihus (title, type, tags, cover, content, status, publishTime, stuId, createdAt, updatedAt) 
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
        "#,
        json.title,
        json._type,
        json.tags,
        json.cover,
        json.content,
        json.status,
        json.publishTime,
        json.stuId,
        now,
        now,
    ).execute(&data.db).await?;

    Ok(().into())
}

pub async fn put_zhihu_handler(
    Path(req): Path<CrudZhihuByIdReq>,
    State(data): AppState,
    Json(json): Json<ZhihuListItem>,
) -> AppResult {
    if let Some(_type) = json._type.clone() {
        if !["article", "link"].contains(&_type.as_str()) {
            return Err("类型必须为'article', 'link'中的一个".into());
        }
    } else {
        return Err("type不能为空".into());
    }

    if json.content.is_none() {
        return Err("content不能为空".into());
    }

    if let Some(status) = json.status {
        if ![0, 1, 2].contains(&status) {
            return Err("status必须为0或1或2".into());
        }
    } else {
        return Err("status不能为空".into());
    }

    let now = chrono::Local::now();

    // 插入到数据库中
    let _ = sqlx::query!(
        r#"
        UPDATE zhihus 
        SET 
            title = ?, 
            type = ?, 
            tags = ?, 
            cover = ?, 
            content = ?, 
            status = ?, 
            publishTime = ?, 
            stuId = ?, 
            updatedAt = ? 
        WHERE 
            id = ?;
        "#,
        json.title,
        json._type,
        json.tags,
        json.cover,
        json.content,
        json.status,
        json.publishTime,
        json.stuId,
        now,
        req.id,
    )
    .execute(&data.db)
    .await?;

    Ok(().into())
}

pub async fn delete_zhihu_handler(
    Path(req): Path<CrudZhihuByIdReq>,
    State(data): AppState,
) -> AppResult {
    let now = chrono::Local::now();
    let _ = sqlx::query!(
        r#"
        Update zhihus
        SET deletedAt = ?
        WHERE id = ?;
        "#,
        now,
        req.id
    )
    .execute(&data.db)
    .await?;

    Ok(().into())
}
