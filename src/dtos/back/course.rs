use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct GetCourseReq {
    pub xn: u32,
    pub xq: u32,
}

#[derive(Deserialize, Debug)]
pub struct AddCourseReq {
    pub classname: String,
    pub location: Option<String>,
    pub teachers: Option<String>,
    pub week: String,
    pub section: String,
    pub day: String,
    pub xn: u32,
    pub xq: u32,
}

#[derive(Deserialize, Debug)]
pub struct DeleteCourseReq {
    pub id: String,
}
