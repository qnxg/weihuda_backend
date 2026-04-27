use crate::{
    result::AppResult,
    service::user_state::{Pt, with_token},
};

pub async fn get_unread_email_count(
    stu_id: &str,
) -> AppResult<Option<u32>> {
    let spider_res = with_token(Pt::new(stu_id), async |token| {
        hnu_query::pt::email::get_unread_email_count(&token).await
    })
    .await?;
    Ok(spider_res)
}
