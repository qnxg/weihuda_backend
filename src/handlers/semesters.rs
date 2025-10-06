use axum::extract::Query;

use crate::{
    app_result::AppResult,
    dtos::spider::hdjw::GetClassStartDateReq,
    entities::spider::info::SemesterInfoRes,
    utils::semester::{
        get_class_start_date_by_xnxq, get_next_semester_start_date,
        get_now_xnxq, get_this_semester_start_date,
        get_vacation_date,
    },
};

pub async fn get_semester_info_handler() -> AppResult {
    let res = SemesterInfoRes {
        startDate: get_this_semester_start_date(),
        term: get_now_xnxq().1,
        year: get_now_xnxq().0,
        vacation: get_vacation_date().await,
        next: get_next_semester_start_date(),
    };
    Ok(res.into())
}

pub async fn get_class_start_date_handler(
    Query(req): Query<GetClassStartDateReq>,
) -> AppResult {
    Ok(get_class_start_date_by_xnxq(req.xn, req.xq)
        .unwrap_or_default()
        .into())
}
