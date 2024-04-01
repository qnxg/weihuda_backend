use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AddExamNumberReq {
    pub num: String,
    pub name: String,
    pub date: String,
}

#[derive(Deserialize, Debug)]
pub struct UpdateExamNumberReq {
    pub num: String,
    pub name: String,
    pub date: String,
    pub id: u32,
}

#[derive(Deserialize, Debug)]
pub struct DeleteExamNumberReq {
    pub id: u32,
}
