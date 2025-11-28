use std::collections::HashMap;

use crate::{
    infra::{self, spider::lab::SpiderLabCourse},
    result::AppResult,
    utils,
};
use anyhow::anyhow;
use serde::Serialize;
use tokio::try_join;

pub async fn set_lab_pass(
    stu_id: &str,
    lab_pass: &str,
) -> AppResult<()> {
    infra::mysql::user::set_lab_pass(
        stu_id,
        &utils::crypto::encrypt(lab_pass),
    )
    .await?;
    infra::redis::clear_stuid_cache(stu_id).await?;
    Ok(())
}

/// 如果返回 None 说明没有错误
/// 如果返回 Some，则表示错误信息
pub async fn check_lab_pass(
    stu_id: &str,
    lab_pass: &str,
) -> AppResult<Option<String>> {
    let spider_res =
        infra::spider::lab::check_lab_pass(stu_id, lab_pass).await?;
    match spider_res.RTNCode {
        1 => Ok(None),
        _ => Ok(Some(
            spider_res
                .Data
                .as_str()
                .unwrap_or("未知错误")
                .to_string(),
        )),
    }
}

#[derive(Serialize, Debug)]
pub struct LabArrange {
    pub seat: String,
    pub name: String,
    pub course: String,
    pub teacher: String,
    pub week: u8,
    pub day: String,
    pub date: String,
    pub time: String,
    pub place: String,
    pub phone: Option<String>,
    pub email: Option<String>,
}
/// 返回 None 说明没有设置密码或者是密码错误
pub async fn get_lab_arrange(
    stu_id: &str,
) -> AppResult<Vec<LabArrange>> {
    let spider_res =
        infra::spider::lab::get_lab_arrange(stu_id).await?;
    let mut res = Vec::new();
    for item in spider_res {
        let tmp = LabArrange {
            seat: item.SeatNo,
            name: item.LabName,
            course: item.CourseName,
            teacher: item.UserName,
            week: item
                .Weeks
                .parse()
                .map_err(|e| anyhow!("解析周数失败: {}", e))?,
            day: item.WeekName,
            date: item
                .ClassDate
                .split(' ')
                .next()
                .ok_or(anyhow!("解析日期失败"))?
                .to_string(),
            time: item.StartTime,
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

#[derive(Serialize, Debug)]
pub struct LabSemInfo {
    pub xn: u32,
    pub xq: u32,
    pub id: String,
}
pub async fn get_sem_info(
    stu_id: &str,
) -> AppResult<Vec<LabSemInfo>> {
    let spider_res = infra::spider::lab::get_sem_info(stu_id).await?;
    let mut res = Vec::new();
    for item in spider_res {
        let parts: Vec<&str> = item
            .text
            .split(|c| ['-', '_', ' '].contains(&c))
            .collect();
        if parts.len() != 3 {
            return Err(
                anyhow!("解析学期信息失败：{}", item.text).into()
            );
        }
        let xn = parts[0].parse::<u32>().map_err(|e| {
            anyhow!("解析学年失败：{}, {}", parts[0], e)
        })?;
        let xq = parts[2].parse::<u32>().map_err(|e| {
            anyhow!("解析学期失败：{}, {}", parts[2], e)
        })?;
        res.push(LabSemInfo {
            xn,
            xq,
            id: item.id,
        });
    }
    Ok(res)
}

/// 获取某门课程下实验的成绩详情
async fn get_lab_grade_detail(
    stu_id: &str,
    course_id: &str,
    sem_id: &str,
) -> AppResult<Option<Vec<LabScoreItem>>> {
    let (lab_score, lab_score_detail, lab_score_structure) = try_join!(
        infra::spider::lab::get_lab_score(stu_id, course_id, sem_id),
        infra::spider::lab::get_lab_score_detail(stu_id, course_id),
        infra::spider::lab::get_lab_score_structure(
            stu_id, course_id
        ),
    )?;
    let score_structure_map: HashMap<i32, String> =
        lab_score_structure
            .into_iter()
            .map(|item| {
                (item.LabScoreStructureID, item.LabScoreStructureName)
            })
            .collect();
    let mut lab_map: HashMap<i32, usize> = HashMap::new();
    let mut labs = Vec::new();
    // 过滤还没有成绩的实验和虚拟实验
    for item in lab_score.into_iter().filter(|i| {
        !i.LabScore.is_empty() && !i.ClassRoom.contains("虚拟")
    }) {
        let lab_id = item.LabID.parse::<i32>().map_err(|e| {
            anyhow!("解析实验ID失败：{}, {}", item.LabID, e)
        })?;
        let temp = LabScoreItem {
            lab_name: item.LabName,
            lab_score: item.LabScore,
            attendance: if item.AttendanceName.is_empty() {
                None
            } else {
                Some(item.AttendanceName)
            },
            details: Vec::new(),
        };
        labs.push(temp);
        lab_map.insert(lab_id, labs.len() - 1);
    }
    for item in lab_score_detail
        .into_iter()
        .filter(|i| i.LabStructureScore.is_some())
    {
        if let Some(index) = lab_map.get(&item.LabID)
            && let Some(structure_name) =
                score_structure_map.get(&item.LabScoreStructureID)
        {
            // labs 和 lab_map 保证了一一对应关系，这里不会有 None
            let lab = labs
                .get_mut(*index)
                .expect("根据实验 id 获得的 index 无效");
            lab.details.push(LabScoreDetailItem {
                name: structure_name.clone(),
                score: item
                    .LabStructureScore
                    .map(|v| v.to_string())
                    .unwrap_or("未知".to_string()),
            });
        }
    }
    labs.iter_mut().for_each(|lab| {
        lab.details.sort_by(|a, b| a.name.cmp(&b.name));
    });
    Ok(Some(labs))
}

#[derive(Serialize, Debug)]
pub struct LabCourse {
    pub course_name: String,          // 课程名称
    pub course_score: Option<String>, // 课程成绩
    pub labs: Vec<LabScoreItem>,      // 该课程下的所有实验成绩
}
#[derive(Serialize, Debug)]
pub struct LabScoreItem {
    pub lab_name: String,                 // 实验名称
    pub lab_score: String,                // 实验成绩
    pub attendance: Option<String>,       // 出勤情况
    pub details: Vec<LabScoreDetailItem>, // 具体的成绩项，key 是成绩结构名称，value 是对应的分数
}
#[derive(Serialize, Debug)]
pub struct LabScoreDetailItem {
    pub name: String,  // 成绩组成名称
    pub score: String, // 分数
}
/// 获取某个学期的课程信息，包含了实验成绩详情
/// 一学期一般只有一个物理实验课程。如果一个人修了多个实验课程的话，这个函数可能会出现问题，目前的行为是只返回第一个课程的信息
/// 返回 None 说明该学期没有课程
pub async fn get_course(
    stu_id: &str,
    sem_id: &str,
) -> AppResult<Option<LabCourse>> {
    let spider_res =
        infra::spider::lab::get_course_list(stu_id, sem_id).await?;
    if let Some(SpiderLabCourse {
        CourseName: course_name,
        CourseFinalScore: course_score,
        CourseID: course_id,
    }) = spider_res.first()
        && let Some(labs) =
            get_lab_grade_detail(stu_id, course_id, sem_id).await?
    {
        let res = LabCourse {
            course_name: course_name.clone(),
            course_score: if course_score.is_empty() {
                None
            } else {
                Some(course_score.clone())
            },
            labs,
        };
        Ok(Some(res))
    } else {
        Ok(None)
    }
}

#[derive(Serialize, Debug)]
pub struct VirtualLabGrade {
    pub lab_name: String,  // 实验名称
    pub lab_score: String, // 实验成绩
}
pub async fn get_virtual_lab_grade(
    stu_id: &str,
) -> AppResult<Vec<VirtualLabGrade>> {
    let spider_res =
        infra::spider::lab::get_virtual_lab_grade(stu_id).await?;
    let mut res = Vec::new();
    for item in
        spider_res.into_iter().filter(|i| !i.LabScore.is_empty())
    {
        let tmp = VirtualLabGrade {
            lab_name: item.LabName,
            lab_score: item.LabScore,
        };
        res.push(tmp);
    }
    // 可能会有重复的，需要去重
    res.sort_by(|a, b| a.lab_name.cmp(&b.lab_name));
    res.dedup_by(|a, b| a.lab_name == b.lab_name);
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    const STUID: &str = "";

    #[tokio::test]
    async fn test_get_sem_info() {
        let sems = get_sem_info(STUID).await.unwrap();
        println!("{:#?}", sems);
    }

    #[tokio::test]
    async fn test_get_course() {
        let course = get_course(STUID, "17").await.unwrap();
        println!("{:#?}", course);
    }

    #[tokio::test]
    async fn test_get_lab_arrange() {
        let arrange = get_lab_arrange(STUID).await.unwrap();
        println!("{:#?}", arrange);
    }

    #[tokio::test]
    async fn test_get_virtual_lab_grade() {
        let grades = get_virtual_lab_grade(STUID).await.unwrap();
        println!("{:#?}", grades);
    }
}
