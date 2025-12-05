use serde::Serialize;
use tokio::try_join;

use crate::{infra, result::AppResult, service};

fn get_class_color(raw: &str) -> String {
    if ["不及格", "缺项", "肥胖"].contains(&raw) {
        "red".to_string()
    } else {
        "green".to_string()
    }
}
#[derive(Serialize, Debug)]
pub struct FitnessGrade {
    pub student: FitnessStudent,
    pub total: FitnessTotal,
    pub report: FitnessReport,
    pub eye: FitnessEye,
    pub items: Vec<FitnessGradeItem>,
}
#[derive(Serialize, Debug)]
pub struct FitnessEye {
    pub eyesight_right: String,
    pub eyesight_left: String,
    pub eye_mirror_right: String,
    pub eye_mirror_left: String,
    pub eye_ametropia_right: String,
    pub eye_ametropia_left: String,
}
#[derive(Serialize, Debug)]
pub struct FitnessReport {
    pub desc: String,
    pub status: String,
    #[serde(rename = "type")]
    pub _type: String,
}
#[derive(Serialize, Debug)]
pub struct FitnessStudent {
    pub name: String,
    pub number: String,
}
#[derive(Serialize, Debug)]
pub struct FitnessTotal {
    pub grade: String,
    pub score: f64,
}
#[derive(Serialize, Debug)]
pub struct FitnessGradeItem {
    pub name: String,
    pub class: String,
    pub rank: String,
    pub grade: i32,
    pub score: String,
}
#[expect(clippy::too_many_lines, reason = "REFACTOR ME")]
pub async fn get_fitness_grade(
    stu_id: &str,
    xn: &str,
) -> AppResult<FitnessGrade> {
    let (grade, raw_grade, person_info) = try_join!(
        infra::spider::gymos::get_fitness_grade(stu_id, xn),
        infra::spider::gymos::get_fitness_raw_grade(stu_id, xn),
        service::user_info::get_person_info(stu_id)
    )?;
    let res = FitnessGrade {
        student: FitnessStudent {
            name: raw_grade.student_name,
            number: raw_grade.student_num,
        },
        total: FitnessTotal {
            score: raw_grade.total_score,
            grade: raw_grade.total_grade,
        },
        report: FitnessReport {
            desc: raw_grade.report_desc,
            status: raw_grade.status.to_string(),
            _type: raw_grade.report_type.to_string(),
        },
        eye: FitnessEye {
            eyesight_right: format!(
                "{} {}",
                raw_grade.eyesight_right,
                raw_grade.eyesight_right_detail
            ),
            eyesight_left: format!(
                "{} {}",
                raw_grade.eyesight_left,
                raw_grade.eyesight_left_detail
            ),
            eye_mirror_right: format!(
                "{} {}",
                raw_grade.eye_mirror_right,
                raw_grade.eye_mirror_right_detail
            ),
            eye_mirror_left: format!(
                "{} {}",
                raw_grade.eye_mirror_left,
                raw_grade.eye_mirror_left_detail
            ),
            eye_ametropia_right: format!(
                "{} {}",
                raw_grade.eye_ametropia_right,
                raw_grade.eye_ametropia_right_detail
            ),
            eye_ametropia_left: format!(
                "{} {}",
                raw_grade.eye_ametropia_left,
                raw_grade.eye_ametropia_left_detail
            ),
        },
        items: vec![
            FitnessGradeItem {
                name: "50m".to_string(),
                class: grade.short_run_class.unwrap_or(
                    get_class_color(&raw_grade.short_run_grade),
                ),
                score: grade
                    .short_run_score
                    .unwrap_or(raw_grade.short_run + "秒"),
                rank: raw_grade.short_run_grade,
                grade: raw_grade.short_run_score,
            },
            FitnessGradeItem {
                name: "BMI".to_string(),
                class: grade
                    .bmi_class
                    .unwrap_or(get_class_color(&raw_grade.bmi_grade)),
                score: grade.bmi_score.unwrap_or(format!(
                    "{}厘米/{}千克",
                    raw_grade.height, raw_grade.weight
                )),
                rank: raw_grade.bmi_grade,
                grade: raw_grade.bmi_score,
            },
            FitnessGradeItem {
                name: "跳远".to_string(),
                class: grade.jump_class.unwrap_or(get_class_color(
                    &raw_grade.jump_grade,
                )),
                score: grade
                    .jump_score
                    .unwrap_or(raw_grade.jump + "厘米"),
                rank: raw_grade.jump_grade,
                grade: raw_grade.jump_score,
            },
            FitnessGradeItem {
                name: if person_info.gender == "男" {
                    "引体向上"
                } else {
                    "仰卧起坐"
                }
                .to_string(),
                class: grade.pull_and_sit_class.unwrap_or(
                    get_class_color(&raw_grade.pull_and_sit_grade),
                ),
                score: grade
                    .pull_and_sit_score
                    .unwrap_or(raw_grade.pull_and_sit.to_string()),
                rank: raw_grade.pull_and_sit_grade,
                grade: raw_grade.pull_and_sit_score
                    + raw_grade.extra_score_pull_or_sit_up,
            },
            FitnessGradeItem {
                name: "长跑".to_string(),
                class: grade
                    .run_class
                    .unwrap_or(get_class_color(&raw_grade.run_grade)),
                score: grade.run_score.unwrap_or({
                    let total_seconds: u32 =
                        raw_grade.run.parse().unwrap_or(0);
                    let minutes = total_seconds / 60;
                    let seconds = total_seconds - minutes * 60;
                    if seconds != 0 {
                        format!("{}'{}\"", minutes, seconds)
                    } else {
                        format!("{}'", minutes)
                    }
                }),
                rank: raw_grade.run_grade,
                grade: raw_grade.run_score
                    + raw_grade.extra_score_run,
            },
            FitnessGradeItem {
                name: "坐位体前屈".to_string(),
                class: grade.sit_and_reach_class.unwrap_or(
                    get_class_color(&raw_grade.sit_and_reach_grade),
                ),
                score: grade
                    .sit_and_reach_score
                    .unwrap_or(raw_grade.sit_and_reach + "厘米"),
                rank: raw_grade.sit_and_reach_grade,
                grade: raw_grade.sit_and_reach_score,
            },
            FitnessGradeItem {
                name: "肺活量".to_string(),
                class: grade
                    .vc_class
                    .unwrap_or(get_class_color(&raw_grade.vc_grade)),
                score: grade
                    .vc_score
                    .unwrap_or(raw_grade.vc.to_string() + "毫升"),
                rank: raw_grade.vc_grade,
                grade: raw_grade.vc_score,
            },
        ],
    };
    Ok(res)
}

#[derive(Serialize, Debug)]
pub struct FitnessAppoint {
    pub appo_desc: String,
    pub show_time: String,
    pub test_time: String,
    pub test_type: String,
    pub class_name: String,
    pub status: String,
}
pub async fn get_fitness_appoint(
    stu_id: &str,
) -> AppResult<Vec<FitnessAppoint>> {
    let spider_res =
        infra::spider::gymos::get_fitness_appoint(stu_id).await?;
    let mut res = Vec::with_capacity(spider_res.len());
    for item in spider_res {
        let detail =
            infra::spider::gymos::get_fitness_appoint_detail(
                stu_id,
                &item.class_id.to_string(),
                &item.class_time,
                &item.test_time,
            )
            .await?;
        let temp = FitnessAppoint {
            appo_desc: detail.class_desc,
            show_time: item.show_time,
            test_time: item.test_time,
            test_type: match detail.appo_type {
                0 => "两项以上".to_string(),
                1 => "身高体重".to_string(),
                2 => "肺活量".to_string(),
                3 => "立定跳远".to_string(),
                4 => "坐位体前屈".to_string(),
                5 => "引体向上/仰卧起坐".to_string(),
                7 => "50米".to_string(),
                8 => "800米/1000米".to_string(),
                9 => "视力".to_string(),
                _ => "未知类型".to_string(),
            },
            class_name: item.class_name,
            status: match item.button_status {
                0 => "未预约".to_string(),
                1 => "已预约".to_string(),
                2 => "已完成".to_string(),
                3 => "已过期".to_string(),
                4 => "已失效".to_string(),
                _ => "未知状态".to_string(),
            },
        };
        res.push(temp);
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_fitness_appoint() {
        let res = get_fitness_appoint("202402050201").await.unwrap();
        println!("{:#?}", res);
    }
}
