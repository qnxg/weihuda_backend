use crate::utils::serde::empty_string_as_none;
use crate::{
    result::RouterResult,
    service::{self, exam::ExamNumberInfo},
    utils,
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
    } = req.extract().await?;
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
    let DeleteExamNumberReq { id } = req.extract().await?;
    let stu_id = utils::jwt::auth(req)?;
    service::exam::delete_exam_num(&stu_id, id).await?;
    Ok("删除成功".into())
}

#[handler]
async fn get_exam_arrange(req: &mut Request) -> RouterResult {
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
    let GetExamArrangeReq { xn, xq } = req.extract().await?;
    let stu_id = utils::jwt::auth(req)?;
    let (current_xn, current_xq) =
        service::semester::get_now_xnxq().await?;
    let xn = xn.unwrap_or(current_xn);
    let xq = xq.unwrap_or(current_xq);
    let res =
        service::exam::get_exam_arrange(&stu_id, xn, xq).await?;
    Ok(res.into())
}
