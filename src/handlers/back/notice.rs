use axum::{
    extract::{Path, State},
    Extension,
};
use chrono::NaiveDateTime;
use serde::Serialize;

use crate::{
    app_result::{AppResult, AppState},
    dtos::back::notice::{GetNoticeReq, PostMessageLeft, PutNoticeByIdReq},
    extractors::{Json, Query},
    utils::jwt::parse_stu_id,
};

#[allow(non_snake_case)]
#[derive(Serialize, Debug)]
pub struct Notice {
    pub id: u32,
    pub content: String,
    pub stuId: String,
    pub sendTime: NaiveDateTime,
    pub isShow: Option<i8>,
    pub status: Option<i32>,
    pub result: Option<String>,
    pub btnConfig: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct NoticeRes {
    pub count: u32,
    pub rows: Vec<Notice>,
}

#[allow(non_snake_case)]
pub async fn get_notice_handler(
    State(data): AppState,
    Query(req): Query<GetNoticeReq>,
    Extension(token): Extension<String>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    // let sendTime = format!(
    //     "%{}%",
    //     if req.sendTime.is_some() {
    //         req.sendTime.unwrap().to_string()
    //     } else {
    //         "".to_string()
    //     }
    // );
    // let status = format!(
    //     "%{}%",
    //     if req.status.is_some() {
    //         req.status.unwrap().to_string()
    //     } else {
    //         "".to_string()
    //     }
    // );
    // let result = format!("%{}%", req.result.unwrap_or_default());    // 这里有bug，数据库为null时候无法查出
    let page = req.page.unwrap_or(1);
    let pageSize = req.pageSize.unwrap_or(10);
    let offset = (page - 1) * pageSize;

    let res = sqlx::query_as!(
        Notice,
        r#"
        SELECT 
            id,
            content,
            stuId,
            sendTime,
            isShow,
            status,
            result,
            btnConfig
        From 
            notices
        WHERE 
            stuId = ?
            AND deletedAt IS NULL
        ORDER BY 
            id DESC
        LIMIT 
            ?, ?
        "#,
        stu_id,
        offset,
        pageSize,
    )
    .fetch_all(&data.db)
    .await?;

    let res = NoticeRes { count: res.len() as u32, rows: res };

    Ok(res.into())
}

#[allow(non_snake_case)]
pub async fn put_notice_by_id_handler(
    State(data): AppState,
    Path(id): Path<u32>,
    Json(json): Json<PutNoticeByIdReq>,
) -> AppResult {
    let result = json.result;
    let status = json.status;
    if let Some(status) = status {
        if status != 0 && status != 1 {
            return Err("status must be 0 or 1".into());
        }
    }

    // 更新result和status
    if result.is_some() && status.is_some() {
        let _ = sqlx::query!(
            r#"
            UPDATE 
                notices
            SET 
                result = ?,
                status = ?
            WHERE 
                id = ?
            "#,
            result.unwrap(),
            status.unwrap(),
            id,
        )
        .execute(&data.db)
        .await?;
    } else if result.is_some() {
        let _ = sqlx::query!(
            r#"
            UPDATE 
                notices
            SET 
                result = ?
            WHERE 
                id = ?
            "#,
            result.unwrap(),
            id,
        )
        .execute(&data.db)
        .await?;
    } else {
        let _ = sqlx::query!(
            r#"
            UPDATE 
                notices
            SET 
                status = ?
            WHERE 
                id = ?
            "#,
            status.unwrap(),
            id,
        )
        .execute(&data.db)
        .await?;
    }
    Ok("更新通知状态成功".into())
}
pub async fn post_message_left_handler(
    State(data): AppState,
    Json(json): Json<PostMessageLeft>,
) -> AppResult {
    let _ = sqlx::query!(
        r#"
        INSERT INTO 
            message_lefts (stuId, `desc`, isAgree, sendTime,isSend)
        VALUES 
            (?, ?, ?, ?,?)
        "#,
        json.stu_id,
        json.desc,
        json.is_agree,
        json.send_time,
        json.is_send,
    )
    .execute(&data.db)
    .await?;
    Ok("留言成功".into())
}
