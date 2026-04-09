use crate::{lab::utils::request_lab, utils::client};
use anyhow::anyhow;
use serde::Deserialize;
use std::collections::HashMap;

const LAB_SCORE_URL: &str =
    "http://10.62.106.112/XPK/StudentScoreSearch/GetStudentLabScore";
const VIRTUAL_LAB_SCORE_URL: &str = "http://10.62.106.112/XPK/StudentScoreSearch/GetStudentFZLabScore";
const LAB_SCORE_STRUCTURE_URL: &str = "http://10.62.106.112/XPK/StudentScoreSearch/GetLabScoreStructure";
const LAB_SCORE_DETAIL_URL: &str =
    "http://10.62.106.112/XPK/StudentScoreSearch/ShowScore";

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct LabScoreItem {
    /// 出勤情况
    pub AttendanceName: String,
    /// 实验名称
    pub LabName: String,
    /// 实验成绩，没有成绩的话是空字符串
    pub LabScore: String,
    /// 实验id
    pub LabID: String,
    /// 上课地点，这个字段只是用来判断是否为虚拟实验的
    pub ClassRoom: String,
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct LabScoreDetailItem {
    /// 对应的成绩结构id
    pub LabScoreStructureID: i32,
    /// 对应的实验id
    pub LabID: i32,
    /// 分数
    pub LabStructureScore: Option<f64>,
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct LabScoreStructureItem {
    /// 成绩结构id
    pub LabScoreStructureID: i32,
    /// 成绩结构名称
    pub LabScoreStructureName: String,
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct VirtualLabScoreItem {
    /// 实验名称
    pub LabName: String,
    /// 实验成绩，没有成绩的话是空字符串
    pub LabScore: String,
}

/// 获取某个课程的实验成绩
///
/// 这里面应该是包含了虚拟实验的。但是貌似专门的虚拟实验的成绩接口能得到最新成绩
pub async fn raw_lab_score_data(
    stu_id: &str,
    course_id: &str,
    semester_id: &str,
) -> Result<Vec<LabScoreItem>, crate::Error> {
    let mut form_data = HashMap::new();
    form_data.insert("page", "1".to_string());
    form_data.insert("rows", "15".to_string());
    form_data.insert("SemID", semester_id.to_string());
    form_data.insert("CourseID", course_id.to_string());
    form_data.insert("UserID", stu_id.to_string());
    let req = client.post(LAB_SCORE_URL).form(&form_data);
    let raw_res = request_lab(stu_id, req).await?;
    let res = raw_res
        .get("rows")
        .and_then(|v| {
            serde_json::from_value::<Vec<LabScoreItem>>(v.clone())
                .ok()
        })
        .ok_or(anyhow!("解析数据失败: {:?}", raw_res))?;
    Ok(res)
}

pub async fn raw_lab_score_structure_data(
    stu_id: &str,
    course_id: &str,
) -> Result<Vec<LabScoreStructureItem>, crate::Error> {
    let req = client.get(format!(
        "{}?CourseID={}",
        LAB_SCORE_STRUCTURE_URL, course_id
    ));
    let raw_res = request_lab(stu_id, req).await?;
    let res = raw_res
        .get("Data")
        .and_then(|v| {
            serde_json::from_value::<Vec<LabScoreStructureItem>>(
                v.clone(),
            )
            .ok()
        })
        .ok_or(anyhow!("解析数据失败: {:?}", raw_res))?;
    Ok(res)
}

pub async fn raw_lab_score_detail_data(
    stu_id: &str,
    course_id: &str,
) -> Result<Vec<LabScoreDetailItem>, crate::Error> {
    let req = client.get(format!(
        "{}?CourseID={}&StudentID={}",
        LAB_SCORE_DETAIL_URL, course_id, stu_id
    ));
    let raw_res = request_lab(stu_id, req).await?;
    let res = raw_res
        .get("Data")
        .and_then(|v| v.get("Lablist"))
        .and_then(|v| {
            serde_json::from_value::<Vec<LabScoreDetailItem>>(
                v.clone(),
            )
            .ok()
        })
        .ok_or(anyhow!("解析数据失败: {:?}", raw_res))?;
    Ok(res)
}

/// 获取虚拟实验的成绩
///
/// 虚拟实验的接口有点奇怪，经过测试，无论学期和课程id怎么给，都会返回一个学期的虚拟实验的成绩
pub async fn raw_virtual_lab_score_data(
    stu_id: &str,
) -> Result<Vec<VirtualLabScoreItem>, crate::Error> {
    let mut form_data = HashMap::new();
    form_data.insert("page", "1".to_string());
    form_data.insert("rows", "15".to_string());
    // 既然怎么给都无所谓，就随便给
    form_data.insert("SemID", "0".to_string());
    form_data.insert("CourseID", "0".to_string());
    form_data.insert("UserID", stu_id.to_string());
    let req = client.post(VIRTUAL_LAB_SCORE_URL).form(&form_data);
    let raw_res = request_lab(stu_id, req).await?;
    let res = raw_res
        .get("rows")
        .and_then(|v| {
            serde_json::from_value::<Vec<VirtualLabScoreItem>>(
                v.clone(),
            )
            .ok()
        })
        .ok_or(anyhow!("解析数据失败: {:?}", raw_res))?;
    Ok(res)
}
