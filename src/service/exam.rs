use crate::service::user_state::{Hdjw, with_token};
use crate::{error::AppResult, infra};
use serde::Serialize;

pub use infra::mysql::exam_num::ExamNumberInfo;
pub use infra::mysql::exam_num::add_exam_num;
pub use infra::mysql::exam_num::delete_exam_num;
pub use infra::mysql::exam_num::get_exam_num_list;

#[derive(Serialize, Debug)]
pub struct ExamArrange {
    pub id: String,
    pub name: String,
    pub place: String,
    pub date: String, // 考试日期，格式为 "YYYY-MM-DD"
    pub time: String, // 考试的时间段，例如：14:00~16:00
    pub seat: String,
}

pub async fn get_exam_arrange(
    stu_id: &str,
    xn: u16,
    xq: u8,
) -> AppResult<Vec<ExamArrange>> {
    let spider_res =
        with_token(Hdjw::new(stu_id), |token| async move {
            hnu_query::hdjw::get_exam_schedule(&token, xn, xq).await
        })
        .await?;
    let mut res = Vec::new();
    for item in spider_res {
        let temp = ExamArrange {
            id: item.course_id,
            name: item.course_name,
            place: match (item.area, item.classroom) {
                (Some(area), Some(classroom)) => {
                    format!("{} {}", area, classroom)
                }
                _ => "未知".to_string(),
            },
            date: item
                .date
                .map(|date| date.format("%Y-%m-%d").to_string())
                .unwrap_or("未知".to_string()),
            time: item.time.unwrap_or("未知".to_string()),
            seat: item.seat.unwrap_or_else(|| "无".to_string()),
        };
        res.push(temp);
    }
    res.sort_by(|a, b| a.date.cmp(&b.date));
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    const STUID: &str = "";

    #[tokio::test]
    async fn test_get_exam_arrange() {
        let exam_arrange =
            get_exam_arrange(STUID, 2025, 1).await.unwrap();
        println!("{:#?}", exam_arrange);
    }
}
