use anyhow::anyhow;
use spider_2024::dtos::gym::{
    FitnessAppointDetailRes, FitnessAppointRes, FitnessRawRes,
    FitnessRes, GymReq,
};

use crate::result::AppResult;

pub async fn get_fitness_grade(
    stu_id: &str,
    xn: &str,
) -> AppResult<FitnessRes> {
    let res = spider_2024::gym::get_gym_grade_handler(GymReq {
        stu_id: stu_id.to_string(),
        xn: xn
            .parse::<u16>()
            .map_err(|e| anyhow!("学年格式错误 ({xn}) {e}"))?,
    })
    .await?;
    Ok(res)
}

pub async fn get_fitness_raw_grade(
    stu_id: &str,
    xn: &str,
) -> AppResult<FitnessRawRes> {
    let res = spider_2024::gym::get_gym_raw_grade_handler(GymReq {
        stu_id: stu_id.to_string(),
        xn: xn
            .parse::<u16>()
            .map_err(|e| anyhow!("学年格式错误 ({xn}) {e}"))?,
    })
    .await?;
    Ok(res)
}

pub async fn get_fitness_appoint(
    stu_id: &str,
) -> AppResult<Vec<FitnessAppointRes>> {
    let res =
        spider_2024::gym::get_gym_appoint_handler(stu_id).await?;
    Ok(res)
}

pub async fn get_fitness_appoint_detail(
    stu_id: &str,
    class_id: &str,
    class_time: &str,
    test_time: &str,
) -> AppResult<FitnessAppointDetailRes> {
    let res = spider_2024::gym::get_gym_appoint_detail_handler(
        stu_id, class_id, class_time, test_time,
    )
    .await?;
    Ok(res)
}
