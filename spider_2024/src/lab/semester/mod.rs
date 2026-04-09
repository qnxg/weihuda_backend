mod raw;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

/// 大物实验平台的学期信息
#[derive(Serialize, Deserialize, Debug)]
pub struct Semester {
    /// 学年
    pub xn: u16,
    /// 学期
    pub xq: u8,
    /// 学期id
    pub id: String,
}

pub async fn get_semester(
    stu_id: &str,
) -> Result<Vec<Semester>, crate::Error> {
    let raw_data = raw::raw_semester_data(stu_id).await?;
    let mut res = Vec::with_capacity(raw_data.len());
    for item in raw_data {
        let [xn_str, _, xq_str] = item
            .text
            .split(|c| ['-', '_', ' '].contains(&c))
            .collect::<Vec<&str>>()[..]
        else {
            return Err(
                anyhow!("解析学期信息失败：{}", item.text).into()
            );
        };
        let (Ok(xn), Ok(xq)) =
            (xn_str.parse::<u16>(), xq_str.parse::<u8>())
        else {
            return Err(
                anyhow!("解析学期信息失败：{}", item.text).into()
            );
        };
        res.push(Semester {
            xn,
            xq,
            id: item.id,
        });
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TEST_STU_ID;

    #[tokio::test]
    async fn test_get_semester() {
        let res = get_semester(&TEST_STU_ID).await.unwrap();
        println!("{:#?}", res);
    }
}
