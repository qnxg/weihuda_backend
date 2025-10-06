use axum::Extension;

use crate::{
    app_result::AppResult,
    dtos::spider::xgxt::PersonInfo,
    entities::spider::info::UserInfoRes,
    utils::{jwt::parse_stu_id, request::spider_data},
};

pub async fn get_user_info_handler(
    Extension(token): Extension<String>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let params = [("stuid", stu_id)];
    let spider_res: PersonInfo =
        spider_data("/xgxt/person_info", &params).await?;

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
