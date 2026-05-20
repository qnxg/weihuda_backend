use salvo::{Request, Router, handler, macros::Extractible};
use serde::Deserialize;

use crate::{
    result::{AppError, RouterResult},
    routers::demo::{DEMO_NAME, DEMO_STU_ID},
    service::{
        self,
        gym::{
            FitnessEye, FitnessGrade, FitnessGradeItem,
            FitnessReport, FitnessStudent, FitnessTotal,
        },
    },
    utils,
};

pub fn routers() -> Router {
    Router::with_path("pt")
        .push(Router::with_path("fitness").get(get_fitness_grade))
        .push(
            Router::with_path("fitness-appoint")
                .get(get_fitness_appoint),
        )
}

fn mock_fitness_grade() -> FitnessGrade {
    FitnessGrade {
        student: FitnessStudent {
            name: DEMO_NAME.to_string(),
            number: DEMO_STU_ID.to_string(),
        },
        total: FitnessTotal {
            grade: "及格".to_string(),
            score: 62.1,
        },
        report: FitnessReport {
            desc: "暂无".to_string(),
            status: "部分体测值异常".to_string(),
            _type: "正常".to_string(),
        },
        eye: FitnessEye {
            eye_ametropia_left: "9 未测".to_string(),
            eye_ametropia_right: "9 未测".to_string(),
            eye_mirror_left: "9 未测".to_string(),
            eye_mirror_right: "9 未测".to_string(),
            eyesight_left: "-- 未测".to_string(),
            eyesight_right: "-- 未测".to_string(),
        },
        items: vec![
            FitnessGradeItem {
                class: "green".to_string(),
                grade: 100,
                name: "BMI".to_string(),
                rank: "正常".to_string(),
                score: "178.6厘米/68.2千克".to_string(),
            },
            FitnessGradeItem {
                class: "red".to_string(),
                grade: 50,
                name: "跳远".to_string(),
                rank: "不及格".to_string(),
                score: "205.0厘米".to_string(),
            },
            FitnessGradeItem {
                class: "red".to_string(),
                grade: 0,
                name: "引体向上".to_string(),
                rank: "不及格".to_string(),
                score: "0次".to_string(),
            },
            FitnessGradeItem {
                class: "red".to_string(),
                grade: 50,
                name: "50m".to_string(),
                rank: "不及格".to_string(),
                score: "9.3秒".to_string(),
            },
            FitnessGradeItem {
                class: "red".to_string(),
                grade: 50,
                name: "长跑".to_string(),
                rank: "不及格".to_string(),
                score: "4'33''".to_string(),
            },
            FitnessGradeItem {
                class: "green".to_string(),
                grade: 72,
                name: "坐位体前屈".to_string(),
                rank: "及格".to_string(),
                score: "12.5厘米".to_string(),
            },
            FitnessGradeItem {
                class: "green".to_string(),
                grade: 80,
                name: "肺活量".to_string(),
                rank: "良好".to_string(),
                score: "4460毫升".to_string(),
            },
        ],
    }
}

#[handler]
async fn get_fitness_grade(req: &mut Request) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    if stu_id == DEMO_STU_ID {
        return Ok(mock_fitness_grade().into());
    }

    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct GetFitnessReq {
        pub xn: String,
    }
    let GetFitnessReq { xn } = req.extract().await?;
    let xn = xn.parse::<u16>().map_err(|_| AppError::ParseError)?;
    let res = service::gym::get_fitness_grade(&stu_id, xn).await?;
    Ok(res.into())
}

#[handler]
async fn get_fitness_appoint(req: &mut Request) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    let res = service::gym::get_fitness_appoint(&stu_id).await?;
    Ok(res.into())
}
