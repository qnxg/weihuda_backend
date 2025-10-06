use crate::{
    app_error::AppError, entities::back::mini_bind::MiniBind,
    handlers::back::common::wechat::get_openid, DbPool,
};
use std::sync::Arc;

pub async fn check_by_code(
    data: Arc<DbPool>,
    code: &str,
) -> Result<MiniBind, AppError> {
    let openid = get_openid(code).await?;
    let res = sqlx::query_as!(
        MiniBind,
        r#"
        SELECT id, openid, stuID, stuPASS, hdjwPASS FROM mini_bind WHERE openid = ? AND deleted_at is null
        "#,
        openid,
    ).fetch_one(&data.db).await?;
    Ok(res)
}

pub async fn check_by_stu_id(
    data: Arc<DbPool>,
    stu_id: &str,
) -> Result<MiniBind, AppError> {
    let res = sqlx::query_as!(
        MiniBind,
        r#"
        SELECT id, openid, stuID, stuPASS, hdjwPASS FROM mini_bind WHERE stuID = ? AND deleted_at is null
        "#,
        stu_id,
    ).fetch_one(&data.db).await?;
    Ok(res)
}

pub async fn check_by_openid(
    data: Arc<DbPool>,
    openid: &str,
) -> Result<MiniBind, AppError> {
    let res = sqlx::query_as!(
        MiniBind,
        r#"
        SELECT id, openid, stuID, stuPASS, hdjwPASS FROM mini_bind WHERE openid = ? AND deleted_at is null
        "#,
        openid,
    ).fetch_one(&data.db).await?;
    Ok(res)
}

#[expect(dead_code)]
pub async fn check_by_id(
    data: Arc<DbPool>,
    id: u32,
) -> Result<MiniBind, AppError> {
    let res = sqlx::query_as!(
        MiniBind,
        r#"
        SELECT id, openid, stuID, stuPASS, hdjwPASS FROM mini_bind WHERE id = ? AND deleted_at is null
        "#,
        id,
    ).fetch_one(&data.db).await?;
    Ok(res)
}
