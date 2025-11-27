use serde::Deserialize;
use serde::Deserializer;

use crate::{infra::spider::spider_data, result::AppResult};

/// If the value is None, return "0" instead.
fn none_to_zero<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer);
    if opt.is_err() {
        Ok(Some("0".to_string()))
    } else {
        Ok(opt?)
    }
}
#[derive(Deserialize, Debug)]
pub struct SpiderFitness {
    #[serde(rename = "50m_class")]
    pub short_run_class: Option<String>,
    // #[serde(rename = "50m_grade")]
    // pub short_run_grade: String,
    #[serde(rename = "50m_score")]
    #[serde(deserialize_with = "none_to_zero")]
    pub short_run_score: Option<String>,
    pub bmi_class: Option<String>,
    // pub bmi_grade: String,
    #[serde(deserialize_with = "none_to_zero")]
    pub bmi_score: Option<String>,
    pub jump_class: Option<String>,
    // pub jump_grade: String,
    #[serde(deserialize_with = "none_to_zero")]
    pub jump_score: Option<String>,
    // pub lack_show_score_msg: f64,
    pub pull_and_sit_class: Option<String>,
    // pub pull_and_sit_grade: String,
    #[serde(deserialize_with = "none_to_zero")]
    pub pull_and_sit_score: Option<String>,
    pub run_class: Option<String>,
    // pub run_grade: String,
    #[serde(deserialize_with = "none_to_zero")]
    pub run_score: Option<String>,
    pub sit_and_reach_class: Option<String>,
    #[serde(deserialize_with = "none_to_zero")]
    pub sit_and_reach_score: Option<String>,
    // pub student_name: String,
    // pub student_num: String,
    // pub total_grade: String,
    // pub total_score: f64,
    pub vc_class: Option<String>,
    // pub vc_grade: String,
    #[serde(deserialize_with = "none_to_zero")]
    pub vc_score: Option<String>,
}
pub async fn get_fitness_grade(
    stu_id: &str,
    xn: &str,
) -> AppResult<SpiderFitness> {
    let params = &[("stuid", stu_id), ("xn", xn)];
    let res = spider_data("/gymos/grade", params).await?;
    Ok(res)
}

#[derive(Deserialize, Debug)]
pub struct SpiderFitnessRaw {
    pub eyesight_right: String,
    pub eyesight_left: String,
    pub eye_mirror_right: String,
    pub eye_mirror_left: String,
    pub eye_ametropia_right: String,
    pub eye_ametropia_left: String,
    pub bmi_score: i32,
    pub vc_score: i32,
    pub jump_score: i32,
    pub sit_and_reach_score: i32,
    pub pull_and_sit_score: i32,
    #[serde(rename = "50m_score")]
    pub short_run_score: i32,
    pub run_score: i32,
    pub total_score: f64,
    pub total_grade: String,
    pub extra_score_pull_or_sit_up: i32,
    pub extra_score_run: i32,
    pub eyesight_right_detail: String,
    pub eyesight_left_detail: String,
    pub eye_mirror_right_detail: String,
    pub eye_mirror_left_detail: String,
    pub eye_ametropia_right_detail: String,
    pub eye_ametropia_left_detail: String,
    pub student_name: String,
    pub student_num: String,
    pub report_desc: String,
    pub status: i32,
    pub report_type: i32,
    pub bmi_grade: String,
    pub jump: String,
    pub jump_grade: String,
    pub pull_and_sit: i32,
    pub pull_and_sit_grade: String,
    #[serde(rename = "50m")]
    pub short_run: String,
    #[serde(rename = "50m_grade")]
    pub short_run_grade: String,
    pub run: String,
    pub run_grade: String,
    pub sit_and_reach: String,
    pub sit_and_reach_grade: String,
    pub vc: i32,
    pub vc_grade: String,
    pub height: String,
    pub weight: String,
}
pub async fn get_fitness_raw_grade(
    stu_id: &str,
    xn: &str,
) -> AppResult<SpiderFitnessRaw> {
    let params = &[("stuid", stu_id), ("xn", xn)];
    let res = spider_data("/gymos/raw_grade", params).await?;
    Ok(res)
}

#[derive(Deserialize, Debug)]
pub struct SpiderFitnessAppoint {
    pub appo_desc: String,
    // pub appo_time: String,
    // pub appoint_date: String,
    // pub appoint_test_time: String,
    // pub appoint_time: String,
    // pub button_status: u32,
    // pub class_id: u32,
    pub class_name: String,
    // pub class_time: String,
    pub show_time: String,
    // pub sign_time: String,
    pub status: String,
    // pub target_id: u32,
    // pub target_type: u32,
    pub test_time: String,
    pub test_type: String,
}
pub async fn get_fitness_appoint(
    stu_id: &str,
) -> AppResult<Vec<SpiderFitnessAppoint>> {
    let params = &[("stuid", stu_id)];
    let res = spider_data("/gymos/appoint", params).await?;
    Ok(res)
}
