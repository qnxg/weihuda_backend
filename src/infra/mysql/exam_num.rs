use super::get_db_pool;
use crate::error::{AppResult, ThrowInternalErrorResult};
use crate::utils;
use serde::Serialize;
use sqlx::FromRow;

#[derive(FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamNumberInfo {
    pub exam_num: String,
    pub exam_name: String,
    pub exam_date: String,
    pub id: u32,
}

#[tracing::instrument(skip_all, fields(otel.kind = "client", event_type = "db"), err)]
pub async fn get_exam_num_list(
    stu_id: &str,
) -> AppResult<Vec<ExamNumberInfo>> {
    let res = sqlx::query_as!(
        ExamNumberInfo,
        r#"
        SELECT id, examName as exam_name, examNum as exam_num, examDate as exam_date FROM mini_exam_num WHERE stuId = ? AND deletedAt IS NULL
        "#,
        stu_id,
    )
    .fetch_all(get_db_pool().await)
    .await
    .internal_err()?;
    Ok(res)
}

/// exam_num 的 id 会被忽略
#[tracing::instrument(skip_all, fields(otel.kind = "client", event_type = "db"), err)]
pub async fn add_exam_num(
    stu_id: &str,
    exam_num: ExamNumberInfo,
) -> AppResult<()> {
    let now = utils::time::now_time();
    sqlx::query!(
        r#"
        INSERT INTO mini_exam_num (examName, examNum, examDate, stuId, createdAt, updatedAt) VALUES (?, ?, ?, ?, ?, ?)
        "#,
        exam_num.exam_name,
        exam_num.exam_num,
        exam_num.exam_date,
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
pub async fn delete_exam_num(
    stu_id: &str,
    exam_num_id: u32,
) -> AppResult<()> {
    let now = utils::time::now_time();
    sqlx::query!(
        r#"
        UPDATE mini_exam_num SET updatedAt = ?, deletedAt = ? WHERE id = ? AND stuId = ? AND deletedAt IS NULL
        "#,
        now,
        now,
        exam_num_id,
        stu_id
    )
    .execute(get_db_pool().await)
    .await
    .internal_err()?;
    Ok(())
}
