use crate::{infra, result::AppResult};
use anyhow::anyhow;
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
        let date_time_parts: Vec<&str> =
            item.kssj.split(' ').collect();
        if date_time_parts.len() != 2 {
            return Err(
                anyhow!("解析考试时间失败：{}", item.kssj).into()
            );
        }
        let temp = ExamArrange {
            id: item.kch,
            name: item.kskcmc,
            place: format!("{} {}", item.js_mc, item.ksxq),
            date: date_time_parts[0].to_string(),
            time: date_time_parts[1].to_string(),
            seat: item.zwh.unwrap_or_else(|| "无".to_string()),
        };
        res.push(temp);
    }
    res.sort_by(|a, b| a.date.cmp(&b.date));
    Ok(res)
}
