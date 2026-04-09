use crate::netflow::user_info::raw::raw_user_info_data;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};

mod raw;

/// 校园网流量锁定状态
#[derive(Deserialize, Serialize, Debug)]
pub enum UnlockStatus {
    /// 已锁定
    Locked,
    /// 未锁定
    Unlocked,
    /// 未知
    Unknown,
}

pub async fn get_unlock_status(
    stu_id: &str,
) -> Result<UnlockStatus, crate::Error> {
    let raw_data = raw_user_info_data(stu_id).await?;
    let is_locked =
        raw_data.get("IsLocked").and_then(|v| v.as_i64()).ok_or(
            anyhow!("解析校园网流量锁定状态失败: {:?}", raw_data),
        )?;
    match is_locked {
        0 => Ok(UnlockStatus::Unlocked),
        1 => Ok(UnlockStatus::Locked),
        _ => Ok(UnlockStatus::Unknown),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TEST_STU_ID;

    #[tokio::test]
    async fn test_get_unlock_status() {
        let res = get_unlock_status(&TEST_STU_ID).await.unwrap();
        println!("{:?}", res);
    }
}
