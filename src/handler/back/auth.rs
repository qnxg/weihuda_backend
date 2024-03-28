use crate::{
    app_result::AppResult,
    extract::{Json, Query},
    handler::back::common::check_user::check_by_code,
    schema::back::auth::AuthReq,
    utility::jwt::{auth, parse_stu_id},
    Pool,
};
use axum::{
    extract::{Path, State},
    Extension,
};
use chrono;
use lazy_static::lazy_static;
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub async fn get_auth_handler(
    State(data): State<Arc<Pool>>,
    Query(req): Query<AuthReq>,
) -> AppResult {
    let user = check_by_code(data, &req.code).await?;
    //TODO 这里的判断有必要吗？
    if user.stuID.is_none() {
        return Err("找不到学号".into());
        // return Err(crate::app_error::AppError::SqlxError(sqlx::Error::RowNotFound));
    }
    let token = auth(user.id, &user.stuID.unwrap())?;
    Ok(token.into())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AuthQrCode {
    code: String,
    status: AuthQrCodeStatus,
    info: Option<Info>,
    create_time: chrono::DateTime<chrono::Local>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum AuthQrCodeStatus {
    #[serde(rename = "unused")]
    Unused,
    #[serde(rename = "using")]
    Using,
    #[serde(rename = "used")]
    Used,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Info {
    stu_id: String,
    name: String,
}

#[derive(Deserialize, Debug)]
pub struct PostAuthQrCodeStatusReq {
    pub status: String,
}

lazy_static! {
    static ref AUTH_QRCODE_MAP: Arc<Mutex<HashMap<String, AuthQrCode>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

pub async fn get_auth_qrcode_handler() -> AppResult {
    let mut map = AUTH_QRCODE_MAP.lock().unwrap();
    let code: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();

    let now = chrono::Local::now();

    map.insert(
        code.clone(),
        AuthQrCode {
            code: code.clone(),
            status: AuthQrCodeStatus::Unused,
            info: None,
            create_time: now,
        },
    );

    // 检查历史二维码，如果超过10分钟则删除
    map.retain(|_, v| now.signed_duration_since(v.create_time).num_minutes() <= 10);
    Ok(code.into())
}

pub async fn get_auth_qrcode_status_handler(Path(code): Path<String>) -> AppResult {
    let map = AUTH_QRCODE_MAP.lock().unwrap();
    if map.contains_key(&code) {
        Ok(map.get(&code).unwrap().status.clone().into())
    } else {
        Err("找不到二维码".into())
    }
}

pub async fn put_auth_qrcode_status_handler(
    Path(code): Path<String>,
    Extension(token): Extension<String>,
    Json(data): Json<PostAuthQrCodeStatusReq>,
) -> AppResult {
    let mut map = AUTH_QRCODE_MAP.lock().unwrap();

    let stu_id = parse_stu_id(&token)?;

    // 参数校验
    if !["using", "confirmed", "canceled"].contains(&data.status.as_str()) {
        return Err("status参数不合法".into());
    }

    if let Some(qrcode) = map.get_mut(&code) {
        qrcode.status = if data.status == "using" {
            AuthQrCodeStatus::Using
        } else {
            AuthQrCodeStatus::Used
        };
        if data.status == "confirmed" {
            qrcode.info = Some(Info {
                stu_id: stu_id.to_string(), // token中的stuId
                name: "新用户".to_string(), // token中的name，或者从数据库中获取
            });
        }
        Ok(qrcode.status.clone().into())
    } else {
        Err("未找到二维码".into())
    }
}

pub async fn get_auth_qrcode_info_handler(Path(code): Path<String>) -> AppResult {
    let mut map = AUTH_QRCODE_MAP.lock().unwrap();

    if let Some(qrcode) = map.get_mut(&code) {
        if let AuthQrCodeStatus::Used = qrcode.status {
            let temp = qrcode.clone();
            if qrcode.info.is_some() {
                map.remove(&code);
            }
            Ok(temp.into())
        } else {
            Err("二维码未被使用".into())
        }
    } else {
        Err("未找到二维码".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_get_auth_qrcode() {
        get_auth_qrcode_handler().await.unwrap();
    }
}
