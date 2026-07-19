use super::get_db_pool;
use crate::error::{AppResult, ThrowInternalErrorResult};
use crate::utils;
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MiniBind {
    pub stu_id: String,
    pub openid: Option<String>,
    pub qq_openid: Option<String>,
    pub password: String,
    pub lab_pass: Option<String>,
}

#[tracing::instrument(skip_all, fields(otel.kind = "client", event_type = "db"), err)]
pub async fn get_by_stu_id(
    stu_id: &str,
) -> AppResult<Option<MiniBind>> {
    let res = sqlx::query_as!(
        MiniBind,
        r#"
        SELECT 
        stuId as `stu_id`, openid, qqOpenid as `qq_openid`, password, labPass as `lab_pass`
        FROM mini_bind 
        WHERE stuId = ?
        "#,
        stu_id,
    ).fetch_optional(get_db_pool().await).await.internal_err()?;
    Ok(res)
}

/// 可能存在多个绑定，只返回最新的一个
#[tracing::instrument(skip_all, fields(otel.kind = "client", event_type = "db"), err)]
pub async fn get_by_openid(
    openid: &str,
) -> AppResult<Option<MiniBind>> {
    let res = sqlx::query_as!(
        MiniBind,
        r#"
        SELECT 
        stuId as `stu_id`, openid, qqOpenid as `qq_openid`, password, labPass as `lab_pass`
        FROM mini_bind 
        WHERE openid = ?
        ORDER BY createdAt DESC
        LIMIT 1
        "#,
        openid,
    ).fetch_optional(get_db_pool().await).await.internal_err()?;
    Ok(res)
}

/// 将指定 openid 的绑定信息删除
#[tracing::instrument(skip_all, fields(otel.kind = "client", event_type = "db"), err)]
pub async fn clear_openid(openid: &str) -> AppResult<()> {
    sqlx::query!(
        r#"
        UPDATE mini_bind SET openid = NULL WHERE openid = ?
        "#,
        openid
    )
    .execute(get_db_pool().await)
    .await
    .internal_err()?;
    Ok(())
}

/// 插入新用户绑定信息，如果用户已存在则更新
#[tracing::instrument(skip_all, fields(otel.kind = "client", event_type = "db"), err)]
pub async fn add_user(
    stu_id: &str,
    password: &str,
    openid: Option<&str>,
    qq_openid: Option<&str>,
) -> AppResult<()> {
    let now = utils::time::now_time();
    sqlx::query!(
        r#"
        INSERT INTO mini_bind 
        (stuId, password, openid, qqOpenid, jifen, createdAt, updatedAt) 
        VALUES (?, ?, ?, ?, 0, ?, ?)
        ON DUPLICATE KEY 
            UPDATE 
            password = VALUES(password), 
            openid = VALUES(openid), 
            qqOpenid = VALUES(qqOpenid), 
            updatedAt = VALUES(updatedAt)
        "#,
        stu_id,
        password,
        openid,
        qq_openid,
        now,
        now
    )
    .execute(get_db_pool().await)
    .await
    .internal_err()?;
    Ok(())
}

#[tracing::instrument(skip_all, fields(otel.kind = "client", event_type = "db"), err)]
pub async fn set_lab_password(
    stu_id: &str,
    lab_pass: &str,
) -> AppResult<()> {
    sqlx::query!(
        r#"
        UPDATE mini_bind SET labPass = ? WHERE stuId = ?
        "#,
        lab_pass,
        stu_id
    )
    .execute(get_db_pool().await)
    .await
    .internal_err()?;
    Ok(())
}

/// 返回 None 时，可能是用户不存在，也可能是对应的 room 字段就是空的
#[tracing::instrument(skip_all, fields(otel.kind = "client", event_type = "db"), err)]
pub async fn get_user_setting(
    stu_id: &str,
) -> AppResult<Option<Value>> {
    let res = sqlx::query!(
        "
        SELECT settings FROM mini_bind WHERE stuId = ?
        ",
        stu_id
    )
    .fetch_optional(get_db_pool().await)
    .await
    .internal_err()?
    .map(|r| r.settings);
    Ok(res.flatten())
}

#[tracing::instrument(skip_all, fields(otel.kind = "client", event_type = "db"), err)]
pub async fn update_user_setting(
    stu_id: &str,
    settings: &Value,
) -> AppResult<()> {
    sqlx::query!(
        r#"
        UPDATE mini_bind SET settings = ? WHERE stuId = ?
        "#,
        settings,
        stu_id
    )
    .execute(get_db_pool().await)
    .await
    .internal_err()?;
    Ok(())
}

#[tracing::instrument(skip_all, fields(otel.kind = "client", event_type = "db"), err)]
pub async fn get_password(stu_id: &str) -> AppResult<Option<String>> {
    let res = sqlx::query_scalar!(
        "SELECT password FROM mini_bind WHERE stuId = ?",
        stu_id
    )
    .fetch_optional(get_db_pool().await)
    .await
    .internal_err()?;
    Ok(res)
}

#[tracing::instrument(skip_all, fields(otel.kind = "client", event_type = "db"), err)]
pub async fn get_lab_password(
    stu_id: &str,
) -> AppResult<Option<String>> {
    let res = sqlx::query_scalar!(
        "SELECT labPass FROM mini_bind WHERE stuId = ?",
        stu_id
    )
    .fetch_optional(get_db_pool().await)
    .await
    .internal_err()?;
    Ok(res.flatten())
}
