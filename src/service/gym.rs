use hnu_query::{
    gym::grade::GradeItemColor, xgxt::personal_info::Gender,
};
use serde::Serialize;

use crate::{
    result::AppResult,
    service::{
        self,
        user_state::{Gym, with_token},
    },
};

fn get_color_str(color: GradeItemColor) -> String {
    match color {
        GradeItemColor::Green => "green".to_string(),
        GradeItemColor::Red => "red".to_string(),
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
    xn: u16,
) -> AppResult<FitnessGrade> {
    let grade = with_token(Gym::new(stu_id), |token| async move {
        hnu_query::gym::get_grade(&token, xn).await
    })
    .await?;
    let person_info =
        service::user_info::get_person_info(stu_id, false).await?;
    let res = FitnessGrade {
        student: FitnessStudent {
            name: grade.name,
            number: grade.stu_id,
        },
        total: FitnessTotal {
            score: grade.score,
            grade: grade.grade,
        },
        report: FitnessReport {
            desc: grade.report_desc,
            status: grade.report_status,
            _type: grade.report_type,
        },
        eye: FitnessEye {
            eyesight_right: format!(
                "{} {}",
                grade.eye.eyesight_right,
                grade.eye.eyesight_right_detail
            ),
            eyesight_left: format!(
                "{} {}",
                grade.eye.eyesight_left,
                grade.eye.eyesight_left_detail
            ),
            eye_mirror_right: format!(
                "{} {}",
                grade.eye.eye_mirror_right,
                grade.eye.eye_mirror_right_detail
            ),
            eye_mirror_left: format!(
                "{} {}",
                grade.eye.eye_mirror_left,
                grade.eye.eye_mirror_left_detail
            ),
            eye_ametropia_right: format!(
                "{} {}",
                grade.eye.eye_ametropia_right,
                grade.eye.eye_ametropia_right_detail
            ),
            eye_ametropia_left: format!(
                "{} {}",
                grade.eye.eye_ametropia_left,
                grade.eye.eye_ametropia_left_detail
            ),
        },
        items: vec![
            FitnessGradeItem {
                name: "50m".to_string(),
                class: get_color_str(grade.short_run.color),
                score: grade.short_run.grade,
                rank: grade.short_run.rank,
                grade: grade.short_run.score,
            },
            FitnessGradeItem {
                name: "BMI".to_string(),
                class: get_color_str(grade.bmi.color),
                score: grade.bmi.grade,
                rank: grade.bmi.rank,
                grade: grade.bmi.score,
            },
            FitnessGradeItem {
                name: "跳远".to_string(),
                class: get_color_str(grade.jump.color),
                score: grade.jump.grade,
                rank: grade.jump.rank,
                grade: grade.jump.score,
            },
            FitnessGradeItem {
                name: if person_info.gender == Gender::Male {
                    "引体向上"
                } else {
                    "仰卧起坐"
                }
                .to_string(),
                class: get_color_str(grade.pull_and_sit.color),
                score: grade.pull_and_sit.grade,
                rank: grade.pull_and_sit.rank,
                grade: grade.pull_and_sit.score,
            },
            FitnessGradeItem {
                name: "长跑".to_string(),
                class: get_color_str(grade.run.color),
                score: grade.run.grade,
                rank: grade.run.rank,
                grade: grade.run.score,
            },
            FitnessGradeItem {
                name: "坐位体前屈".to_string(),
                class: get_color_str(grade.sit_and_reach.color),
                score: grade.sit_and_reach.grade,
                rank: grade.sit_and_reach.rank,
                grade: grade.sit_and_reach.score,
            },
            FitnessGradeItem {
                name: "肺活量".to_string(),
                class: get_color_str(grade.vc.color),
                score: grade.vc.grade,
                rank: grade.vc.rank,
                grade: grade.vc.score,
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
    let spider_res = with_token(Gym::new(stu_id), async |token| {
        hnu_query::gym::get_appointment(&token).await
    })
    .await?;
    let res = spider_res
        .into_iter()
        .map(|item| FitnessAppoint {
            appo_desc: item.desc,
            show_time: item.show_date,
            test_time: item.time,
            test_type: match item.test_type {
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
            class_name: item.name,
            status: match item.status {
                0 => "未预约".to_string(),
                1 => "已预约".to_string(),
                2 => "已完成".to_string(),
                3 => "已过期".to_string(),
                4 => "已失效".to_string(),
                _ => "未知状态".to_string(),
            },
        })
        .collect::<Vec<_>>();
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::{TEST_STU_ID, TEST_XN};

    #[tokio::test]
    async fn test_get_fitness_grade() {
        let res =
            get_fitness_grade(&TEST_STU_ID, TEST_XN).await.unwrap();
        println!("{:#?}", res);
    }

    #[tokio::test]
    async fn test_get_fitness_appoint() {
        let res = get_fitness_appoint(&TEST_STU_ID).await.unwrap();
        println!("{:#?}", res);
    }
}
