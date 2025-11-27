use serde::Deserialize;
use serde_json::Value;

use crate::{
    infra::spider::spider_data,
    result::{AppError, AppResult},
};

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct SpiderLabLoginInfo {
    pub RTNCode: i32, // -1 表示账号或密码错误，1 表示登录成功
    pub Data: Value, // 这个字段有可能是 string（当登录失败时），也有可能是 object（当登录成功时）
}
pub async fn check_lab_pass(
    stu_id: &str,
    lab_pass: &str,
) -> AppResult<SpiderLabLoginInfo> {
    let spider_res: SpiderLabLoginInfo = spider_data(
        "/lab/checkPassword",
        &[("stuid", stu_id), ("password", lab_pass)],
    )
    .await?;
    Ok(spider_res)
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct SpiderLabArrange {
    pub SeatNo: String,     // 座位号
    pub LabName: String,    // 实验名称
    pub CourseName: String, // 课程名称
    pub UserName: String,   // 上课老师名称
    pub Weeks: String,      // 上课周次
    pub WeekName: String,   // 上课星期几
    pub ClassDate: String, // 上课日期，格式如“2025/9/27 0:00:00”目前来看就前面的日期部分正确
    pub StartTime: String, // 上课开始时间
    pub ClassRoom: String, // 上课地点
    pub MobileNum: String, // 联系电话
    pub Email: String,     // 联系邮箱
}
/// 获取实验安排列表
pub async fn get_lab_arrange(
    stu_id: &str,
) -> AppResult<Vec<SpiderLabArrange>> {
    let spider_res: Option<Vec<SpiderLabArrange>> =
        spider_data("/lab/list/lab", &[("stuid", stu_id)]).await?;
    spider_res.ok_or(AppError::PasswordError)
}

#[derive(Deserialize, Debug)]
pub struct SpiderLabSemInfo {
    pub id: String,
    pub text: String,
}
pub async fn get_sem_info(
    stu_id: &str,
) -> AppResult<Vec<SpiderLabSemInfo>> {
    let spider_res: Option<Vec<SpiderLabSemInfo>> =
        spider_data("/lab/sem_info", &[("stuid", stu_id)]).await?;
    spider_res.ok_or(AppError::PasswordError)
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct SpiderLabScore {
    pub AttendanceName: String, // 出勤情况
    pub LabName: String,        // 实验名称
    pub LabScore: String,       // 实验成绩，没有成绩的话是空字符串
    pub LabID: String,          // 实验id
    pub ClassRoom: String, // 上课地点，这个字段只是用来判断是否为虚拟实验的
}
/// 获取某门课程下实验的成绩
pub async fn get_lab_score(
    stu_id: &str,
    course_id: &str,
    sem_id: &str,
) -> AppResult<Vec<SpiderLabScore>> {
    let spider_params = [
        ("stuid", stu_id),
        ("course_id", course_id),
        ("sem", sem_id),
    ];
    let spider_res: Option<Vec<SpiderLabScore>> =
        spider_data("/lab/score", &spider_params).await?;
    spider_res.ok_or(AppError::PasswordError)
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct SpiderLabScoreDetail {
    pub LabScoreStructureID: i32, // 对应的成绩结构id
    pub LabID: i32,               // 对应的实验id
    pub LabStructureScore: Option<i32>, // 分数
}
/// 获取某门课程下实验的具体成绩
pub async fn get_lab_score_detail(
    stu_id: &str,
    course_id: &str,
) -> AppResult<Vec<SpiderLabScoreDetail>> {
    let spider_params = [("stuid", stu_id), ("course_id", course_id)];
    let spider_res: Option<Vec<SpiderLabScoreDetail>> =
        spider_data("/lab/score/detail", &spider_params).await?;
    spider_res.ok_or(AppError::PasswordError)
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct SpiderLabScoreStructure {
    pub LabScoreStructureID: i32, // 成绩结构id
    pub LabScoreStructureName: String, // 成绩结构名称
}
/// 获取某门课程的实验成绩结构
pub async fn get_lab_score_structure(
    stu_id: &str,
    course_id: &str,
) -> AppResult<Vec<SpiderLabScoreStructure>> {
    let spider_params = [("stuid", stu_id), ("course_id", course_id)];
    let spider_res: Option<Vec<SpiderLabScoreStructure>> =
        spider_data("/lab/score/structure", &spider_params).await?;
    spider_res.ok_or(AppError::PasswordError)
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct SpiderLabCourse {
    pub CourseName: String,       // 课程名称
    pub CourseFinalScore: String, // 课程成绩，没有成绩的话是空字符串
    pub CourseID: String,         // 课程id
}
/// 获取实验课程列表
pub async fn get_course_list(
    stu_id: &str,
    sem_id: &str,
) -> AppResult<Vec<SpiderLabCourse>> {
    let spider_res: Option<Vec<SpiderLabCourse>> = spider_data(
        "/lab/list/course",
        &[("stuid", stu_id), ("sem", sem_id)],
    )
    .await?;
    spider_res.ok_or(AppError::PasswordError)
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct SpiderVirtualLabGrade {
    pub LabName: String,  // 实验名称
    pub LabScore: String, // 实验成绩，没有成绩的话是空字符串
}
pub async fn get_virtual_lab_grade(
    stu_id: &str,
) -> AppResult<Vec<SpiderVirtualLabGrade>> {
    let spider_res: Option<Vec<SpiderVirtualLabGrade>> =
        spider_data("/lab/score/virtual", &[("stuid", stu_id)])
            .await?;
    spider_res.ok_or(AppError::PasswordError)
}
