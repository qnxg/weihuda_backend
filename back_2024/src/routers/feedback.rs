use crate::result::{AppError, RouterResult};
use crate::service::feedback::FeedbackInfo;
use crate::utils::serde::empty_string_as_none;
use crate::{service, utils};
use anyhow::anyhow;
use salvo::macros::Extractible;
use salvo::{Request, Router, handler};
use serde::{Deserialize, Serialize};

pub fn routers() -> Router {
    Router::with_path("feedback")
        .post(add_feedback)
        .get(get_feedback)
        // 搞这个是因为前端会在一些路由上故意不携带 token。token 可有可无的路由需要这么分
        .push(Router::with_path("no_auth").post(add_feedback))
        .push(Router::with_path("msg").get(get_feedback_msg))
}

#[handler]
async fn get_feedback(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct GetFeedbackReq {
        pub page: u32,
    }
    #[derive(Serialize, Debug)]
    struct GetFeedbackRes {
        pub count: u32,
        pub rows: Vec<FeedbackInfo>,
    }
    let stu_id = utils::jwt::auth(req)?;
    let GetFeedbackReq { page } = req.extract().await?;
    let res = service::feedback::get_feedback_list(&stu_id, 10, page)
        .await?;
    Ok(GetFeedbackRes {
        count: res.len() as u32,
        rows: res,
    }
    .into())
}

#[handler]
async fn add_feedback(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(
        default_source(from = "body"),
        rename_all = "camelCase"
    ))]
    struct AddFeedbackReq {
        pub stu_id: Option<String>,
        pub desc: String,
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub contact: Option<String>,
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub img_url: Option<String>,
    }
    let stu_id = utils::jwt::auth(req).ok();
    let feedback: AddFeedbackReq = req.extract().await?;
    // 如果已经登陆，那么 stu_id 不看传入的
    // 如果没有登录，传入的 stu_id 必选，但是插入的时候不插入 stu_id，并且必须提供联系方式
    if stu_id.is_none()
        && (feedback.stu_id.is_none() || feedback.contact.is_none())
    {
        return Err(AppError::ParseError());
    }
    let mut msg = feedback.desc.clone();
    if stu_id.is_none() {
        // 已经检查过了，可以 unwrap
        msg += &format!(
            "\n(提供学号：{})",
            feedback.stu_id.unwrap_or_default()
        );
    }
    service::feedback::add_feedback(
        &msg,
        feedback.contact.as_ref(),
        feedback.img_url.as_ref(),
        stu_id.as_deref(),
    )
    .await?;
    Ok("添加反馈成功".into())
}

#[handler]
async fn get_feedback_msg(req: &mut Request) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(
        default_source(from = "query"),
        rename_all = "camelCase"
    ))]
    struct GetFeedbackMsgReq {
        pub feedback_id: u32,
    }
    let GetFeedbackMsgReq { feedback_id } = req.extract().await?;
    let Some(feedback) =
        service::feedback::get_feedback(feedback_id).await?
    else {
        return Err(anyhow!("反馈不存在").into());
    };
    if feedback.stu_id != Some(stu_id) {
        return Err(anyhow!("反馈不存在").into());
    }
    let res =
        service::feedback::get_feedback_msg(feedback_id).await?;
    Ok(res.into())
}
