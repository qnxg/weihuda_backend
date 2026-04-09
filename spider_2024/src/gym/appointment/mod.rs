mod raw;

use anyhow::anyhow;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Appointment {
    /// 预约名称
    pub name: String,
    /// 预约描述
    pub desc: String,
    /// 体测系统显示的预约时间，如 `2025年12月15号（周一）`
    pub show_date: String,
    /// 预约日期，
    pub date: NaiveDate,
    /// 时间段，如 `10:00 - 11:30`
    pub time: String,
    /// 体测类型
    ///
    /// 目前已知的类型：
    /// - `0`：两项以上
    /// - `1`：身高体重
    /// - `2`：肺活量
    /// - `3`：立定跳远
    /// - `4`：坐位体前屈
    /// - `5`：引体向上/仰卧起坐
    /// - `7`：50米
    /// - `8`：800米/1000米
    /// - `9`：视力
    pub test_type: i32,
    /// 预约状态
    ///
    /// 目前已知的类型：
    /// - `0`：未预约
    /// - `1`：已预约
    /// - `2`：已完成
    /// - `3`：已过期
    /// - `4`：已失效
    pub status: i32,
}

pub async fn get_appointment(
    stu_id: &str,
) -> Result<Vec<Appointment>, crate::Error> {
    let raw_data = raw::raw_appointment_list_data(stu_id).await?;
    let mut res = Vec::with_capacity(raw_data.len());
    for raw_item in raw_data {
        let raw_detail = raw::raw_appointment_detail_data(
            stu_id,
            raw_item.class_id,
            &raw_item.class_time,
            &raw_item.test_time,
        )
        .await?;
        let temp = Appointment {
            name: raw_item.class_name,
            desc: raw_detail.class_desc,
            show_date: raw_item.show_time,
            date: NaiveDate::parse_from_str(
                &raw_item.class_time,
                "%Y-%m-%d",
            )
            .map_err(|e| {
                anyhow!(
                    "解析体测预约日期失败: date = {:?}, err = {:?}",
                    raw_item.class_time,
                    e
                )
            })?,
            time: raw_item.test_time,
            test_type: raw_detail.appo_type,
            status: raw_item.button_status,
        };
        res.push(temp);
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TEST_STU_ID;

    #[tokio::test]
    async fn test_get_appointment() {
        let res = get_appointment(&TEST_STU_ID).await.unwrap();
        println!("{:#?}", res);
    }
}
