use salvo::{Request, Router, handler};
use serde::Serialize;
use spider_2024::xgxt::personal_info::Gender;

use crate::{result::RouterResult, service, utils};

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
        pub xz: u32,
        pub stu_id: String,
    }
    let stu_id = utils::jwt::auth(req)?;
    let user_info =
        service::user_info::get_person_info(&stu_id).await?;
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
        xz: user_info.xz as u32,
        stu_id: user_info.stu_id,
    };
    Ok(res.into())
}
