use crate::{
    app_result::{AppResult, AppState}, dtos::back::course::{AddCourseReq, DeleteCourseReq, GetCourseReq}, entities::back::course::CourseInfo, extractors::{Json, Query}, utils::jwt::parse_id
};
use axum::{extract::State, Extension};

#[allow(dead_code)]
pub async fn get_course_handler(
    State(data): AppState,
    Query(req): Query<GetCourseReq>,
    Extension(token): Extension<String>,
) -> AppResult {
    let mini_bind_id = parse_id(&token)?;

    let res = sqlx::query_as!(
        CourseInfo,
        r#"
        SELECT id, classname, location, teachers, week, day, section FROM mini_course WHERE xn = ? AND xq = ? AND mini_bind_id = ? AND deleted_at IS NULL
        "#,
        req.xn,
        req.xq,
        mini_bind_id,
    )
    .fetch_all(&data.db)
    .await?; // the type of res is Vec<CourseInfo>

    Ok(res.into())
}

pub async fn add_course_handler(
    State(data): AppState,
    Extension(token): Extension<String>,
    Json(req): Json<AddCourseReq>,
) -> AppResult {
    let mini_bind_id = parse_id(&token)?;

    let now = chrono::Local::now();

    let account = sqlx::query!(
        r#"
        INSERT INTO mini_course (classname, location, teachers, week, day, section, xn, xq, mini_bind_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        req.classname,
        req.location,
        req.teachers,
        req.week,
        req.day,
        req.section,
        req.xn,
        req.xq - 1,
        mini_bind_id,
        now,
        now,
    )
    .execute(&data.db) 
    .await?;

    if account.rows_affected() == 0 {
        return Err("添加失败".into());
        // return Err(crate::app_error::AppError::SqlxError(sqlx::Error::RowNotFound));
    }

    Ok("添加成功".into())
}

pub async fn delete_course_handler(
    State(data): AppState,
    Query(req): Query<DeleteCourseReq>,
    Extension(token): Extension<String>,
) -> AppResult {
    let mini_bind_id = parse_id(&token)?;

    let now = chrono::Local::now();

    let account = sqlx::query!(
        r#"
        UPDATE mini_course SET updated_at = ?, deleted_at = ? WHERE id = ? AND mini_bind_id = ? AND deleted_at IS NULL
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
