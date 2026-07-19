use crate::{
    error::RouterResult,
    routers::ThrowParseError,
    service::{self, lab::CheckPasswordResult},
    utils,
};
use salvo::{Request, Router, handler, macros::Extractible};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

#[handler]
async fn set_lab_password(req: &mut Request) -> RouterResult {
    #[derive(Serialize, Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "body")))]
    struct SetLabPasswordReq {
        password: String,
    }
    #[derive(Serialize)]
    struct SetLabPasswordRes {
        pub success: bool,
        pub msg: Option<String>,
    }
    let SetLabPasswordReq { password } =
        req.extract().await.parse_error()?;
    let stu_id = utils::jwt::auth(req)?;
    let res =
        service::lab::check_password(&stu_id, &password).await?;
    match res {
        CheckPasswordResult::Success => {
            service::lab::set_lab_pass(&stu_id, &password).await?;
            Ok(SetLabPasswordRes {
                success: true,
                msg: None,
            }
            .into())
        }
        CheckPasswordResult::PasswordError => Ok(SetLabPasswordRes {
            success: false,
            msg: Some("密码错误".to_string()),
        }
        .into()),
        CheckPasswordResult::OtherError(msg) => {
            Ok(SetLabPasswordRes {
                success: false,
                msg: Some(msg.unwrap_or("未知错误".to_string())),
            }
            .into())
        }
    }
}

#[handler]
async fn get_lab_arrange(req: &mut Request) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    match service::lab::get_lab_arrange(&stu_id).await {
        Ok(res) => Ok(res.into()),
        Err(e) => {
            if e.is_password_error() {
                Ok(Value::Null.into())
            } else {
                Err(e)
            }
        }
    }
}

#[handler]
async fn get_lab_sem_info(req: &mut Request) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    match service::lab::get_sem_info(&stu_id).await {
        Ok(res) => Ok(res.into()),
        Err(e) => {
            if e.is_password_error() {
                Ok(Value::Null.into())
            } else {
                Err(e)
            }
        }
    }
}

#[handler]
async fn get_lab_grade(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct GetLabGradeReq {
        sem_id: String,
    }
    let GetLabGradeReq { sem_id } =
        req.extract().await.parse_error()?;
    let stu_id = utils::jwt::auth(req)?;
    match service::lab::get_course(&stu_id, &sem_id).await {
        Ok(res) => Ok(res.into()),
        Err(e) => {
            if e.is_password_error() {
                Ok(Value::Null.into())
            } else {
                Err(e)
            }
        }
    }
}

#[handler]
async fn get_virtual_lab_grade(req: &mut Request) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    match service::lab::get_virtual_lab_grade(&stu_id).await {
        Ok(res) => Ok(res.into()),
        Err(e) => {
            if e.is_password_error() {
                Ok(Value::Null.into())
            } else {
                Err(e)
            }
        }
    }
}
