use crate::{
    result::{AppError, RouterResult},
    service::{self, auth::qrcode::AuthQrCodeStatus},
    utils,
};
use salvo::{Request, Router, handler, macros::Extractible};
use serde::Deserialize;
use serde_json::json;
use tokio::try_join;

pub fn routers() -> Router {
    Router::new()
        .push(Router::with_path("token").get(get_auth)) // 用jscode换取token
        .push(Router::with_path("bind").post(bind_user)) // 绑定用户
        .push(Router::with_path("flutter").post(flutter_auth)) // Tomzz 的 Flutter 端使用的绑定接口
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
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
struct BindUserReq {
    pub code: String,
    pub stuId: String,
    pub stuPassword: String,
    pub hdjwPassword: String,
    // pub mode: u32,
    // pub platform: u32,
}
#[handler]
async fn bind_user(req: &mut Request) -> RouterResult {
    let BindUserReq {
        code,
        stuId,
        stuPassword,
        hdjwPassword,
    } = req.parse_json().await?;

    let (verify_res, openid) = try_join!(
        service::auth::user::verify_password(
            &stuId,
            &hdjwPassword,
            &stuPassword
        ),
        service::auth::user::get_openid(&code)
    )?;

    match verify_res.code {
        0 => {} // 验证成功
        1 => return Err("个人门户密码错误".into()),
        // 2 => return Err("教务系统密码错误".into()),
        // 3 => return Err("个人门户和教务系统密码均错误".into()),
        _ => return Err("密码验证服务返回值错误".into()),
    }

    if service::auth::user::check_by_openid(&openid)
        .await?
        .is_some()
    {
        return Err("该微信号已绑定".into());
    }

    service::auth::user::bind(
        &openid,
        &stuId,
        &hdjwPassword,
        &stuPassword,
    )
    .await?;

    Ok("绑定成功".into())
}

#[handler]
async fn unbind_user(req: &mut Request) -> RouterResult {
    let (mini_bind_id, _) = utils::jwt::auth(req)?;
    service::auth::user::unbind(mini_bind_id).await?;
    Ok("解绑成功".into())
}

#[derive(Deserialize, Debug)]
struct GetAuthReq {
    pub code: String,
}
/// 根据微信提供的 jscode 下发 jwt
/// 这个的前提是已经绑定过了
#[handler]
async fn get_auth(req: &mut Request) -> RouterResult {
    let GetAuthReq { code } = req.parse_queries()?;
    let openid = service::auth::user::get_openid(&code).await?;
    if let Some(user) =
        service::auth::user::check_by_openid(&openid).await?
    {
        let Some(stu_id) = user.stuID else {
            return Err("找不到学号".into());
        };
        let token = utils::jwt::generate_jwt(user.id, &stu_id)?;
        Ok(token.into())
    } else {
        Err("该微信号未绑定".into())
    }
}

#[derive(Deserialize, Debug)]
struct FlutterAuthReq {
    pub stu_id: String,
    pub stu_pwd: String,
}
#[handler]
async fn flutter_auth(req: &mut Request) -> RouterResult {
    let FlutterAuthReq { stu_id, stu_pwd } = req.parse_json().await?;
    let verify_res = service::auth::user::verify_password(
        &stu_id, &stu_pwd, &stu_pwd,
    )
    .await?;
    match verify_res.code {
        0 => {} // 验证成功
        1 => return Err("个人门户密码错误".into()),
        _ => return Err("密码验证服务返回值错误".into()),
    }
    if let Some(user) =
        service::auth::user::check_by_stu_id(&stu_id).await?
    {
        if let Some(openid) = user.openid {
            if openid.is_empty() {
                return Err("需要先登录一次微生活小程序".into());
            }
        } else {
            return Err("数据库中没有账号信息".into()); // 应该不会出现这种情况，在check_by_stu_id时就会返回 None
        }
        let token = utils::jwt::generate_jwt(user.id, &stu_id)?;
        Ok(token.into())
    } else {
        Err("数据库中没有账号信息".into())
    }
}

/// 生成一个用于扫码登录的二维码
/// 主要是工作台在使用
#[handler]
async fn get_auth_qrcode() -> RouterResult {
    let code = service::auth::qrcode::generate_auth_qrcode().await?;
    Ok(code.into())
}

#[derive(Deserialize, Debug)]
struct GetAuthQrCodeStatusReq {
    pub code: String,
}
#[handler]
async fn get_auth_qrcode_status(req: &mut Request) -> RouterResult {
    let GetAuthQrCodeStatusReq { code } = req.parse_params()?;
    if let Some(status) =
        service::auth::qrcode::get_auth_qrcode_status(&code).await?
    {
        Ok(status.into())
    } else {
        Err("找不到二维码".into())
    }
}

#[derive(Deserialize, Debug, Extractible)]
struct PutAuthQrCodeStatusReq {
    #[salvo(extract(source(from = "param")))]
    pub code: String,
    #[salvo(extract(source(from = "body")))]
    pub status: String,
}
#[handler]
async fn put_auth_qrcode_status(req: &mut Request) -> RouterResult {
    let PutAuthQrCodeStatusReq { code, status } =
        req.extract().await?;

    let (_, stu_id) = utils::jwt::auth(req)?;

    if !["using", "confirmed", "canceled"].contains(&status.as_str())
    {
        return Err(AppError::ParseError());
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

#[derive(Deserialize, Debug)]
struct GetAuthQrCodeInfoReq {
    pub code: String,
}
#[handler]
async fn get_auth_qrcode_info(req: &mut Request) -> RouterResult {
    let GetAuthQrCodeInfoReq { code } = req.parse_params()?;

    if let Some(status) =
        service::auth::qrcode::get_auth_qrcode_status(&code).await?
    {
        if let AuthQrCodeStatus::Used = status {
            // 已经确定 qrcode 存在了，所以可以直接 expect
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
