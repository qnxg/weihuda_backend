use axum::Extension;

use crate::{
    app_result::AppResult,
    entities::spider::info::{SemesterInfoRes, SpiderUserInfo, UserInfoRes},
    utils::{jwt::parse_stu_id, request::spider_data, semester::{get_next_semester_start_date, get_next_vacation, get_now_xnxq, get_this_semester_start_date}},
};

pub async fn get_semester_info_handler() -> AppResult {
    let res = SemesterInfoRes {
        startDate: get_this_semester_start_date(),
        term: get_now_xnxq().1,
        year: get_now_xnxq().0,
        vacation: get_next_vacation().await,
        next: get_next_semester_start_date(),
    };
    Ok(res.into())
}

pub async fn get_user_info_handler(Extension(token): Extension<String>) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let params = [("stuid", stu_id)];
    let spider_res: SpiderUserInfo = spider_data("/bks/personInfo", &params).await?;

    let res = UserInfoRes {
        class: spider_res.bj_name.unwrap_or_default(),
        name: spider_res.name,
        major: spider_res.ndzy_name,
        enter: spider_res.rxnf.parse::<u32>().unwrap(), // 不应该出现错误，直接unwrap
        college: spider_res.skdw_name,
        sex: spider_res.xb,
        xz: spider_res.xz.parse::<u32>().unwrap(), // 不应该出现错误，直接unwrap
        stuId: spider_res.xh,
    };

    Ok(res.into())
}
