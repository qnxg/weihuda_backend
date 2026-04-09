use crate::netflow::pay_info::raw::raw_pay_info_data;

mod raw;

/// 获取欠费金额
pub async fn get_overdue_payment(
    stu_id: &str,
) -> Result<f64, crate::Error> {
    let raw_data = raw_pay_info_data(stu_id).await?;
    Ok(raw_data.Total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TEST_STU_ID;

    #[tokio::test]
    async fn test_get_overdue_payment() {
        let res = get_overdue_payment(&TEST_STU_ID).await.unwrap();
        println!("{:?}", res);
    }
}
