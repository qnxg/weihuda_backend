use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct SpiderLabGrade {
    pub zcj: String,
    pub zxs: String,
    pub items: Vec<SpiderLabGradeItems>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SpiderLabGradeItems {
    pub grade: String,
    pub grade_bg: String,
    pub grade_cz: String,
    pub grade_kq: String,
    pub grade_yx: String,
    pub labdate: String,
    pub labname: String,
    pub labtime: String,
    pub labtype: String,
    pub xs: String,
}

#[derive(Serialize, Debug)]
pub struct LabGradeRes {
    pub items: Vec<SpiderLabGradeItems>,
    pub total: LabGradeResTotal,
}

#[derive(Serialize, Debug)]
pub struct LabGradeResTotal {
    pub cj: String,
    pub xs: String,
}

#[derive(Deserialize, Debug)]
pub struct SpiderLabArrange {
    pub classname: String,
    pub classtype: String,
    pub labdate: String,
    pub labtime: String,
    pub labweek: String,
    pub labname: String,
    pub labplace: String,
}

#[derive(Serialize, Debug)]
pub struct LabArrangeRes {
    pub classname: String,
    pub classtype: String,
    pub date: String,
    pub time: String,
    pub week: String,
    pub name: String,
    pub place: String,
}
