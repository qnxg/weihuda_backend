use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct SpiderLabLoginInfo {
    pub RTNCode: i32, // -1 表示账号或密码错误，1 表示登录成功
    pub Data: Value, // 这个字段有可能是 string（当登录失败时），也有可能是 object（当登录成功时）
}

#[derive(Serialize)]
pub struct LabSetPasswordRes {
    pub success: bool,
    pub msg: Option<String>,
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct SpiderLabArrange {
    pub SeatNo: String,     // 座位号
    pub LabName: String,    // 实验名称
    pub CourseName: String, // 课程名称
    pub UserName: String,   // 上课老师名称
    pub Weeks: String,      // 上课周次
    pub WeekName: String,   // 上课星期几
    pub ClassDate: String, // 上课日期，格式如“2025/9/27 0:00:00”目前来看就前面的日期部分正确
    pub StartTime: String, // 上课开始时间
    pub ClassRoom: String, // 上课地点
    pub MobileNum: String, // 联系电话
    pub Email: String,     // 联系邮箱
}

#[derive(Serialize, Debug)]
pub struct LabArrangeRes {
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

#[derive(Deserialize, Debug)]
pub struct SpiderLabSemInfo {
    pub id: String,
    pub text: String,
}

#[derive(Serialize, Debug)]
pub struct LabSemInfoRes {
    pub xn: u32,
    pub xq: u32,
    pub id: String,
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct SpiderLabCourse {
    pub CourseName: String,       // 课程名称
    pub CourseFinalScore: String, // 课程成绩，没有成绩的话是空字符串
    pub CourseID: String,         // 课程id
}

#[derive(Serialize, Debug)]
pub struct LabCourseRes {
    pub name: String,
    pub score: Option<String>,
    pub id: String,
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct SpiderLabScore {
    pub AttendanceName: String, // 出勤情况
    pub LabName: String,        // 实验名称
    pub LabScore: String,       // 实验成绩，没有成绩的话是空字符串
    pub LabID: String,          // 实验id
    pub ClassRoom: String, // 上课地点，这个字段只是用来判断是否为虚拟实验的
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct SpiderLabScoreStructure {
    pub LabScoreStructureID: i32, // 成绩结构id
    pub LabScoreStructureName: String, // 成绩结构名称
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct SpiderLabScoreDetail {
    pub LabScoreStructureID: i32, // 对应的成绩结构id
    pub LabID: i32,               // 对应的实验id
    pub LabStructureScore: Option<i32>, // 分数
}

#[derive(Serialize, Debug)]
pub struct LabGradeRes {
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

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct SpiderVirtualLabGrade {
    pub LabName: String,  // 实验名称
    pub LabScore: String, // 实验成绩，没有成绩的话是空字符串
}

#[derive(Serialize, Debug)]
pub struct VirtualLabGradeRes {
    pub lab_name: String,  // 实验名称
    pub lab_score: String, // 实验成绩
}
