use axum::Extension;
use tokio::try_join;

use crate::entities::spider::library::{
    Borrowed, Borrowing, Finance, LibraryRes,
};
use crate::{
    app_result::AppResult,
    utils::{jwt::parse_stu_id, request::spider_data},
};

pub async fn get_library_handler(
    Extension(token): Extension<String>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let params = [("stuid", stu_id)];
    let (borrowed, borrowing, finance): (
        Vec<Borrowed>,
        Vec<Borrowing>,
        Vec<Finance>,
    ) = try_join!(
        spider_data("/library/history_loan", &params),
        spider_data("/library/current_loan", &params),
        spider_data("/library/finance", &params),
    )?;
    let res = LibraryRes {
        borrowed,
        borrowing,
        finance,
    };
    Ok(res.into())
}
