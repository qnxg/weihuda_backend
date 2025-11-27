use crate::utils::serde::empty_string_as_none;
use crate::{
    result::RouterResult,
    service::{self, exam::ExamNumberInfo},
    utils,
};
use salvo::{Request, Router, handler};
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
    let (mini_bind_id, _) = utils::jwt::auth(req)?;
    let res = service::exam::get_exam_num_list(mini_bind_id).await?;
    Ok(res.into())
}

#[derive(Deserialize, Debug)]
struct AddExamNumberReq {
    pub num: String,
    pub name: String,
    pub date: String,
}
#[handler]
async fn add_exam_num(req: &mut Request) -> RouterResult {
    let exam_num: AddExamNumberReq = req.parse_json().await?;
    let (mini_bind_id, _) = utils::jwt::auth(req)?;

    service::exam::add_exam_num(
        mini_bind_id,
        ExamNumberInfo {
            exam_num: exam_num.num,
            exam_name: exam_num.name,
            exam_date: exam_num.date,
            id: 0,
        },
    )
    .await?;

    Ok("添加成功".into())
}

#[derive(Deserialize, Debug)]
struct DeleteExamNumberReq {
    pub id: u32,
}
#[handler]
async fn delete_exam_num(req: &mut Request) -> RouterResult {
    let delete_req: DeleteExamNumberReq = req.parse_queries()?;
    let (mini_bind_id, _) = utils::jwt::auth(req)?;

    service::exam::delete_exam_num(mini_bind_id, delete_req.id)
        .await?;

    Ok("删除成功".into())
}

#[derive(Deserialize, Debug)]
struct GetExamArrangeReq {
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub xn: Option<u32>,
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub xq: Option<u32>,
}
#[handler]
async fn get_exam_arrange(req: &mut Request) -> RouterResult {
    let (_, stu_id) = utils::jwt::auth(req)?;
    let GetExamArrangeReq { xn, xq } = req.parse_queries()?;
    let (current_xn, current_xq) =
        service::semester::get_now_xnxq().await?;
    let xn = xn.unwrap_or(current_xn);
    let xq = xq.unwrap_or(current_xq);
    let res =
        service::exam::get_exam_arrange(&stu_id, xn, xq).await?;
    Ok(res.into())
}
