use crate::{
    error::{AppError, RouterResult},
    routers::{
        ThrowParseError,
        demo::{DEMO_COURSE_ID, DEMO_COURSE_NAME, DEMO_STU_ID},
    },
    service::{
        self,
        grade_rank::{
            GradeInfo, HdjwRankDataSource, HdjwRankDisplay,
            HdjwRankRange,
        },
    },
    utils::{self, serde::empty_string_as_none},
};
use salvo::{Request, Router, handler, macros::Extractible};
use serde::Deserialize;

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
    let stu_id = utils::jwt::auth(req)?;
    if stu_id == DEMO_STU_ID {
        return Ok(vec![GradeInfo {
            course_id: DEMO_COURSE_ID.to_string(),
            course_name: DEMO_COURSE_NAME.to_string(),
            credit: 2.0,
            course_type1: Some("必修".to_string()),
            course_type2: "通识必修".to_string(),
            gpa: 1.5,
            score: 81.0,
            tags: vec![],
            jx0404id: None,
        }]
        .into());
    }

    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct GetGradeReq {
        pub xn: u16,
        pub xq: u8,
    }
    let GetGradeReq { xn, xq } = req.extract().await.parse_error()?;
    let res = service::grade_rank::get_grade(xn, xq, &stu_id).await?;
    Ok(res.into())
}

#[handler]
async fn get_rank_from_hdjw(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct Request {
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub xn: Option<u16>,
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub xq: Option<u8>,
        pub range: u8,
        pub data_source: u8,
        pub display: u8,
    }
    let query: Request = req.extract().await.parse_error()?;
    let stu_id = utils::jwt::auth(req)?;

    let range = match query.range {
        1 => HdjwRankRange::Major,
        2 => HdjwRankRange::Minor,
        _ => return Err(AppError::parse_error()),
    };
    let data_source = match query.data_source {
        1 => HdjwRankDataSource::Total,
        2 => HdjwRankDataSource::Execution,
        _ => return Err(AppError::parse_error()),
    };
    let display = match query.display {
        1 => HdjwRankDisplay::Max,
        2 => HdjwRankDisplay::Initial,
        _ => return Err(AppError::parse_error()),
    };

    let res = service::grade_rank::get_rank_from_hdjw(
        &stu_id,
        query.xn,
        query.xq,
        range,
        data_source,
        display,
    )
    .await?;
    Ok(res.into())
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
    let GetGradeDetailReq { jx0404id } =
        req.extract().await.parse_error()?;
    let res =
        service::grade_rank::get_grade_detail(&stu_id, &jx0404id)
            .await?;
    Ok(res.into())
}
