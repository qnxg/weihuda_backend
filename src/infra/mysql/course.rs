use super::get_db_pool;
use crate::error::{AppResult, ThrowInternalErrorResult};
use crate::utils;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CustomizeCourseInfo {
    pub classname: String,
    pub location: Option<String>,
    pub teachers: Option<String>,
    pub week: String,
    pub day: String,
    pub section: String,
    pub id: u32,
}

#[tracing::instrument(skip_all, fields(otel.kind = "client", event_type = "db"), err)]
pub async fn get_course_list(
    stu_id: &str,
    xn: u16,
    xq: u8,
) -> AppResult<Vec<CustomizeCourseInfo>> {
    let res = sqlx::query_as!(
        CustomizeCourseInfo,
        r#"
        SELECT id, classname, location, teachers, week, day, section FROM mini_course WHERE xn = ? AND xq = ? AND stuId = ? AND deletedAt IS NULL
        "#,
        xn,
        xq-1,
        stu_id,
    )
    .fetch_all(get_db_pool().await)
    .await
    .internal_err()?;
    Ok(res)
}

/// CustomizeCourseInfo 的 id 会被忽略
#[tracing::instrument(skip_all, fields(otel.kind = "client", event_type = "db"), err)]
pub async fn add_course(
    course: CustomizeCourseInfo,
    xn: u32,
    xq: u32,
    stu_id: &str,
) -> AppResult<()> {
    let now = utils::time::now_time();
    sqlx::query!(
        r#"
        INSERT INTO mini_course (classname, location, teachers, week, day, section, xn, xq, stuId, createdAt, updatedAt) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        course.classname,
        course.location,
        course.teachers,
        course.week,
        course.day,
        course.section,
        xn,
        xq-1,
        stu_id,
        now,
        now,
    )
    .execute(get_db_pool().await)
    .await
    .internal_err()?;
    Ok(())
}

#[tracing::instrument(skip_all, fields(otel.kind = "client", event_type = "db"), err)]
pub async fn delete_course(
    course_id: u32,
    stu_id: &str,
) -> AppResult<()> {
    let now = utils::time::now_time();
    sqlx::query!(
        r#"
        UPDATE mini_course SET updatedAt = ?, deletedAt = ? WHERE id = ? AND stuId = ? AND deletedAt IS NULL
        "#,
        now,
        now,
        course_id,
        stu_id,
    )
    .execute(get_db_pool().await)
    .await
    .internal_err()?;
    Ok(())
}

/// 更新自定义课程
#[tracing::instrument(skip_all, fields(otel.kind = "client", event_type = "db"), err)]
pub async fn update_course(
    course_id: u32,
    stu_id: &str,
    course: CustomizeCourseInfo,
) -> AppResult<()> {
    let now = utils::time::now_time();
    sqlx::query!(
        r#"
        UPDATE mini_course
        SET classname = ?, location = ?, teachers = ?, week = ?, day = ?,section = ?, updatedAt = ?
        WHERE id = ? AND stuId = ? AND deletedAt IS NULL
        "#,

        course.classname,
        course.location,
        course.teachers,
        course.week,
        course.day,
        course.section,
        now,
        course_id,
        stu_id,
    )
    .execute(get_db_pool().await)
    .await
    .internal_err()?;
    Ok(())
}
#[tracing::instrument(skip_all, fields(otel.kind = "client", event_type = "db"), err)]
pub async fn get_custom_course_details_by_id(
    course_id: u32,
    stu_id: &str,
) -> AppResult<CustomizeCourseInfo> {
    let res = sqlx::query_as!(
        CustomizeCourseInfo,
        r#"
            SELECT 
            id, classname, location, teachers, week, day, section 
            FROM mini_course 
            WHERE id = ? 
            AND stuId = ? 
            AND deletedAt IS NULL 
            LIMIT 1
        "#,
        course_id,
        stu_id
    )
    .fetch_one(get_db_pool().await)
    .await
    .internal_err()?;
    Ok(res)
}
