use crate::{
    result::{AppError, RouterResult, throw_error},
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
        .push(Router::with_path("bind").post(bind_user)) // 绑定用户
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
        pub code: String,
        pub stu_id: String,
        pub password: String,
    }
    let BindUserReq {
        code,
        stu_id,
        password,
    } = req.extract().await?;
    let stu_id = utils::format_stuid(&stu_id);
    let openid = service::auth::user::get_openid(&code).await?;

    let mut cas_token =
        hnu_query::cas::login::CasToken::new(&stu_id, &password);

    match hnu_query::pt::login::PtToken::acquire_by_cas_login(
        &mut cas_token,
    )
    .await
    {
        Ok(_) => {}
        Err(hnu_query::Error::Other(AccountIssue::PasswordError)) => {
            return Err(AppError::PasswordError);
        }
        Err(hnu_query::Error::Other(
            AccountIssue::PasswordShouldChange,
        )) => {
            return Err(AppError::Text(
                "请前往个人门户修改密码后重试".to_string(),
            ));
        }
        Err(hnu_query::Error::Other(AccountIssue::AccountLocked)) => {
            return Err(AppError::Text(
                "账号被锁定，请10分钟之后再试".to_string(),
            ));
        }
        // 需要双因子认证的话，反而说明密码验证通过了
        Err(hnu_query::Error::Other(AccountIssue::TFARequired(
            _,
        ))) => {}
        Err(e) => {
            return Err(throw_error(e, "验证密码失败"));
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
    let stu_id = utils::jwt::auth(req)?;
    if let Some(user) =
        service::auth::user::check_by_stu_id(&stu_id).await?
        && let Some(openid) = user.openid
    {
        service::auth::user::clear_openid(&openid).await?;
    }
    Ok("解绑成功".into())
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
    let GetAuthReq { code } = req.extract().await?;
    let openid = service::auth::user::get_openid(&code).await?;
    if let Some(user) =
        service::auth::user::check_by_openid(&openid).await?
    {
        let token = utils::jwt::generate_jwt(&user.stu_id)?;
        Ok(token.into())
    } else {
        Err("该微信号未绑定".into())
    }
}

/// 生成一个用于扫码登录的二维码
/// 主要是工作台在使用
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
    let GetAuthQrCodeStatusReq { code } = req.extract().await?;
    if let Some(status) =
        service::auth::qrcode::get_auth_qrcode_status(&code).await?
    {
        Ok(status.into())
    } else {
        Err("找不到二维码".into())
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
        req.extract().await?;

    let stu_id = utils::jwt::auth(req)?;

    if !["using", "confirmed", "canceled"].contains(&status.as_str())
    {
        return Err(AppError::ParseError);
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
                Err("已在其他设备上扫描".into())
            }
        } else {
            Ok(old_status.into())
        }
    } else {
        Err("未找到二维码".into())
    }
}

#[handler]
async fn get_auth_qrcode_info(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "param")))]
    struct GetAuthQrCodeInfoReq {
        pub code: String,
    }
    let GetAuthQrCodeInfoReq { code } = req.extract().await?;

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
            Err("二维码未被使用".into())
        }
    } else {
        Err("未找到二维码".into())
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
        return Err("未找到双因子认证信息(NO_TOAST)".into());
    };
    match tfa_token.send_sms().await {
        Ok(SMSResult::Success) => Ok("发送成功".into()),
        Ok(SMSResult::Valid) => Ok("之前发送的验证码仍有效".into()),
        Ok(SMSResult::Other(e)) => {
            tracing::error!(e = ?e, tfa_token = ?tfa_token, "发送双因子认证短信时遇到未知错误");
            Err(AppError::Text("发送失败，遇到未知错误".to_string()))
        }
        Err(e) => Err(throw_error(e, "发送双因子认证短信失败")),
    }
}

#[handler]
async fn tfa_verify(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "body")))]
    struct TFAVerifyReq {
        pub code: String,
    }
    let TFAVerifyReq { code } = req.extract().await?;
    let stu_id = utils::jwt::auth(req)?;
    let Some(tfa_token) =
        service::user_state::tfa::TFA_TOKEN.remove(&stu_id).await
    else {
        return Err("未找到双因子认证信息(NO_TOAST)".into());
    };
    match tfa_token.verify(&code).await {
        Ok(VerifyResult::Expired) => {
            Err("双因子认证已过期(NO_TOAST)".into())
        }
        Ok(VerifyResult::Success(cas_token)) => {
            service::user_state::tfa::apply_verified_cas_token(
                &stu_id, &cas_token,
            )
            .await;
            Ok("验证通过".into())
        }
        Ok(VerifyResult::CodeError(tfa_token)) => {
            service::user_state::tfa::TFA_TOKEN
                .insert(stu_id.to_string(), tfa_token.clone())
                .await;
            Err("验证码错误".into())
        }
        Err(e) => Err(throw_error(e, "验证双因子验证码失败")),
    }
}
