use serde::Serialize;

mod raw;

/// 大物实验平台的课程信息
#[derive(Serialize, Debug)]
pub struct Course {
    /// 课程名称
    pub name: String,
    /// 课程成绩
    ///
    /// 为 None 说明暂时没有成绩
    pub score: Option<String>,
    /// 课程id
    pub id: String,
}

/// 获取课程列表
///
/// # Parameters
///
/// - `stu_id`: 学号
/// - `semester_id`: 学期id，需要通过 [`crate::lab::get_semester`] 获取
///
/// # Returns
///
/// 返回课程列表
pub async fn get_course_list(
    stu_id: &str,
    semester_id: &str,
) -> Result<Vec<Course>, crate::Error> {
    let raw_data =
        raw::raw_course_list_data(stu_id, semester_id).await?;
    let mut res = Vec::with_capacity(raw_data.len());
    for item in raw_data {
        res.push(Course {
            name: item.CourseName,
            score: if item.CourseFinalScore.is_empty() {
                None
            } else {
                Some(item.CourseFinalScore)
            },
            id: item.CourseID,
        });
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lab::test::TEST_SEMESTER_ID;
    use crate::test::TEST_STU_ID;

    #[tokio::test]
    async fn test_get_course_list() {
        let res = get_course_list(&TEST_STU_ID, TEST_SEMESTER_ID)
            .await
            .unwrap();
        println!("{:#?}", res);
    }
}
