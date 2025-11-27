use salvo::{Request, Router, handler};
use serde::{Deserialize, Serialize};

use crate::{result::RouterResult, service};

pub fn routers() -> Router {
    // 这个信息完全来自小程序配置，根本就不应该在hdjw的路由里
    Router::new()
        .push(
            Router::with_path("hdjw/class-start-date")
                .get(get_class_start_date),
        )
        .push(
            Router::with_path("info/smester").get(get_semester_info),
        )
}

#[derive(Serialize, Debug)]
#[expect(non_snake_case)]
struct SemesterInfoRes {
    pub startDate: String,
    pub term: u32,
    pub year: u32,
    pub vacation: String,
    pub next: String,
}
/// 获取学期信息
#[handler]
async fn get_semester_info() -> RouterResult {
    let res = SemesterInfoRes {
        startDate: service::semester::get_this_semester_start_date()
            .await?,
        term: service::semester::get_now_xnxq().await?.1,
        year: service::semester::get_now_xnxq().await?.0,
        vacation: service::semester::get_vacation_date().await?,
        next: service::semester::get_next_semester_start_date()
            .await?,
    };
    Ok(res.into())
}

#[derive(Deserialize, Debug)]
struct GetClassStartDateReq {
    pub xn: u32,
    pub xq: u32,
}
/// 获取学期开始时间
#[handler]
async fn get_class_start_date(req: &mut Request) -> RouterResult {
    let query: GetClassStartDateReq = req.parse_queries()?;
    Ok(service::semester::get_class_start_date_by_xnxq(
        query.xn, query.xq,
    )
    .await
    .unwrap_or_default()
    .into())
}
