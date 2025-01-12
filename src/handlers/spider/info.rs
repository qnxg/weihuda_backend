use axum::Extension;

use crate::{
    app_result::AppResult,
    entities::spider::info::{SemesterInfoRes, SpiderUserInfo, UserInfoRes},
    utils::{jwt::parse_stu_id, request::spider_data},
};

///FIXME 每学期要手动更新数据
pub async fn get_semester_info_handler() -> AppResult {
    let res = SemesterInfoRes {
        startDate: "2024-09-08".to_string(),
        term: 1,
        year: 2024,
        vacation: "2025-01-19".to_string(),
        next: "2025-02-16".to_string(),
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
