use salvo::{Request, Router, handler, macros::Extractible};
use serde::{Deserialize, Serialize};

use crate::{error::RouterResult, routers::ThrowParseError, service};

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

/// 获取学期信息
#[handler]
async fn get_semester_info() -> RouterResult {
    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    struct SemesterInfoRes {
        pub start_date: String,
        pub term: u32,
        pub year: u32,
        pub vacation: String,
        pub next: String,
    }
    let res = SemesterInfoRes {
        start_date: service::semester::get_this_semester_start_date()
            .await?,
        term: service::semester::get_now_xnxq().await?.1,
        year: service::semester::get_now_xnxq().await?.0,
        vacation: service::semester::get_vacation_date().await?,
        next: service::semester::get_next_semester_start_date()
            .await?,
    };
    Ok(res.into())
}

/// 获取学期开始时间
#[handler]
async fn get_class_start_date(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct GetClassStartDateReq {
        pub xn: u32,
        pub xq: u32,
    }
    let GetClassStartDateReq { xn, xq } =
        req.extract().await.parse_error()?;
    let res = service::semester::get_class_start_date_by_xnxq(xn, xq)
        .await?;
    Ok(res.into())
}
