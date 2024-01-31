use crate::{
    app_result::AppResult, extract::Query, handler::back::common::check_user::check_by_code,
    schema::back::auth::AuthReq, utility::jwt::auth, Pool,
};
use axum::extract::State;
use std::sync::Arc;

pub async fn get_auth_handler(
    State(data): State<Arc<Pool>>,
    Query(req): Query<AuthReq>,
) -> AppResult {
    let user = check_by_code(data, &req.code).await?;
    //TODO 这里的判断有必要吗？
    if user.stuID.is_none() {
        return Err("找不到学号".into());
        // return Err(crate::app_error::AppError::SqlxError(sqlx::Error::RowNotFound));
    }
    let token = auth(user.id, &user.stuID.unwrap())?;
    Ok(token.into())
}
