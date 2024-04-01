use crate::{app_result::AppState, extractors::{Json, Query}};
use axum::{extract::State, Extension};

use crate::{
    app_result::AppResult,
    entities::back::exam_num::ExamNumberInfo,
    dtos::back::exam_num::{AddExamNumberReq, DeleteExamNumberReq, UpdateExamNumberReq},
    utils::jwt::parse_id,
};

pub async fn get_exam_num_handler(
    State(data): AppState,
    Extension(token): Extension<String>,
) -> AppResult {
    let mini_bind_id = parse_id(&token)?;

    let res = sqlx::query_as!(
        ExamNumberInfo,
        r#"
        SELECT id, exam_name, exam_num, exam_date FROM mini_exam_num WHERE mini_bind_id = ? AND deleted_at IS NULL
        "#,
        mini_bind_id,
    )
    .fetch_all(&data.db)
    .await?;

    Ok(res.into())
}

pub async fn add_exam_num_handler(
    State(data): AppState,
    Extension(token): Extension<String>,
    Json(req): Json<AddExamNumberReq>,
) -> AppResult {
    let mini_bind_id = parse_id(&token)?;

    let now = chrono::Local::now();

    let account = sqlx::query!(
        r#"
        INSERT INTO mini_exam_num (exam_name, exam_num, exam_date, mini_bind_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)
        "#,
        req.name,
        req.num,
        req.date,
        mini_bind_id,
        now,
        now,
    )
    .execute(&data.db) 
    .await?;

    if account.rows_affected() == 0 {
        return Err("添加失败".into());
    }

    Ok("添加成功".into())
}

#[allow(dead_code)]
pub async fn update_exam_num_handler(
    State(data): AppState,
    Extension(token): Extension<String>,
    Json(req): Json<UpdateExamNumberReq>,
) -> AppResult {
    let mini_bind_id = parse_id(&token)?;

    let now = chrono::Local::now();

    let account = sqlx::query!(
        r#"
        UPDATE mini_exam_num SET exam_name = ?, exam_num = ?, exam_date = ?, updated_at = ? WHERE id = ? AND mini_bind_id = ? AND deleted_at IS NULL
        "#,
        req.name,
        req.num,
        req.date,
        now,
        req.id,
        mini_bind_id,
    )
    .execute(&data.db) 
    .await?;

    if account.rows_affected() == 0 {
        return Err("找不到指定更新项".into());
    }

    Ok("更新成功".into())
}

pub async fn delete_exam_num_handler(
    State(data): AppState,
    Query(req): Query<DeleteExamNumberReq>,
    Extension(token): Extension<String>,
) -> AppResult {
    let mini_bind_id = parse_id(&token)?;

    let now = chrono::Local::now();

    let account = sqlx::query!(
        r#"
        UPDATE mini_exam_num SET updated_at = ?, deleted_at = ? WHERE id = ? AND mini_bind_id = ? AND deleted_at IS NULL
        "#,
        now,
        now,
        req.id,
        mini_bind_id,
    )
    .execute(&data.db) 
    .await?;

    if account.rows_affected() == 0 {
        return Err("找不到指定删除项".into());
    }

    Ok("删除成功".into())
}
