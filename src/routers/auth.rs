use crate::{
    config::CFG,
    error::{
        AppError, RouterResult, ThrowInternalError,
        ThrowInternalErrorMsg, ThrowInternalErrorResult,
    },
    routers::{
        ThrowParseError,
        demo::{DEMO_PASSWORD, DEMO_STU_ID},
    },
    service::{self, auth::qrcode::AuthQrCodeStatus},
    utils,
};
use hnu_query::cas::{
    login::AccountIssue,
    tfa::{SMSResult, VerifyResult},
};
use salvo::{Request, Router, handler, macros::Extractible};
use serde::{Deserialize, Serialize};
use serde_json::json;

pub fn routers() -> Router {
    Router::new()
        .push(Router::with_path("token").get(get_auth)) // 用jscode换取token
        .push(
            Router::with_path("bind")
                .post(bind_user)
                .push(Router::with_path("pow").get(get_pow)),
        ) // 绑定用户
        .push(Router::with_path("unbind").post(unbind_user))
        .push(
            Router::with_path("auth-qrcode")
                .push(
                    Router::with_path("status/{code}")
                        .get(get_auth_qrcode_status)
                        .put(put_auth_qrcode_status),
                )
                .push(
                    Router::with_path("info/{code}")
                        .get(get_auth_qrcode_info),
                )
                .get(get_auth_qrcode),
        )
        .push(
            Router::with_path("tfa")
                .get(get_tfa)
                .push(Router::with_path("send_sms").get(tfa_send_sms))
                .push(Router::with_path("verify").post(tfa_verify)),
        )
}

#[handler]
async fn bind_user(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(
        default_source(from = "body"),
        rename_all = "camelCase"
    ))]
    struct BindUserReq {
        pub pow_ticket: String,
        pub pow_answer: usize,
        pub code: String,
        pub stu_id: String,
        pub password: String,
    }
    let BindUserReq {
        pow_ticket,
        pow_answer,
        code,
        stu_id,
        password,
    } = req.extract().await.parse_error()?;

    let Some(pow_stu_id) =
        service::auth::pow::verify_pow(&pow_ticket, pow_answer)
            .await?
    else {
        return Err(AppError::customized("pow 验证失败，请重试"));
    };

    let stu_id = utils::format_stuid(&stu_id);
    if pow_stu_id != stu_id {
        return Err(AppError::customized("pow 验证失败，请重试"));
    }

    let password = utils::crypto::decrypt_frontend(&password)?;
    let openid = service::auth::user::get_openid(&code).await?;

    // 演示账号特判
    if stu_id == DEMO_STU_ID && password != DEMO_PASSWORD {
        return Err(AppError::customized("密码错误"));
    }

    if stu_id != DEMO_STU_ID {
        match hnu_query::cas::login::CasToken::acquire_by_login(
            &stu_id, &password,
        )
        .await
        {
            Ok(_) => {}
            Err(hnu_query::Error::Other(
                AccountIssue::PasswordError,
            )) => {
                return Err(AppError::customized("密码错误"));
            }
            Err(hnu_query::Error::Other(
                AccountIssue::PasswordShouldChange,
            )) => {
                return Err(AppError::customized(
                    "请前往个人门户修改密码后重试",
                ));
            }
            Err(hnu_query::Error::Other(
                AccountIssue::AccountLocked,
            )) => {
                return Err(AppError::customized(
                    "账号被锁定，请10分钟之后再试",
                ));
            }
            // 需要双因子认证的话，反而说明密码验证通过了
            Err(hnu_query::Error::Other(
                AccountIssue::TFARequired(_),
            )) => {}
            Err(e) => {
                return Err(e.internal_err().into());
            }
        }
    }

    service::auth::user::clear_openid(&openid).await?;
    service::auth::user::bind(&stu_id, &openid, &password).await?;

    // 重置账号登录错误状态
    service::user_state::ACCOUNT_TAG.invalidate(&stu_id).await;

    Ok("绑定成功".into())
}

#[handler]
async fn unbind_user(req: &mut Request) -> RouterResult {
    let Ok(stu_id) = utils::jwt::auth(req) else {
        // TODO 这里应该要前端处理，但是目前的前端会在 unbind 返回 401 时死循环
        return Err(AppError::customized("未登录"));
    };
    if let Some(user) =
        service::auth::user::check_by_stu_id(&stu_id).await?
        && let Some(openid) = user.openid
    {
        service::auth::user::clear_openid(&openid).await?;
    }
    Ok("解绑成功".into())
}

#[handler]
async fn get_pow(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(
        default_source(from = "query"),
        rename_all = "camelCase"
    ))]
    struct GetPowReq {
        pub stu_id: String,
    }
    let GetPowReq { stu_id } = req.extract().await.parse_error()?;
    let pow = service::auth::pow::generate_pow(&stu_id).await?;
    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    struct GetPowRes {
        pub ticket: String,
        pub difficulty: usize,
    }
    let GetPowRes { ticket, difficulty } = GetPowRes {
        ticket: pow,
        difficulty: CFG.pow.difficulty as usize,
    };
    let res = GetPowRes { ticket, difficulty };
    Ok(res.into())
}

/// 根据微信提供的 jscode 下发 jwt
/// 这个的前提是已经绑定过了
#[handler]
async fn get_auth(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct GetAuthReq {
        pub code: String,
    }
    let GetAuthReq { code } = req.extract().await.parse_error()?;
    let openid = service::auth::user::get_openid(&code).await?;
    if let Some(user) =
        service::auth::user::check_by_openid(&openid).await?
    {
        let token = utils::jwt::generate_jwt(&user.stu_id)
            .map_err(|e| e.internal_err().with("生成 jwt 失败"))?;
        Ok(token.into())
    } else {
        Err(AppError::customized("该微信号未绑定"))
    }
}

/// 生成一个用于扫码登录的二维码
/// 主要是工作台在使用
#[handler]
async fn get_auth_qrcode() -> RouterResult {
    let code = service::auth::qrcode::generate_auth_qrcode().await?;
    Ok(code.into())
}

#[handler]
async fn get_auth_qrcode_status(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "param")))]
    struct GetAuthQrCodeStatusReq {
        pub code: String,
    }
    let GetAuthQrCodeStatusReq { code } =
        req.extract().await.parse_error()?;
    if let Some(status) =
        service::auth::qrcode::get_auth_qrcode_status(&code).await?
    {
        Ok(status.into())
    } else {
        Err(AppError::customized("找不到二维码"))
    }
}

#[handler]
async fn put_auth_qrcode_status(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    struct PutAuthQrCodeStatusReq {
        #[salvo(extract(source(from = "param")))]
        pub code: String,
        #[salvo(extract(source(from = "body")))]
        pub status: String,
    }
    let PutAuthQrCodeStatusReq { code, status } =
        req.extract().await.parse_error()?;

    let stu_id = utils::jwt::auth(req)?;

    if !["using", "confirmed", "canceled"].contains(&status.as_str())
    {
        return Err(AppError::parse_error());
    }

    if let Some(old_status) =
        service::auth::qrcode::get_auth_qrcode_status(&code).await?
    {
        // using 状态说明扫描了，我们暂时不去管他
        // canceled 状态说明用户取消扫描这个二维码，我们也暂时不管他
        // confirmed 状态说明用户确认使用了这个二维码
        if status == "confirmed" {
            if let AuthQrCodeStatus::Unused = old_status {
                service::auth::qrcode::confirm_auth_qrcode(
                    &code, &stu_id,
                )
                .await?;
                Ok(AuthQrCodeStatus::Used.into())
            } else {
                Err(AppError::customized("已在其他设备上扫描"))
            }
        } else {
            Ok(old_status.into())
        }
    } else {
        Err(AppError::customized("未找到二维码"))
    }
}

#[handler]
async fn get_auth_qrcode_info(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "param")))]
    struct GetAuthQrCodeInfoReq {
        pub code: String,
    }
    let GetAuthQrCodeInfoReq { code } =
        req.extract().await.parse_error()?;

    if let Some(status) =
        service::auth::qrcode::get_auth_qrcode_status(&code).await?
    {
        if let AuthQrCodeStatus::Used = status {
            // 已经确定 qrcode 存在了，所以可以直接 expect
            // 这里应该只有 stu_id 是有用的，这里这样返回是为了兼容旧接口
            let stu_id =
                service::auth::qrcode::consume_auth_qrcode(&code)
                    .await?
                    .expect("获取到 qrcode 的状态，但是尝试获取 stu_id 时又不存在");
            // 这里应该只有 stu_id 是有用的
            // 这里这样返回是为了兼容旧接口
            Ok(json!({
                "code": code,
                "status": "used",
                "info": {
                    "stu_id": stu_id,
                    "name": "新用户",
                },
                "create_time": chrono::Local::now().to_rfc3339(),
            })
            .into())
        } else {
            Err(AppError::customized("二维码未被使用"))
        }
    } else {
        Err(AppError::customized("未找到二维码"))
    }
}

#[handler]
async fn get_tfa(req: &mut Request) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    #[derive(Serialize, Debug)]
    struct GetTFARes {
        phone: String,
    }
    let Some(tfa_token) =
        service::user_state::tfa::TFA_TOKEN.get(&stu_id).await
    else {
        return Ok(serde_json::Value::Null.into());
    };
    Ok(GetTFARes {
        phone: tfa_token.phone().to_string(),
    }
    .into())
}

#[handler]
async fn tfa_send_sms(req: &mut Request) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    let Some(tfa_token) =
        service::user_state::tfa::TFA_TOKEN.get(&stu_id).await
    else {
        return Err(AppError::customized(
            "未找到双因子认证信息(NO_TOAST)",
        ));
    };
    match tfa_token.send_sms().await.internal_err()? {
        SMSResult::Success => Ok("发送成功".into()),
        SMSResult::Valid => Ok("之前发送的验证码仍有效".into()),
        SMSResult::Other(e) => {
            tracing::error!(e = ?e, tfa_token = ?tfa_token, "发送双因子认证短信时遇到未知错误");
            Err(e
                .internal_err()
                .with("发送双因子认证短信时遇到未知错误")
                .show("发送失败，遇到未知错误")
                .into())
        }
    }
}

#[handler]
async fn tfa_verify(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "body")))]
    struct TFAVerifyReq {
        pub code: String,
    }
    let TFAVerifyReq { code } = req.extract().await.parse_error()?;
    let stu_id = utils::jwt::auth(req)?;
    let Some(tfa_token) =
        service::user_state::tfa::TFA_TOKEN.remove(&stu_id).await
    else {
        return Err(AppError::customized(
            "未找到双因子认证信息(NO_TOAST)",
        ));
    };
    match tfa_token.verify(&code).await.internal_err()? {
        VerifyResult::Expired => {
            Err(AppError::customized("双因子认证已过期(NO_TOAST)"))
        }
        VerifyResult::Success(cas_token) => {
            service::user_state::tfa::apply_verified_cas_token(
                &stu_id, &cas_token,
            )
            .await?;
            Ok("验证通过".into())
        }
        VerifyResult::CodeError(tfa_token) => {
            service::user_state::tfa::TFA_TOKEN
                .insert(stu_id.to_string(), tfa_token.clone())
                .await;
            Err(AppError::customized("验证码错误"))
        }
    }
}
