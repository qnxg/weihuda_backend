use crate::{
    app_error::AppError, entities::back::mini_bind::MiniBind,
    handlers::back::common::wechat::get_openid, Pool,
};
use std::sync::Arc;

#[allow(non_snake_case)]
pub async fn check_by_code(data: Arc<Pool>, code: &str) -> Result<MiniBind, AppError> {
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

#[allow(dead_code)]
#[allow(non_snake_case)]
pub async fn check_by_openid(data: Arc<Pool>, openid: &str) -> Result<MiniBind, AppError> {
    let res = sqlx::query_as!(
        MiniBind,
        r#"
        SELECT id, openid, stuID, stuPASS, hdjwPASS FROM mini_bind WHERE openid = ? AND deleted_at is null
        "#,
        openid,
    ).fetch_one(&data.db).await?;
    Ok(res)
}

#[allow(dead_code)]
#[allow(non_snake_case)]
pub async fn check_by_id(data: Arc<Pool>, id: u32) -> Result<MiniBind, AppError> {
    let res = sqlx::query_as!(
        MiniBind,
        r#"
        SELECT id, openid, stuID, stuPASS, hdjwPASS FROM mini_bind WHERE id = ? AND deleted_at is null
        "#,
        id,
    ).fetch_one(&data.db).await?;
    Ok(res)
}
