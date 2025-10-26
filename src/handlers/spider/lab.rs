use std::collections::HashMap;

use anyhow::anyhow;
use axum::{
    extract::{Query, State},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    app_error::AppError,
    app_result::{AppResult, AppState},
    entities::spider::lab::{
        LabArrangeRes, LabGradeRes, LabScoreDetailItem, LabScoreItem,
        LabSemInfoRes, LabSetPasswordRes, SpiderLabArrange,
        SpiderLabCourse, SpiderLabLoginInfo, SpiderLabScore,
        SpiderLabScoreDetail, SpiderLabScoreStructure,
        SpiderLabSemInfo, SpiderVirtualLabGrade, VirtualLabGradeRes,
    },
    handlers::back::common::validation::crypto_password,
    utils::{
        self,
        jwt::parse_stu_id,
        request::{client, spider_data},
    },
};

#[derive(Serialize, Deserialize, Debug)]
pub struct SetLabPasswordReq {
    password: String,
}

pub async fn set_lab_password_handler(
    State(data): AppState,
    Extension(token): Extension<String>,
    Json(req): Json<SetLabPasswordReq>,
) -> AppResult {
    let stuid = parse_stu_id(&token)?;
    utils::redis::clear_redis_cache(&stuid).await?;
    let spider_res: SpiderLabLoginInfo = spider_data(
        "/lab/checkPassword",
        &[("stuid", &stuid), ("password", &req.password)],
    )
    .await?;
    let res = match spider_res.RTNCode {
        1 => {
            // 加密密码
            let crypto_res = crypto_password(
                &client,
                &req.password,
                &req.password,
            )
            .await?;
            sqlx::query!(
                r#"
                UPDATE mini_bind SET labPASS = ? WHERE stuID = ?
                "#,
                crypto_res.data.pt_encrypted,
                stuid,
            )
            .execute(&data.db)
            .await?;
            LabSetPasswordRes {
                success: true,
                msg: None,
            }
        }
        _ => LabSetPasswordRes {
            success: false,
            msg: Some(
                spider_res
                    .Data
                    .as_str()
                    .unwrap_or("未知错误")
                    .to_string(),
            ),
        },
    };
    Ok(res.into())
}

pub async fn get_lab_list_handler(
    Extension(token): Extension<String>,
) -> AppResult {
    let stuid = parse_stu_id(&token)?;
    let spider_res: Option<Vec<SpiderLabArrange>> =
        spider_data("/lab/list/lab", &[("stuid", &stuid)]).await?;
    if let Some(spider_res) = spider_res {
        let mut res = Vec::new();
        for item in spider_res {
            let tmp = LabArrangeRes {
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
        Ok(res.into())
    } else {
        // 说明没有设置密码或者是密码错误
        Ok(Value::Null.into())
    }
}

pub async fn get_lab_sem_info_handler(
    Extension(token): Extension<String>,
) -> AppResult {
    let stuid = parse_stu_id(&token)?;
    let params = [("stuid", stuid)];
    let spider_res: Option<Vec<SpiderLabSemInfo>> =
        spider_data("/lab/sem_info", &params).await?;
    if let Some(spider_res) = spider_res {
        let mut res = Vec::new();
        for item in spider_res {
            let parts: Vec<&str> = item
                .text
                .split(|c| ['-', '_', ' '].contains(&c))
                .collect();
            if parts.len() != 3 {
                return Err(anyhow!(
                    "解析学期信息失败：{}",
                    item.text
                )
                .into());
            }
            let xn = parts[0].parse::<u32>().map_err(|e| {
                anyhow!("解析学年失败：{}, {}", parts[0], e)
            })?;
            let xq = parts[2].parse::<u32>().map_err(|e| {
                anyhow!("解析学期失败：{}, {}", parts[2], e)
            })?;
            res.push(LabSemInfoRes {
                xn,
                xq,
                id: item.id,
            });
        }
        Ok(res.into())
    } else {
        Ok(Value::Null.into())
    }
}

/// 获取某门课程的实验成绩详情
async fn get_lab_grade_detail(
    stuid: &str,
    course_id: &str,
    sem_id: &str,
) -> Result<Option<Vec<LabScoreItem>>, AppError> {
    // 并发请求提高速度
    let spider_params =
        [("stuid", stuid), ("course_id", course_id), ("sem", sem_id)];
    type SpiderResultType = (
        Result<Option<Vec<SpiderLabScore>>, AppError>,
        Result<Option<Vec<SpiderLabScoreDetail>>, AppError>,
        Result<Option<Vec<SpiderLabScoreStructure>>, AppError>,
    );
    let (lab_score, lab_score_detail, lab_score_structure): SpiderResultType = tokio::join!(
            spider_data("/lab/score", &spider_params),
            spider_data("/lab/score/detail", &spider_params),
            spider_data("/lab/score/structure", &spider_params)
        );
    let (lab_score, lab_score_detail, lab_score_structure) =
        match (lab_score?, lab_score_detail?, lab_score_structure?) {
            (Some(score), Some(detail), Some(structure)) => {
                (score, detail, structure)
            }
            _ => return Ok(None),
        };
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
        if let Some(index) = lab_map.get(&item.LabID) {
            if let Some(structure_name) =
                score_structure_map.get(&item.LabScoreStructureID)
            {
                let lab = labs.get_mut(*index).unwrap();
                lab.details.push(LabScoreDetailItem {
                    name: structure_name.clone(),
                    score: item
                        .LabStructureScore
                        .unwrap()
                        .to_string(),
                });
            }
        }
    }
    labs.iter_mut().for_each(|lab| {
        lab.details.sort_by(|a, b| a.name.cmp(&b.name));
    });
    Ok(Some(labs))
}

#[derive(Deserialize, Debug)]
pub struct LabGradeReq {
    sem_id: String,
}

// 注意，这个请求返回的 null 字段可能有两种含义，一种是没有设置密码/密码错误，另一种是该学期没有课程
pub async fn get_lab_grade_handler(
    Extension(token): Extension<String>,
    Query(req): Query<LabGradeReq>,
) -> AppResult {
    let stuid = parse_stu_id(&token)?;
    let sem_id = req.sem_id;
    let spider_res: Option<Vec<SpiderLabCourse>> = spider_data(
        "/lab/list/course",
        &[("stuid", &stuid), ("sem", &sem_id)],
    )
    .await?;
    if let Some(spider_res) = spider_res {
        if spider_res.is_empty() {
            return Ok(Value::Null.into());
        }
        // 考虑到一学期一般只有一个物理实验课程，所以这里直接取第一个
        let SpiderLabCourse {
            CourseName: course_name,
            CourseFinalScore: course_score,
            CourseID: course_id,
        } = spider_res.first().unwrap();
        if let Some(labs) = get_lab_grade_detail(
            stuid.as_str(),
            course_id.as_str(),
            sem_id.as_str(),
        )
        .await?
        {
            let res = LabGradeRes {
                course_name: course_name.clone(),
                course_score: if course_score.is_empty() {
                    None
                } else {
                    Some(course_score.clone())
                },
                labs,
            };
            Ok(res.into())
        } else {
            Ok(Value::Null.into())
        }
    } else {
        Ok(Value::Null.into())
    }
}

pub async fn get_lab_virtual_grade_handler(
    Extension(token): Extension<String>,
) -> AppResult {
    let stuid = parse_stu_id(&token)?;
    let spider_res: Option<Vec<SpiderVirtualLabGrade>> =
        spider_data("/lab/score/virtual", &[("stuid", &stuid)])
            .await?;
    if let Some(spider_res) = spider_res {
        let mut res = Vec::new();
        for item in
            spider_res.into_iter().filter(|i| !i.LabScore.is_empty())
        {
            let tmp = VirtualLabGradeRes {
                lab_name: item.LabName,
                lab_score: item.LabScore,
            };
            res.push(tmp);
        }
        // 可能会有重复的，需要去重
        res.sort_by(|a, b| a.lab_name.cmp(&b.lab_name));
        res.dedup_by(|a, b| a.lab_name == b.lab_name);
        Ok(res.into())
    } else {
        // 说明没有设置密码或者是密码错误
        Ok(Value::Null.into())
    }
}
