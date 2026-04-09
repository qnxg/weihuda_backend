use anyhow::anyhow;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::Serialize;

mod raw;

/// 大物实验安排
#[derive(Serialize, Debug)]
pub struct LabSchedule {
    /// 座位号
    pub seat: String,
    /// 实验名称
    pub name: String,
    /// 实验所属的课程名称
    pub course: String,
    /// 授课老师
    pub teacher: String,
    /// 时间周次
    pub week: u8,
    /// 星期几
    ///
    /// 星期一为 `1`，星期日为 `7`
    pub day: u8,
    /// 实验的日期和时间
    pub date_time: NaiveDateTime,
    /// 实验地点
    pub place: String,
    /// 授课老师的联系电话
    pub phone: Option<String>,
    /// 授课老师的邮箱
    pub email: Option<String>,
}

pub async fn get_lab_schedule(
    stu_id: &str,
) -> Result<Vec<LabSchedule>, crate::Error> {
    let raw_data = raw::raw_lab_schedule_data(stu_id).await?;
    let mut res = Vec::with_capacity(raw_data.len());
    for item in raw_data {
        let day = match item.WeekName.as_str() {
            "星期一" => 1,
            "星期二" => 2,
            "星期三" => 3,
            "星期四" => 4,
            "星期五" => 5,
            "星期六" => 6,
            "星期日" => 7,
            _ => {
                return Err(anyhow!(
                    "意料之外的星期几: {}",
                    item.WeekName
                )
                .into());
            }
        };
        let week = item.Weeks.parse().map_err(|e| {
            anyhow!(
                "解析周数失败: data = {}, err = {}",
                item.Weeks,
                e
            )
        })?;
        let date = item
            .ClassDate
            .split(' ')
            .next()
            .and_then(|v| {
                NaiveDate::parse_from_str(v, "%Y/%m/%d").ok()
            })
            .ok_or(anyhow!("解析时间失败: {}", item.ClassDate))?;
        let time =
            NaiveTime::parse_from_str(&item.StartTime, "%H:%M")
                .map_err(|e| {
                    anyhow!(
                        "解析时间失败: data = {}, err = {}",
                        item.StartTime,
                        e
                    )
                })?;
        let tmp = LabSchedule {
            seat: item.SeatNo,
            name: item.LabName,
            course: item.CourseName,
            teacher: item.UserName,
            week,
            day,
            date_time: date.and_time(time),
            place: item.ClassRoom,
            phone: if item.MobileNum.is_empty() {
                None
            } else {
                Some(item.MobileNum)
            },
            email: if item.Email.is_empty() {
                None
            } else {
                Some(item.Email)
            },
        };
        res.push(tmp);
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TEST_STU_ID;

    #[tokio::test]
    async fn test_get_lab_schedule() {
        let res = get_lab_schedule(&TEST_STU_ID).await.unwrap();
        println!("{:#?}", res);
    }
}
