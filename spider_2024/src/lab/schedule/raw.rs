use crate::{lab::utils::request_lab, utils::client};
use anyhow::anyhow;
use serde::Deserialize;
use std::collections::HashMap;

const LAB_LIST_URL: &str =
    "http://10.62.106.112/XPK/StuCourseElectiveLook/LoadTableInfo";

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct LabScheduleItem {
    /// 座位号
    pub SeatNo: String,
    /// 实验名称
    pub LabName: String,
    /// 课程名称
    pub CourseName: String,
    /// 上课老师名称
    pub UserName: String,
    /// 上课周次
    pub Weeks: String,
    /// 上课星期几
    pub WeekName: String,
    /// 上课日期，格式如“2025/9/27 0:00:00”目前来看就前面的日期部分正确
    pub ClassDate: String,
    /// 上课开始时间
    pub StartTime: String,
    /// 上课地点
    pub ClassRoom: String,
    /// 联系电话
    pub MobileNum: String,
    /// 联系邮箱
    pub Email: String,
}

pub async fn raw_lab_schedule_data(
    stu_id: &str,
) -> Result<Vec<LabScheduleItem>, crate::Error> {
    let mut form_data = HashMap::new();
    form_data.insert("CourseID", "-999".to_string());
    form_data.insert("weeks", "-999".to_string());
    form_data.insert("labID", "-999".to_string());
    form_data.insert("page", "1".to_string());
    form_data.insert("rows", "200".to_string());
    let req = client.post(LAB_LIST_URL).form(&form_data);
    let raw_res = request_lab(stu_id, req).await?;
    let res = raw_res
        .get("rows")
        .and_then(|v| {
            serde_json::from_value::<Vec<LabScheduleItem>>(v.clone())
                .ok()
        })
        .ok_or(anyhow!("解析数据失败: {:?}", raw_res))?;
    Ok(res)
}
