#![allow(non_snake_case)]
use axum::extract::State;
// use regex::Regex;

use crate::{
    app_result::{AppResult, AppState},
    dtos::back::feedback::{
        AddFeedbackReq, GetFeedbackReq, UpdateFeedbackReq,
    },
    entities::back::feedback::{FeedbackInfo, FeedbackRes},
    extractors::{Json, Query},
    rabbitmq::{self, RabbitMessage},
};

pub async fn get_feedback_handler(
    State(data): AppState,
    Query(req): Query<GetFeedbackReq>,
) -> AppResult {
    // if req.stuId.is_empty() {
    //     return Err("学号不能为空".into());
    // }
    if req.stuId.len() != 12 {
        return Err("学号格式不正确，学号长度应为12位".into());
    }
    let feedback_items: Vec<FeedbackInfo> = sqlx::query_as!(
        FeedbackInfo,
        r#"
        SELECT stuId, status, comment, `desc` FROM feedbacks WHERE stuId LIKE ? ORDER BY id DESC LIMIT 10 OFFSET ?
        "#,
        req.stuId,
        (req.page - 1) * 10
    )
    .fetch_all(&data.db)
    .await?;

    let res = FeedbackRes {
        count: feedback_items.len() as u32,
        rows: feedback_items,
    };

    Ok(res.into())
}

pub async fn add_feedback_handler(
    State(data): AppState,
    Json(req): Json<AddFeedbackReq>,
) -> AppResult {
    // 检查时间字符串格式为类似 2022-06-08 05:48:09
    // if !Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$")
    //     .unwrap()
    //     .is_match(&req.createTime)
    // {
    //     return Err("时间格式不正确".into());
    // }
    let now = chrono::Local::now();
    let result = sqlx::query!(
        r#"
        INSERT INTO feedbacks (stuId, `desc`, contact, imgUrl, type, createTime, createdAt, updatedAt) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        req.stuId,
        req.desc,
        req.contact,
        req.imgUrl,
        req._type,
        req.createTime,
        now,
        now,
    )
    .execute(&data.db)
    .await?;

    let msg = RabbitMessage::Feedback {
        stu_id: req.stuId,
        desc: req.desc,
        img_url: req.imgUrl,
        id: result.last_insert_id(),
    };
    rabbitmq::publish_message(msg).await?;

    Ok("添加反馈成功".into())
}

pub async fn update_feedback_handler(
    State(data): AppState,
    Json(req): Json<UpdateFeedbackReq>,
) -> AppResult {
    let now = chrono::Local::now();
    sqlx::query!(
        r#"
        UPDATE feedbacks SET status = ?, updatedAt = ? WHERE id = ?
        "#,
        req.status,
        now,
        req.id,
    )
    .execute(&data.db)
    .await?;

    Ok("修改反馈成功".into())
}
