use super::get_db_pool;
use crate::result::AppResult;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct CustomizeCourseInfo {
    pub classname: String,
    pub location: Option<String>,
    pub teachers: Option<String>,
    pub week: String,
    pub day: String,
    pub section: String,
    #[serde(rename = "classID")]
    pub id: u32,
}

pub async fn get_course_list(
    mini_bind_id: u32,
    xn: u32,
    xq: u32,
) -> AppResult<Vec<CustomizeCourseInfo>> {
    let res = sqlx::query_as!(
        CustomizeCourseInfo,
        r#"
        SELECT id, classname, location, teachers, week, day, section FROM mini_course WHERE xn = ? AND xq = ? AND mini_bind_id = ? AND deleted_at IS NULL
        "#,
        xn,
        xq-1,
        mini_bind_id,
    )
    .fetch_all(get_db_pool().await)
    .await?;
    Ok(res)
}

/// CustomizeCourseInfo 的 id 会被忽略
pub async fn add_course(
    course: CustomizeCourseInfo,
    xn: u32,
    xq: u32,
    mini_bind_id: u32,
) -> AppResult<()> {
    let now = chrono::Local::now();
    sqlx::query!(
        r#"
        INSERT INTO mini_course (classname, location, teachers, week, day, section, xn, xq, mini_bind_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        course.classname,
        course.location,
        course.teachers,
        course.week,
        course.day,
        course.section,
        xn,
        xq-1,
        mini_bind_id,
        now,
        now,
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}

pub async fn delete_course(
    course_id: u32,
    mini_bind_id: u32,
) -> AppResult<()> {
    let now = chrono::Local::now();
    sqlx::query!(
        r#"
        UPDATE mini_course SET updated_at = ?, deleted_at = ? WHERE id = ? AND mini_bind_id = ? AND deleted_at IS NULL
        "#,
        now,
        now,
        course_id,
        mini_bind_id,
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}
