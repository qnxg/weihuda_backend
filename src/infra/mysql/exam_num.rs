use super::get_db_pool;
use crate::result::AppResult;
use serde::Serialize;
use sqlx::FromRow;
#[derive(FromRow, Serialize)]
pub struct ExamNumberInfo {
    #[serde(rename = "num")]
    pub exam_num: String,
    #[serde(rename = "name")]
    pub exam_name: String,
    #[serde(rename = "date")]
    pub exam_date: String,
    pub id: u32,
}

pub async fn get_exam_num_list(
    mini_bind_id: u32,
) -> AppResult<Vec<ExamNumberInfo>> {
    let res = sqlx::query_as!(
        ExamNumberInfo,
        r#"
        SELECT id, exam_name, exam_num, exam_date FROM mini_exam_num WHERE mini_bind_id = ? AND deleted_at IS NULL
        "#,
        mini_bind_id,
    )
    .fetch_all(get_db_pool().await)
    .await?;
    Ok(res)
}

/// exam_num 的 id 会被忽略
pub async fn add_exam_num(
    mini_bind_id: u32,
    exam_num: ExamNumberInfo,
) -> AppResult<()> {
    let now = chrono::Local::now();
    sqlx::query!(
        r#"
        INSERT INTO mini_exam_num (exam_name, exam_num, exam_date, mini_bind_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)
        "#,
        exam_num.exam_name,
        exam_num.exam_num,
        exam_num.exam_date,
        mini_bind_id,
        now,
        now,
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}

pub async fn delete_exam_num(
    mini_bind_id: u32,
    exam_num_id: u32,
) -> AppResult<()> {
    let now = chrono::Local::now();
    sqlx::query!(
        r#"
        UPDATE mini_exam_num SET updated_at = ?, deleted_at = ? WHERE id = ? AND mini_bind_id = ? AND deleted_at IS NULL
        "#,
        now,
        now,
        exam_num_id,
        mini_bind_id,
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}
