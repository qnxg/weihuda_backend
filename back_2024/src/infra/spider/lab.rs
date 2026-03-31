use spider_2024::dtos::lab::{
    LabArrangeItem, LabCourseItem, LabLoginInfoRes,
    LabScoreDetailItem, LabScoreItem, LabScoreStructureItem,
    LabSemInfoRes, VirtualLabGradeItem,
};

use crate::result::{AppError, AppResult};

pub async fn check_lab_pass(
    stu_id: &str,
    lab_pass: &str,
) -> AppResult<LabLoginInfoRes> {
    let spider_res = spider_2024::lab::check_lab_password_handler(
        stu_id, lab_pass,
    )
    .await?;
    Ok(spider_res)
}

/// 获取实验安排列表
pub async fn get_lab_arrange(
    stu_id: &str,
) -> AppResult<Vec<LabArrangeItem>> {
    let spider_res =
        spider_2024::lab::get_lab_list_handler(stu_id).await?;
    spider_res.ok_or(AppError::PasswordError)
}

pub async fn get_sem_info(
    stu_id: &str,
) -> AppResult<Vec<LabSemInfoRes>> {
    let spider_res =
        spider_2024::lab::get_lab_sem_info_handler(stu_id).await?;
    spider_res.ok_or(AppError::PasswordError)
}

/// 获取某门课程下实验的成绩
pub async fn get_lab_score(
    stu_id: &str,
    course_id: &str,
    sem_id: &str,
) -> AppResult<Vec<LabScoreItem>> {
    let spider_res = spider_2024::lab::get_lab_score_handler(
        stu_id, course_id, sem_id,
    )
    .await?;
    spider_res.ok_or(AppError::PasswordError)
}

/// 获取某门课程下实验的具体成绩
pub async fn get_lab_score_detail(
    stu_id: &str,
    course_id: &str,
) -> AppResult<Vec<LabScoreDetailItem>> {
    let spider_res = spider_2024::lab::get_lab_score_detail_handler(
        stu_id, course_id,
    )
    .await?;
    spider_res.ok_or(AppError::PasswordError)
}

/// 获取某门课程的实验成绩结构
pub async fn get_lab_score_structure(
    stu_id: &str,
    course_id: &str,
) -> AppResult<Vec<LabScoreStructureItem>> {
    let spider_res =
        spider_2024::lab::get_lab_score_structure_handler(
            stu_id, course_id,
        )
        .await?;
    spider_res.ok_or(AppError::PasswordError)
}

/// 获取实验课程列表
pub async fn get_course_list(
    stu_id: &str,
    sem_id: &str,
) -> AppResult<Vec<LabCourseItem>> {
    let spider_res =
        spider_2024::lab::get_lab_course_list_handler(stu_id, sem_id)
            .await?;
    spider_res.ok_or(AppError::PasswordError)
}

pub async fn get_virtual_lab_grade(
    stu_id: &str,
) -> AppResult<Vec<VirtualLabGradeItem>> {
    let spider_res =
        spider_2024::lab::get_virtual_lab_score_handler(stu_id)
            .await?;
    spider_res.ok_or(AppError::PasswordError)
}
