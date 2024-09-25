use serde::Deserializer;
use serde::{Deserialize, Serialize};

fn zero_to_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
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
    pub data: SpiderFitnessData,
    // pub status: u32,
    // pub info: String,
    pub report_status: Option<String>,
}
#[derive(Deserialize, Debug)]
pub struct SpiderFitnessData {
    #[serde(rename = "50m_class")]
    pub short_run_class: Option<String>,
    // #[serde(rename = "50m_grade")]
    // pub short_run_grade: String,
    #[serde(rename = "50m_score")]
    #[serde(deserialize_with = "zero_to_none")]
    pub short_run_score: Option<String>,
    pub bmi_class: Option<String>,
    // pub bmi_grade: String,
    #[serde(deserialize_with = "zero_to_none")]
    pub bmi_score: Option<String>,
    pub jump_class: Option<String>,
    // pub jump_grade: String,
    #[serde(deserialize_with = "zero_to_none")]
    pub jump_score: Option<String>,
    // pub lack_show_score_msg: f64,
    pub pull_and_sit_class: Option<String>,
    // pub pull_and_sit_grade: String,
    #[serde(deserialize_with = "zero_to_none")]
    pub pull_and_sit_score: Option<String>,
    pub report_desc: String,
    pub report_status: String,
    pub report_type: String,
    pub run_class: Option<String>,
    // pub run_grade: String,
    #[serde(deserialize_with = "zero_to_none")]
    pub run_score: Option<String>,
    pub sit_and_reach_class: Option<String>,
    pub sit_and_reach_grade: String,
    #[serde(deserialize_with = "zero_to_none")]
    pub sit_and_reach_score: Option<String>,
    // pub student_name: String,
    // pub student_num: String,
    // pub total_grade: String,
    // pub total_score: f64,
    pub vc_class: Option<String>,
    // pub vc_grade: String,
    #[serde(deserialize_with = "zero_to_none")]
    pub vc_score: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct SpiderFitnessRaw {
    pub year_num: i32,
    pub eyesight_right: String,
    pub eyesight_left: String,
    pub eye_mirror_right: String,
    pub eye_mirror_left: String,
    pub eye_ametropia_right: String,
    pub eye_ametropia_left: String,
    pub update_at: String,
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
    pub basic_score: f64,
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
    pub bmi: String,
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

#[derive(Serialize, Debug)]
pub struct FitnessRes {
    pub student: FitnessResStudent,
    pub total: FitnessResTotal,
    pub report: FitnessResReport,
    pub eye: FitnessResEye,
    pub items: Vec<FitnessResItem>,
}

#[derive(Serialize, Debug)]
pub struct FitnessResEye {
    pub eyesight_right: String,
    pub eyesight_left: String,
    pub eye_mirror_right: String,
    pub eye_mirror_left: String,
    pub eye_ametropia_right: String,
    pub eye_ametropia_left: String,
}

#[derive(Serialize, Debug)]
pub struct FitnessResReport {
    pub desc: String,
    pub status: String,
    #[serde(rename = "type")]
    pub _type: String,
}

#[derive(Serialize, Debug)]
pub struct FitnessResStudent {
    pub name: String,
    pub number: String,
}

#[derive(Serialize, Debug)]
pub struct FitnessResTotal {
    pub grade: String,
    pub score: f64,
}

#[derive(Serialize, Debug)]
pub struct FitnessResItem {
    pub name: String,
    pub class: String,
    pub rank: String,
    pub grade: i32,
    pub score: String,
}

pub fn get_class_color(raw: &str) -> String {
    if ["不及格", "缺项", "肥胖"].contains(&raw) {
        "red".to_string()
    } else {
        "green".to_string()
    }
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

#[derive(Serialize, Debug)]
pub struct FitnessAppointRes {
    pub appo_desc: String,
    pub show_time: String,
    pub test_time: String,
    pub test_type: String,
    pub class_name: String,
    pub status: String,
}
