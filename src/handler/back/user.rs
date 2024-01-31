#![allow(non_snake_case)]
use std::sync::Arc;

use crate::{extract::Json, utility::request::client};
use axum::{extract::State, Extension};
use tokio::try_join;

use crate::{
    app_result::AppResult,
    handler::back::common::{
        check_user::check_by_openid,
        validation::{crypto_password, verify_password},
        wechat::get_openid,
    },
    model::back::user::UserBind,
    schema::back::user::BindReq,
    utility::jwt::parse_id,
    Pool,
};

pub async fn bind_user_handler(
    State(data): State<Arc<Pool>>,
    Json(req): Json<BindReq>,
) -> AppResult {
    // 三个请求并发提高速度，验证密码和获取openid和加密密码
    let (verify_res, openid, crypto_res) = try_join!(
        verify_password(&client, &req.stuId, &req.hdjwPassword, &req.stuPassword),
        get_openid(&req.code),
        crypto_password(&client, &req.hdjwPassword, &req.stuPassword)
    )?;

    match verify_res.code {
        0 => {} // 验证成功
        1 => return Err("个人门户密码错误".into()),
        // 2 => return Err("教务系统密码错误".into()),
        // 3 => return Err("个人门户和教务系统密码均错误".into()),
        _ => return Err("密码验证服务返回值错误".into()),
    }

    let check_res = check_by_openid(data.clone(), &openid).await;

    if check_res.is_ok() {
        return Err("该微信号已绑定".into());
    }

    let user_bind = UserBind {
        openid,
        stuID: req.stuId,
        hdjwPASS: crypto_res.data.hdjw_encrypted,
        stuPASS: crypto_res.data.pt_encrypted,
    };

    let res = sqlx::query_as!(
        UserBind,
        r#"
        SELECT openid, stuID, stuPASS, hdjwPASS FROM mini_bind WHERE stuID = ?
        "#,
        user_bind.stuID,
    )
    .fetch_one(&data.db)
    .await;

    match res {
        // 如果绑定过，就更新
        Ok(_) => {
            sqlx::query!(
                    r#"
                    UPDATE mini_bind SET openid = ?, stuID = ?, stuPASS = ?, hdjwPASS = ?, updated_at = ? WHERE stuID = ?
                    "#,
                    user_bind.openid,
                    user_bind.stuID,
                    user_bind.stuPASS,
                    user_bind.hdjwPASS,
                    chrono::Local::now(),
                    user_bind.stuID,
                )
                .execute(&data.db)
                .await?;
        }
        // 如果没有绑定过，就插入
        Err(_) => {
            sqlx::query!(
                r#"
                INSERT INTO mini_bind (openid, stuID, stuPASS, hdjwPASS) VALUES (?, ?, ?, ?)
                "#,
                user_bind.openid,
                user_bind.stuID,
                user_bind.stuPASS,
                user_bind.hdjwPASS,
            )
            .execute(&data.db)
            .await?;
        }
    }

    Ok("绑定成功".into())
}

pub async fn unbind_user_handler(
    State(data): State<Arc<Pool>>,
    Extension(token): Extension<String>,
) -> AppResult {
    let mini_bind_id = parse_id(&token)?;

    let res = sqlx::query!(
        r#"
        UPDATE mini_bind SET updated_at = ?, openid = '' WHERE id = ? AND deleted_at is null
        "#,
        chrono::Local::now(),
        mini_bind_id
    )
    .execute(&data.db)
    .await?;

    if res.rows_affected() == 0 {
        return Err("解绑失败".into());
    }

    Ok("解绑成功".into())
}
