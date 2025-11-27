use salvo::{Request, Router, handler};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    result::{AppError, RouterResult},
    service, utils,
};

pub fn routers() -> Router {
    Router::with_path("lab")
        .push(Router::with_path("list").get(get_lab_arrange))
        .push(Router::with_path("setPassword").post(set_lab_password))
        .push(Router::with_path("sem_info").get(get_lab_sem_info))
        .push(Router::with_path("grade").get(get_lab_grade))
        .push(
            Router::with_path("virtual_grade")
                .get(get_virtual_lab_grade),
        )
}

#[derive(Serialize, Deserialize, Debug)]
struct SetLabPasswordReq {
    password: String,
}
#[derive(Serialize)]
struct SetLabPasswordRes {
    pub success: bool,
    pub msg: Option<String>,
}
#[handler]
async fn set_lab_password(req: &mut Request) -> RouterResult {
    let SetLabPasswordReq { password } = req.parse_json().await?;
    let (_, stu_id) = utils::jwt::auth(req)?;
    if let Some(err) =
        service::lab::check_lab_pass(&stu_id, &password).await?
    {
        Ok(SetLabPasswordRes {
            success: false,
            msg: Some(err),
        }
        .into())
    } else {
        service::lab::set_lab_pass(&stu_id, &password).await?;
        Ok(SetLabPasswordRes {
            success: true,
            msg: None,
        }
        .into())
    }
}

#[handler]
async fn get_lab_arrange(req: &mut Request) -> RouterResult {
    let (_, stu_id) = utils::jwt::auth(req)?;
    match service::lab::get_lab_arrange(&stu_id).await {
        Ok(res) => Ok(res.into()),
        Err(AppError::PasswordError) => Ok(Value::Null.into()),
        Err(e) => Err(e),
    }
}

#[handler]
async fn get_lab_sem_info(req: &mut Request) -> RouterResult {
    let (_, stu_id) = utils::jwt::auth(req)?;
    match service::lab::get_sem_info(&stu_id).await {
        Ok(res) => Ok(res.into()),
        Err(AppError::PasswordError) => Ok(Value::Null.into()),
        Err(e) => Err(e),
    }
}

#[derive(Deserialize, Debug)]
struct GetLabGradeReq {
    sem_id: String,
}
#[handler]
async fn get_lab_grade(req: &mut Request) -> RouterResult {
    let GetLabGradeReq { sem_id } = req.parse_queries()?;
    let (_, stu_id) = utils::jwt::auth(req)?;
    match service::lab::get_course(&stu_id, &sem_id).await {
        Ok(res) => Ok(res.into()),
        Err(AppError::PasswordError) => Ok(Value::Null.into()),
        Err(e) => Err(e),
    }
}

#[handler]
async fn get_virtual_lab_grade(req: &mut Request) -> RouterResult {
    let (_, stu_id) = utils::jwt::auth(req)?;
    match service::lab::get_virtual_lab_grade(&stu_id).await {
        Ok(res) => Ok(res.into()),
        Err(AppError::PasswordError) => Ok(Value::Null.into()),
        Err(e) => Err(e),
    }
}
