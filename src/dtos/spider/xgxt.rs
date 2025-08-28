use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PersonInfo {
    pub name: String,      // 姓名
    pub gender: String,    // 性别
    pub politic: String,   // 政治面貌
    pub race: String,      // 民族
    pub hometown: String,  // 籍贯
    pub level: String,     // 培养层次，本科/研究生/博士生
    pub academy: String,   // 学院
    pub major: String,     // 专业
    pub class: String,     // 班级
    pub dormitory: String, // 寝室楼
    pub room: String,      // 房间号
    pub phone: String,     // 手机号
    pub wechat: String,    // 微信号
    pub qq: String,        // qq号
    pub email: String,     // 电子邮箱
    pub enter_year: u16,   // 年级（入学年份应该与年级相等）
    pub xz: u8,            // 学制
    pub stu_id: String,    // 学号
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Dormitory {
    pub park: String,
    pub build: String,
    pub room: String,
}
