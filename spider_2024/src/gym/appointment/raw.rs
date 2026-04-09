use serde::{Deserialize, Serialize};

use crate::{
    gym::{
        login::{gym_headers, gym_headers_from_cas},
        raw::{GymResponse, GymResponseExtractor},
    },
    utils::client,
};

const DETAIL_URL: &str = "http://gymos.hnu.edu.cn/bdlp_api_fitness_test_student_h5/public/index.php/index/Appoint/getSchoolFitClassDetail";
const APPOINT_URL: &str = "http://gymos.hnu.edu.cn/bdlp_api_fitness_test_student_h5/public/index.php/index/Appoint/getStudentClass";

#[derive(Serialize, Deserialize, Debug)]
pub struct AppointmentItem {
    pub class_id: u32,
    pub button_status: i32,
    pub class_name: String,
    /// 如：2025-12-15
    pub class_time: String,
    /// 如：2025年12月15号（周一）
    pub show_time: String,
    /// 如：10:00 - 11:30
    pub test_time: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppointmentDetail {
    pub class_desc: String,
    pub appo_type: i32,
}

pub async fn raw_appointment_list_data(
    stu_id: &str,
) -> Result<Vec<AppointmentItem>, crate::Error> {
    let gym_headers =
        if let Ok(direct_login) = gym_headers(stu_id).await {
            direct_login
        } else {
            gym_headers_from_cas(stu_id).await?
        };

    client
        .post(APPOINT_URL)
        .headers(gym_headers)
        .send()
        .await?
        .error_for_status()?
        .extract_data::<Vec<AppointmentItem>>()
        .await?
        .check_cache(stu_id)
        .await
        .into_result()
}

/// 获取体测预约详情
///
/// # Arguments
///
/// - `class_id`, `class_time`, `test_time` 均为 `raw_appointment_list_data` 返回的 `AppointmentItem` 中的字段
pub async fn raw_appointment_detail_data(
    stu_id: &str,
    class_id: u32,
    class_time: &str,
    test_time: &str,
) -> Result<AppointmentDetail, crate::Error> {
    let gym_headers =
        if let Ok(direct_login) = gym_headers(stu_id).await {
            direct_login
        } else {
            gym_headers_from_cas(stu_id).await?
        };
    client
        .post(DETAIL_URL)
        .form(&[
            ("class_id", class_id.to_string()),
            ("class_time", class_time.to_string()),
            ("test_time", test_time.to_string()),
        ])
        .headers(gym_headers)
        .send()
        .await?
        .error_for_status()?
        .extract_data::<AppointmentDetail>()
        .await?
        .check_cache(stu_id)
        .await
        .into_result()
}
