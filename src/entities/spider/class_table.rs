use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct SpiderCourseInfo {
    pub djkssj: String,
    pub djjssj: String,
    pub jczy013id: String,
    pub js_name: Option<String>,
    pub kc_name: String,
    pub ktmc_name: String,
    pub pkzcmx: String,
    pub jczy01501ids: String,
    pub teachernames: Option<String>,
    pub pksj: String,
    pub id: String,
    pub skqk: String,
}

#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClassTableRes {
    pub id: String,
    // #[serde(rename = "classId")]
    pub class_id: String,
    pub classname: String,
    pub location: String,
    pub teachers: String,
    pub week: String,
    pub day: String,
    pub section: String,
    // #[serde(rename = "startTime")]
    pub start_time: String,
    // #[serde(rename = "endTime")]
    pub end_time: String,
    #[serde(rename = "type")]
    pub _type: u32,
    pub skqk: String,
}
