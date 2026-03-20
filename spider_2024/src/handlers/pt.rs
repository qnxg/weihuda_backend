#![allow(deprecated)] // 用于忽略deprecated的警告
use anyhow::anyhow;
use salvo::{Request, Response, handler, writing::Json};
use serde_json::json;

use crate::{
    app_result::HandlerResult,
    dtos::pt::CardHistoryReq,
    spiders::{self, pt::CasPasswordStatus},
};

#[handler]
pub async fn check_password_handler(
    req: &mut Request,
    res: &mut Response,
) {
    let stuid: String = req.form("stuid").await.unwrap();
    let password: String = req.form("ptpass").await.unwrap();
    let result =
        spiders::pt::check_password_with_cas(&stuid, &password).await;
    match result {
        Ok(CasPasswordStatus::Success) => {
            res.render(Json(json!({"code": 0, "status": "success", "message": "密码正确"})));
        }
        Ok(CasPasswordStatus::Fail) => {
            res.render(Json(json!({"code": 1, "status": "error", "message": "密码错误"})));
        }
        Ok(CasPasswordStatus::ShouldChange) => {
            res.render(Json(json!({"code": 1, "status": "error", "message": "请前往个人门户修改密码后重试"})));
        }
        Ok(CasPasswordStatus::Lock) => {
            res.render(Json(json!({"code": 1, "status": "error", "message": "账号被锁定，请10分钟之后再试"})));
        }
        Err(e) => {
            res.render(Json(json!({"code": 1, "status": "error", "message": format!("服务器错误: {}", e)})));
        }
    }
}

#[handler]
pub async fn get_unread_email_handler(
    req: &mut Request,
) -> HandlerResult {
    let stuid = req
        .query::<String>("stuid")
        .ok_or(anyhow!("stuid is required"))?;
    let res = spiders::pt::get_unread_email(&stuid).await?;
    let res = &res["data"];
    Ok(res.into())
}

#[handler]
pub async fn get_card_info_handler(
    req: &mut Request,
) -> HandlerResult {
    let stuid = req
        .query::<String>("stuid")
        .ok_or(anyhow!("stuid is required"))?;
    let res = spiders::pt::get_card_info(&stuid).await?;
    let res = &res["data"];
    Ok(res.into())
}

/// 返回消费/充值记录，需要注意消费总额默认是负数
#[handler]
pub async fn get_card_history_handler(
    req: &mut Request,
) -> HandlerResult {
    let req: CardHistoryReq = req.parse_queries()?;
    let res = spiders::pt::get_card_history(
        &req.stuid, &req.year, &req.month, &req._type,
    )
    .await?;
    if res["data"].is_null() || res["data"]["amt"].is_null() {
        return Err(anyhow!("数据获取失败").into());
    }
    // 把原始数据传给前端，不处理正负，由前端负责决定要不要取反
    let total = res["data"]["amt"].as_f64().unwrap() / 100.0; //总额
    let tran_count = res["data"]["count"].as_number().unwrap(); //交易数量
    let items = &res["data"]["webTrjnDTO"]; //交易项列表
    let null = &json!([]);
    let res = json!(
        {
            "total": total,
            "TranCount": tran_count,
            "items": if items.is_null() { null } else { items }
        }
    );
    Ok(res.into())
}

#[deprecated(
    note = "这个pt个人门户的个人信息接口不可以使用，要用hdjw的个人信息接口"
)] // 警告下防止误用
#[handler]
pub async fn get_user_info_handler(
    req: &mut Request,
) -> HandlerResult {
    let stuid = req
        .query::<String>("stuid")
        .ok_or(anyhow!("stuid is required"))?;
    let res = spiders::pt::get_user_info(&stuid).await?;
    Ok(res.into())
}
