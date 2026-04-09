use crate::hdjw::exam_schedule::raw::raw_exam_schedule_data;
use anyhow::anyhow;
use chrono::NaiveDate;
use serde::Serialize;

mod raw;

/// 考试安排
#[derive(Serialize, Debug)]
pub struct ExamSchedule {
    /// 考试课程的课程代码
    pub course_id: String,
    /// 考试课程的课程名称
    pub course_name: String,
    /// 考试校区
    ///
    /// 一些比如体育理论这样的课程，没有该信息，则该字段为 `None`
    pub area: Option<String>,
    /// 考试的教室
    ///
    /// 一些比如体育理论这样的课程，没有该信息，则该字段为 `None`
    pub classroom: Option<String>,
    /// 考试的日期
    ///
    /// 一些比如体育理论这样的课程，没有该信息，则该字段为 `None`
    ///
    /// `date` 和 `time` 会同时为 `None` 或同时为 `Some`
    pub date: Option<NaiveDate>,
    /// 考试的时间，为一个时间段，如 `14:00~16:00`
    ///
    /// 一些比如体育理论这样的课程，没有该信息，则该字段为 `None`
    ///
    /// `date` 和 `time` 会同时为 `None` 或同时为 `Some`
    pub time: Option<String>,
    /// 考试的座位号
    ///
    /// 一些比如体育理论这样的课程，没有该信息，则该字段为 `None`
    pub seat: Option<String>,
}

pub async fn get_exam_schedule(
    stu_id: &str,
    xn: u16,
    xq: u8,
) -> Result<Vec<ExamSchedule>, crate::Error> {
    let raw_data = raw_exam_schedule_data(stu_id, xn, xq).await?;
    let mut res = Vec::with_capacity(raw_data.len());
    for item in raw_data {
        let (date, time) = match item.kssj {
            Some(kssj) => {
                let [date, time] =
                    kssj.split(' ').collect::<Vec<_>>()[..]
                else {
                    return Err(anyhow!(
                        "解析考试安排时间错误: {}",
                        kssj
                    )
                    .into());
                };
                let date =
                    NaiveDate::parse_from_str(date, "%Y-%m-%d")
                        .map_err(|_| {
                            anyhow!("解析考试安排日期失败: {}", date)
                        })?;
                (Some(date), Some(time.to_string()))
            }
            None => (None, None),
        };

        res.push(ExamSchedule {
            course_id: item.kch,
            course_name: item.kskcmc,
            area: item.ksxq,
            classroom: item.js_mc,
            date,
            time: time.map(|s| s.to_string()),
            seat: item.zwh,
        });
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::{TEST_STU_ID, TEST_XN, TEST_XQ};

    #[tokio::test]
    async fn test_get_exam_schedule() {
        let res = get_exam_schedule(&TEST_STU_ID, TEST_XN, TEST_XQ)
            .await
            .unwrap();
        println!("{:#?}", res);
    }
}
