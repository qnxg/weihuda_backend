use crate::result::{AppError, RouterResult};
use crate::service;
use crate::service::feedback::FeedbackInfo;
use crate::utils::serde::empty_string_as_none;
use salvo::{Request, Router, handler};
use serde::{Deserialize, Serialize};

pub fn routers() -> Router {
    Router::with_path("feedback")
        .post(add_feedback)
        .put(update_feedback)
        .push(
            Router::with_path("no-auth")
                .get(get_feedback)
                .post(add_feedback),
        )
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
struct GetFeedbackReq {
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub page: Option<u32>,
    pub stuId: String,
}
#[derive(Serialize, Debug)]
struct GetFeedbackRes {
    pub count: u32,
    pub rows: Vec<FeedbackInfo>,
}
#[handler]
async fn get_feedback(req: &mut Request) -> RouterResult {
    let GetFeedbackReq { page, stuId } = req.parse_queries()?;
    if stuId.len() != 12 {
        return Err(AppError::ParseError());
    }
    let res = service::feedback::get_feedback_list(
        &stuId,
        10,
        page.unwrap_or(1),
    )
    .await?;
    Ok(GetFeedbackRes {
        count: res.len() as u32,
        rows: res,
    }
    .into())
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
struct AddFeedbackReq {
    pub stuId: String,
    pub desc: String,
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub contact: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub imgUrl: Option<String>,
    #[serde(rename = "type")]
    pub _type: String,
}
#[handler]
async fn add_feedback(req: &mut Request) -> RouterResult {
    let feedback: AddFeedbackReq = req.parse_json().await?;
    service::feedback::add_feedback(
        &feedback._type,
        &feedback.desc,
        feedback.contact.as_ref(),
        feedback.imgUrl.as_ref(),
        &feedback.stuId,
    )
    .await?;
    Ok("添加反馈成功".into())
}

#[derive(Deserialize, Debug)]
struct UpdateFeedbackReq {
    pub id: u32,
    pub status: i8,
}
#[handler]
async fn update_feedback(req: &mut Request) -> RouterResult {
    let UpdateFeedbackReq { id, status } = req.parse_json().await?;
    service::feedback::update_feedback_status(id, status).await?;
    Ok("更新反馈状态成功".into())
}
