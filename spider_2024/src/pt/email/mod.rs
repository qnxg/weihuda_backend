use crate::pt::email::raw::raw_unread_email_data;

mod raw;

/// 获取未读邮件数
///
/// # Arguments
///
/// - `stu_id`: 学号
///
/// # Returns
///
/// 如果返回 None，说明未绑定邮箱，需要前往个人门户 -> 安全中心绑定邮箱
pub async fn get_unread_email_count(
    stu_id: &str,
) -> Result<Option<u32>, crate::Error> {
    let res = raw_unread_email_data(stu_id).await?;
    Ok(res.unReadCount)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TEST_STU_ID;

    #[tokio::test]
    async fn test_get_unread_email_count() {
        let res = get_unread_email_count(&TEST_STU_ID).await.unwrap();
        println!("{:#?}", res);
    }
}
