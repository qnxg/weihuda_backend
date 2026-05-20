use hnu_query::xgxt::personal_info::Gender;
use salvo::{Request, Router, handler};
use serde::Serialize;

use crate::{
    result::RouterResult,
    routers::demo::{DEMO_NAME, DEMO_STU_ID},
    service, utils,
};

pub fn routers() -> Router {
    Router::with_path("info/user").get(get_user_info)
}

#[handler]
async fn get_user_info(req: &mut Request) -> RouterResult {
    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    struct GetUserInfoRes {
        pub class: String,
        pub name: String,
        pub major: String,
        pub enter: u32,
        pub college: String,
        pub sex: String,
        pub xz: Option<u8>,
        pub stu_id: String,
    }

    let stu_id = utils::jwt::auth(req)?;
    if stu_id == DEMO_STU_ID {
        return Ok(GetUserInfoRes {
            class: "计科2501班".to_string(),
            name: DEMO_NAME.to_string(),
            major: "计算机科学与技术".to_string(),
            enter: 2025,
            college: "计算机学院".to_string(),
            sex: "男".to_string(),
            xz: None,
            stu_id: DEMO_STU_ID.to_string(),
        }
        .into());
    }

    let user_info =
        service::user_info::get_person_info(&stu_id, false).await?;
    let res = GetUserInfoRes {
        class: user_info.class,
        name: user_info.name,
        major: user_info.major,
        enter: user_info.enter_year as u32,
        college: user_info.academy,
        sex: match user_info.gender {
            Gender::Male => "男",
            Gender::Female => "女",
        }
        .to_string(),
        xz: user_info.xz,
        stu_id: user_info.stu_id,
    };
    Ok(res.into())
}
