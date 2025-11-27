use crate::{infra, result::AppResult};

pub async fn get_campus_email_unread_count(
    stu_id: &str,
) -> AppResult<Option<u32>> {
    let spider_res = infra::spider::pt::get_email(stu_id).await?;
    Ok(spider_res.unReadCount)
}
