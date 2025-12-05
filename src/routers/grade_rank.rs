use crate::utils::serde::empty_string_as_none;
use salvo::{Request, Router, handler};
use serde::Deserialize;

use crate::{
    result::{AppError, RouterResult},
    service::{
        self,
        grade_rank::{HdjwRankMethod, HdjwRankRange},
    },
    utils,
};

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
                .get(get_rank_from_ca),
        )
}

/// 获取成绩
#[derive(Deserialize, Debug)]
struct GetGradeReq {
    pub xn: u32,
    pub xq: u32,
}
#[handler]
async fn get_grade(req: &mut Request) -> RouterResult {
    let GetGradeReq { xn, xq } = req.parse_queries()?;
    let (_, stu_id) = utils::jwt::auth(req)?;
    let res = service::grade_rank::get_grade(xn, xq, &stu_id).await?;
    Ok(res.into())
}

#[derive(Deserialize, Debug)]
struct GetRankFromHdjwReq {
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub year: Option<u32>,
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub term: Option<u32>,
    pub course: u32,
    pub rank: u32,
}
#[handler]
async fn get_rank_from_hdjw(req: &mut Request) -> RouterResult {
    let GetRankFromHdjwReq {
        year,
        term,
        course,
        rank,
    } = req.parse_queries()?;
    let (_, stu_id) = utils::jwt::auth(req)?;

    let range = match course {
        1 => HdjwRankRange::All,
        2 => HdjwRankRange::Must,
        3 => HdjwRankRange::Core,
        _ => return Err(AppError::ParseError()),
    };
    let method = match rank {
        1 => HdjwRankMethod::ArithmeticAvg,
        2 => HdjwRankMethod::WeightedAvg,
        3 => HdjwRankMethod::Gpa,
        _ => return Err(AppError::ParseError()),
    };
    let res = service::grade_rank::get_rank_from_hdjw(
        &stu_id, range, method, year, term,
    )
    .await?;
    Ok(res.into())
}

#[handler]
async fn get_rank_from_ca(req: &mut Request) -> RouterResult {
    let (_, stu_id) = utils::jwt::auth(req)?;
    let res = service::grade_rank::get_rank_from_ca(&stu_id).await?;
    Ok(res.into())
}

#[derive(Deserialize, Debug)]
struct GetGradeDetailReq {
    pub jx0404id: String,
}
#[handler]
async fn get_grade_detail(req: &mut Request) -> RouterResult {
    let (_, stu_id) = utils::jwt::auth(req)?;
    let GetGradeDetailReq { jx0404id } = req.parse_queries()?;
    let res =
        service::grade_rank::get_grade_detail(&stu_id, &jx0404id)
            .await?;
    Ok(res.into())
}
