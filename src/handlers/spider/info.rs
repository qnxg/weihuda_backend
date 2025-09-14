use axum::Extension;

use crate::{
    app_result::AppResult,
    dtos::spider::xgxt::PersonInfo,
    entities::spider::info::{SemesterInfoRes, UserInfoRes},
    utils::{
        jwt::parse_stu_id,
        request::spider_data,
        semester::{
            get_next_semester_start_date, get_vacation_date, get_now_xnxq,
            get_this_semester_start_date,
        },
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

pub async fn get_user_info_handler(Extension(token): Extension<String>) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let params = [("stuid", stu_id)];
    let spider_res: PersonInfo = spider_data("/xgxt/person_info", &params).await?;

    let res = UserInfoRes {
        class: spider_res.class,
        name: spider_res.name,
        major: spider_res.major,
        enter: spider_res.enter_year as u32,
        college: spider_res.academy,
        sex: spider_res.gender,
        xz: spider_res.xz as u32,
        stuId: spider_res.stu_id,
    };

    Ok(res.into())
}
