use crate::{
    error::RouterResult,
    routers::{
        ThrowParseError,
        demo::{DEMO_COURSE_ID, DEMO_COURSE_NAME, DEMO_STU_ID},
    },
    service::{
        self,
        exam::{ExamArrange, ExamNumberInfo},
    },
    utils::{self, serde::empty_string_as_none},
};
use salvo::{Request, Router, handler, macros::Extractible};
use serde::Deserialize;

pub fn routers() -> Router {
    Router::new()
        .push(
            Router::with_path("exam-num") // 考号预存
                .get(get_exam_num)
                .post(add_exam_num)
                .delete(delete_exam_num),
        )
        .push(
            Router::with_path("hdjw/exam-arrange")
                .get(get_exam_arrange),
        )
}

#[handler]
async fn get_exam_num(req: &mut Request) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    let res = service::exam::get_exam_num_list(&stu_id).await?;
    Ok(res.into())
}

#[handler]
async fn add_exam_num(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(
        default_source(from = "body"),
        rename_all = "camelCase"
    ))]
    struct AddExamNumberReq {
        pub exam_num: String,
        pub exam_name: String,
        pub exam_date: String,
    }
    let AddExamNumberReq {
        exam_num,
        exam_name,
        exam_date,
    } = req.extract().await.parse_error()?;
    let stu_id = utils::jwt::auth(req)?;

    service::exam::add_exam_num(
        &stu_id,
        ExamNumberInfo {
            exam_num,
            exam_name,
            exam_date,
            id: 0,
        },
    )
    .await?;

    Ok("添加成功".into())
}

#[handler]
async fn delete_exam_num(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct DeleteExamNumberReq {
        pub id: u32,
    }
    let DeleteExamNumberReq { id } =
        req.extract().await.parse_error()?;
    let stu_id = utils::jwt::auth(req)?;
    service::exam::delete_exam_num(&stu_id, id).await?;
    Ok("删除成功".into())
}

#[handler]
async fn get_exam_arrange(req: &mut Request) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    if stu_id == DEMO_STU_ID {
        return Ok(vec![ExamArrange {
            id: DEMO_COURSE_ID.to_string(),
            name: DEMO_COURSE_NAME.to_string(),
            place: "综合楼601".to_string(),
            date: "2026-05-04".to_string(),
            time: "14:30~16:00".to_string(),
            seat: "42号".to_string(),
        }]
        .into());
    }

    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct GetExamArrangeReq {
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub xn: Option<u32>,
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub xq: Option<u32>,
    }
    let GetExamArrangeReq { xn, xq } =
        req.extract().await.parse_error()?;
    let (current_xn, current_xq) =
        service::semester::get_now_xnxq().await?;
    let xn = xn.unwrap_or(current_xn) as u16;
    let xq = xq.unwrap_or(current_xq) as u8;
    let res =
        service::exam::get_exam_arrange(&stu_id, xn, xq).await?;
    Ok(res.into())
}
