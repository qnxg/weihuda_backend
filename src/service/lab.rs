use crate::{
    infra::{self, captcha::LabCaptchaResolver},
    result::{AppError, AppResult, throw_error},
    service::user_state::{Lab, with_token},
    utils,
};
use hnu_query::{
    Error as SpiderError,
    lab::{login::LabToken, login::LoginIssue},
};
use serde::Serialize;

pub enum CheckPasswordResult {
    Success,
    PasswordError,
    OtherError(Option<String>),
}

pub async fn check_password(
    stu_id: &str,
    password: &str,
) -> AppResult<CheckPasswordResult> {
    match LabToken::acquire_by_login(
        stu_id,
        password,
        &LabCaptchaResolver,
        5,
    )
    .await
    {
        // TODO 把检查密码时获取到的 token 缓存起来
        Ok(_) => Ok(CheckPasswordResult::Success),
        Err(SpiderError::Other(LoginIssue::CaptchaError)) => {
            tracing::warn!("验证码识别失败");
            Err(AppError::Text("登陆失败，请重试".to_string()))
        }
        Err(SpiderError::Other(LoginIssue::PasswordError)) => {
            Ok(CheckPasswordResult::PasswordError)
        }
        Err(SpiderError::Other(LoginIssue::OtherError(error))) => {
            Ok(CheckPasswordResult::OtherError(error))
        }
        Err(e) => Err(throw_error(e, "检查大物实验系统密码失败")),
    }
}

pub async fn set_lab_pass(
    stu_id: &str,
    lab_pass: &str,
) -> AppResult<()> {
    infra::mysql::user::set_lab_password(
        stu_id,
        &utils::crypto::encrypt(lab_pass),
    )
    .await?;
    Ok(())
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
    let spider_res = with_token(Lab::new(stu_id), async |token| {
        hnu_query::lab::get_lab_schedule(&token).await
    })
    .await?;
    let mut res = Vec::new();
    for item in spider_res {
        let day = match item.day {
            1 => "星期一",
            2 => "星期二",
            3 => "星期三",
            4 => "星期四",
            5 => "星期五",
            6 => "星期六",
            7 => "星期日",
            _ => "未知",
        };
        let tmp = LabArrange {
            seat: item.seat,
            name: item.name,
            course: item.course,
            teacher: item.teacher,
            week: item.week,
            day: day.to_string(),
            date: item.date_time.format("%Y-%m-%d").to_string(),
            time: item.date_time.format("%H:%M").to_string(),
            place: item.place,
            phone: item.phone,
            email: item.email,
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
    let spider_res = with_token(Lab::new(stu_id), async |token| {
        hnu_query::lab::get_semester(&token).await
    })
    .await?;
    let mut res = Vec::new();
    for item in spider_res {
        res.push(LabSemInfo {
            xn: item.xn as u32,
            xq: item.xq as u32,
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
    let course_id_value = course_id.to_string();
    let sem_id_value = sem_id.to_string();
    let spider_res = with_token(Lab::new(stu_id), |token| {
        let course_id_value = &course_id_value;
        let sem_id_value = &sem_id_value;
        async move {
            hnu_query::lab::get_lab_grade(
                &token,
                course_id_value.as_str(),
                sem_id_value.as_str(),
            )
            .await
        }
    })
    .await?;
    let mut labs = Vec::new();
    // 过滤还没有成绩的实验和虚拟实验
    for item in spider_res {
        let details = item
            .details
            .iter()
            .map(|i| LabScoreDetailItem {
                name: i.name.clone(),
                score: i
                    .score
                    .map(|v| v.to_string())
                    .unwrap_or("未知".to_string()),
            })
            .collect();
        let temp = LabScoreItem {
            lab_name: item.lab_name,
            lab_score: item.score,
            attendance: item.attendance,
            details,
        };
        labs.push(temp);
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
///
/// 一学期一般只有一个物理实验课程。如果一个人修了多个实验课程的话，这个函数可能会出现问题，目前的行为是只返回第一个课程的信息
///
/// 返回 None 说明该学期没有课程
pub async fn get_course(
    stu_id: &str,
    sem_id: &str,
) -> AppResult<Option<LabCourse>> {
    let sem_id_value = sem_id.to_string();
    let spider_res = with_token(Lab::new(stu_id), |token| {
        let sem_id_value = &sem_id_value;
        async move {
            hnu_query::lab::get_course_list(
                &token,
                sem_id_value.as_str(),
            )
            .await
        }
    })
    .await?;
    if let Some(course) = spider_res.into_iter().next()
        && let Some(labs) =
            get_lab_grade_detail(stu_id, &course.id, sem_id).await?
    {
        let res = LabCourse {
            course_name: course.name,
            course_score: course.score,
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
    let spider_res = with_token(Lab::new(stu_id), async |token| {
        hnu_query::lab::get_virtual_lab_grade(&token).await
    })
    .await?;
    let mut res = Vec::new();
    for item in spider_res {
        res.push(VirtualLabGrade {
            lab_name: item.lab_name,
            lab_score: item.score.unwrap_or("暂无".to_string()),
        });
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TEST_STU_ID;

    #[tokio::test]
    async fn test_get_sem_info() {
        let sems = get_sem_info(&TEST_STU_ID).await.unwrap();
        println!("{:#?}", sems);
    }

    #[tokio::test]
    async fn test_get_course() {
        let course = get_course(&TEST_STU_ID, "17").await.unwrap();
        println!("{:#?}", course);
    }

    #[tokio::test]
    async fn test_get_lab_arrange() {
        let arrange = get_lab_arrange(&TEST_STU_ID).await.unwrap();
        println!("{:#?}", arrange);
    }

    #[tokio::test]
    async fn test_get_virtual_lab_grade() {
        let grades =
            get_virtual_lab_grade(&TEST_STU_ID).await.unwrap();
        println!("{:#?}", grades);
    }
}
