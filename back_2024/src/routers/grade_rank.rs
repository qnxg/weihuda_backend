use crate::service::grade_rank::{HdjwRankMethod, HdjwRankRange};
use crate::utils::serde::empty_string_as_none;
use crate::{
    result::{AppError, RouterResult},
    service::{self},
    utils,
};
use salvo::{Request, Router, handler, macros::Extractible};
use serde::{Deserialize, Serialize};

pub fn routers() -> Router {
    Router::with_path("hdjw")
        .push(
            Router::with_path("grade").get(get_grade).push(
                Router::with_path("detail").get(get_grade_detail),
            ),
        )
        .push(Router::with_path("grade-rank").get(get_rank_from_hdjw))
        .push(
            Router::with_path("grade-rank-from-ca")
                .get(get_rank_from_ca)
                .push(
                    salvo::Router::with_path("refresh")
                        .get(refresh_ca_rank),
                ),
        )
}

/// 获取成绩
#[handler]
async fn get_grade(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct GetGradeReq {
        pub xn: u16,
        pub xq: u8,
    }
    let GetGradeReq { xn, xq } = req.extract().await?;
    let stu_id = utils::jwt::auth(req)?;
    let res = service::grade_rank::get_grade(xn, xq, &stu_id).await?;
    Ok(res.into())
}

#[handler]
async fn get_rank_from_hdjw(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct GetRankFromHdjwReq {
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub year: Option<u16>,
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub term: Option<u8>,
        pub course: u32,
        pub rank: u32,
    }
    #[derive(Serialize, Debug)]
    struct GetRankFromHdjwRes {
        pub rank: String,
        pub score: String,
    }
    let query: GetRankFromHdjwReq = req.extract().await?;
    let stu_id = utils::jwt::auth(req)?;

    let range = match query.course {
        1 => HdjwRankRange::All,
        2 => HdjwRankRange::Must,
        3 => HdjwRankRange::Core,
        _ => return Err(AppError::ParseError),
    };
    let method = match query.rank {
        1 => HdjwRankMethod::ArithmeticAvg,
        2 => HdjwRankMethod::WeightedAvg,
        3 => HdjwRankMethod::Gpa,
        _ => return Err(AppError::ParseError),
    };
    let res = service::grade_rank::get_rank_from_hdjw(
        &stu_id, range, method, query.year, query.term,
    )
    .await?;
    Ok(GetRankFromHdjwRes {
        rank: res
            .clone()
            .and_then(|v| v.rank)
            .unwrap_or("无数据".to_string()),
        score: res
            .and_then(|v| v.score)
            .unwrap_or("无数据".to_string()),
    }
    .into())
}

#[handler]
async fn get_rank_from_ca(req: &mut Request) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    let res = service::grade_rank::ca::get_ca_rank(&stu_id).await?;
    Ok(res.into())
}

#[handler]
async fn refresh_ca_rank(req: &mut Request) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    service::grade_rank::ca::refresh_ca_rank(&stu_id).await?;
    Ok(().into())
}

#[handler]
async fn get_grade_detail(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct GetGradeDetailReq {
        pub jx0404id: String,
    }
    let stu_id = utils::jwt::auth(req)?;
    let GetGradeDetailReq { jx0404id } = req.extract().await?;
    let res =
        service::grade_rank::get_grade_detail(&stu_id, &jx0404id)
            .await?;
    Ok(res.into())
}
