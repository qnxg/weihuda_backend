use super::get_db_pool;
use crate::result::AppResult;
use serde::Serialize;

#[expect(non_snake_case)]
#[derive(Serialize, Debug)]
pub struct MiniBind {
    pub openid: Option<String>,
    pub stuID: Option<String>,
    pub stuPASS: Option<String>,
    pub hdjwPASS: Option<String>,
    pub id: u32,
}

pub async fn get_by_stu_id(
    stu_id: &str,
) -> AppResult<Option<MiniBind>> {
    let res = sqlx::query_as!(
        MiniBind,
        r#"
        SELECT id, openid, stuID, stuPASS, hdjwPASS FROM mini_bind WHERE stuID = ? AND deleted_at is null
        "#,
        stu_id,
    ).fetch_optional(get_db_pool().await).await?;
    Ok(res)
}

pub async fn get_by_openid(
    openid: &str,
) -> AppResult<Option<MiniBind>> {
    let res = sqlx::query_as!(
        MiniBind,
        r#"
        SELECT id, openid, stuID, stuPASS, hdjwPASS FROM mini_bind WHERE openid = ? AND deleted_at is null
        "#,
        openid,
    ).fetch_optional(get_db_pool().await).await?;
    Ok(res)
}

#[expect(dead_code)]
pub async fn get_by_id(id: u32) -> AppResult<Option<MiniBind>> {
    let res = sqlx::query_as!(
        MiniBind,
        r#"
        SELECT id, openid, stuID, stuPASS, hdjwPASS FROM mini_bind WHERE id = ? AND deleted_at is null
        "#,
        id,
    ).fetch_optional(get_db_pool().await).await?;
    Ok(res)
}

/// 插入新用户绑定信息，如果用户已存在则更新
pub async fn add_user(
    open_id: &str,
    stu_id: &str,
    stu_pass: &str,
    hdjw_pass: &str,
) -> AppResult<()> {
    let now = chrono::Local::now();
    sqlx::query!(
        r#"
        INSERT INTO mini_bind (openid, stuID, stuPASS, hdjwPASS) VALUES (?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE openid = VALUES(openid), stuPASS = VALUES(stuPASS), hdjwPASS = VALUES(hdjwPASS), updated_at = ?
        "#,
        open_id,
        stu_id,
        stu_pass,
        hdjw_pass,
        now
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}

pub async fn delete_user(mini_bind_id: u32) -> AppResult<()> {
    let now = chrono::Local::now();
    sqlx::query!(
        r#"
        UPDATE mini_bind SET updated_at = ?, deleted_at = ?, openid = '' WHERE id = ? AND deleted_at is null
        "#,
        now,
        now,
        mini_bind_id
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}

pub async fn get_room(stu_id: &str) -> AppResult<Option<String>> {
    let text = sqlx::query_scalar!(
        "SELECT room FROM mini_bind WHERE stuID = ? AND deleted_at is NULL",
        stu_id
    )
    .fetch_optional(get_db_pool().await)
    .await?;
    Ok(text)
}

pub async fn update_room(stu_id: &str, room: &str) -> AppResult<()> {
    sqlx::query!(
        r#"
        UPDATE mini_bind SET room = ? WHERE stuID = ? AND deleted_at is null
        "#,
        room,
        stu_id
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}

pub async fn set_lab_pass(
    stu_id: &str,
    lab_pass: &str,
) -> AppResult<()> {
    sqlx::query!(
        r#"
        UPDATE mini_bind SET labPASS = ? WHERE stuID = ? AND deleted_at is null
        "#,
        lab_pass,
        stu_id
    )
    .execute(get_db_pool().await)
    .await?;
    Ok(())
}
