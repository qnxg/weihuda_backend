use salvo::{Request, Router, handler};
use serde::Serialize;

use crate::{result::RouterResult, service, utils};

pub fn routers() -> Router {
    Router::with_path("info/user").get(get_user_info)
}

#[derive(Serialize, Debug)]
#[expect(non_snake_case)]
struct GetUserInfoRes {
    pub class: String,
    pub name: String,
    pub major: String,
    pub enter: u32,
    pub college: String,
    pub sex: String,
    pub xz: u32,
    pub stuId: String,
}

#[handler]
async fn get_user_info(req: &mut Request) -> RouterResult {
    let (_, stu_id) = utils::jwt::auth(req)?;
    let user_info =
        service::user_info::get_person_info(&stu_id).await?;
    let res = GetUserInfoRes {
        class: user_info.class,
        name: user_info.name,
        major: user_info.major,
        enter: user_info.enter_year as u32,
        college: user_info.academy,
        sex: user_info.gender,
        xz: user_info.xz as u32,
        stuId: user_info.stu_id,
    };
    Ok(res.into())
}
