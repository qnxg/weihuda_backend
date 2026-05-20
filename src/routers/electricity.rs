use salvo::{Request, Router, handler, macros::Extractible};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    result::RouterResult, routers::demo::DEMO_STU_ID, service, utils,
};

pub fn routers() -> Router {
    Router::new()
        .push(Router::with_path("electricity").get(get_electricity))
        .push(
            Router::with_path("dormitory")
                .push(Router::with_path("query").get(get_dormitory))
                .push(
                    Router::with_path("update").get(update_dormitory),
                ),
        )
}

#[handler]
async fn get_electricity(req: &mut Request) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    if stu_id == DEMO_STU_ID {
        return Ok("79.6度".into());
    }

    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct GetElectricityReq {
        pub refresh: u8,
    }
    let GetElectricityReq { refresh } = req.extract().await?;
    let res =
        service::electricity::get_electricity(&stu_id, refresh != 0)
            .await?;
    Ok(res.into())
}

#[handler]
async fn get_dormitory(req: &mut Request) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    if stu_id == DEMO_STU_ID {
        return Ok(GetDormitoryRes {
            room: "375".to_string(),
            build: "三区28栋".to_string(),
            park: "天马园区".to_string(),
        }
        .into());
    }

    #[derive(Serialize, Debug)]
    struct GetDormitoryRes {
        pub park: String,
        pub build: String,
        pub room: String,
    }
    let Some(dormitory) =
        service::user_info::get_person_info(&stu_id, false)
            .await?
            .dormitory
    else {
        return Ok(Value::Null.into());
    };
    let (Some(park), Some(build)) =
        (dormitory.park(), dormitory.build())
    else {
        return Ok(Value::Null.into());
    };
    let room = dormitory.room();
    Ok(GetDormitoryRes {
        park: park.to_string(),
        build: build.to_string(),
        room: room.to_string(),
    }
    .into())
}

#[handler]
async fn update_dormitory(req: &mut Request) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    // 直接更新个人信息的缓存就好了
    service::user_info::get_person_info(&stu_id, true).await?;
    Ok("更新成功".into())
}
