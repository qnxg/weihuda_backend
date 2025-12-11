use crate::{infra, result::AppResult};
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
    xn: u32,
    xq: u32,
) -> AppResult<Vec<ExamArrange>> {
    let spider_res =
        infra::spider::hdjw::get_exam_arrange(stu_id, xn, xq).await?;
    let mut res = Vec::new();
    for item in spider_res {
        let date_time = item.kssj.map(|kssj| {
            kssj.split(' ').map(|s| s.to_string()).collect::<Vec<_>>()
        });
        let temp = ExamArrange {
            id: item.kch,
            name: item.kskcmc,
            place: match (item.ksxq, item.js_mc) {
                (Some(xq), Some(js)) => format!("{} {}", xq, js),
                _ => "未知".to_string(),
            },
            date: date_time
                .as_ref()
                .and_then(|v| v.first())
                .unwrap_or(&"未知".to_string())
                .to_string(),
            time: date_time
                .as_ref()
                .and_then(|v| v.get(1))
                .unwrap_or(&"未知".to_string())
                .to_string(),
            seat: item.zwh.unwrap_or_else(|| "无".to_string()),
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
