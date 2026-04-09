use std::collections::HashMap;

use anyhow::anyhow;
use serde::Deserialize;

use crate::{lab::utils::request_lab, utils::client};

const COURSE_LIST_URL: &str =
    "http://10.62.106.112/XPK/StudentScoreSearch/GetStudentScoreList";

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct CourseItem {
    /// 课程名称
    pub CourseName: String,
    /// 课程总成绩，没有成绩的话是空字符串
    ///
    /// 如果需要获取课程的具体成绩，请使用 `lab::get_lab_grade` 来获取
    pub CourseFinalScore: String,
    /// 课程id
    pub CourseID: String,
}

pub async fn raw_course_list_data(
    stu_id: &str,
    semester_id: &str,
) -> Result<Vec<CourseItem>, crate::Error> {
    let mut form_data = HashMap::new();
    form_data.insert("page", "1".to_string());
    form_data.insert("rows", "15".to_string());
    form_data.insert("SemID", semester_id.to_string());
    form_data.insert("UserID", stu_id.to_string());
    let req = client.post(COURSE_LIST_URL).form(&form_data);
    let raw_res = request_lab(stu_id, req).await?;
    let res = raw_res
        .get("rows")
        .and_then(|v| {
            serde_json::from_value::<Vec<CourseItem>>(v.clone()).ok()
        })
        .ok_or(anyhow!("解析数据失败: {:?}", raw_res))?;
    Ok(res)
}
