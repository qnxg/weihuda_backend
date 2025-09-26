use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SpiderCourseInfo {
    pub kch: String,          // 课程代码
    pub kc_mc: String,        // 课程名称
    pub jg0101mc: String,     // 教师名称
    pub jsgh: String,         // 教师工号（暂时不用）
    pub kt_mc: String,        // 上课班级
    pub pkrs: u16,            // 课堂容量（暂时不用）
    pub xkrs: u16,            // 上课人数
    pub kcxz: String,         // 课程性质（通识必修/专业核心等）
    pub kclb: String,         // 课程类别（必修/选修等）
    pub jx0404id: String,     // 通知单编号（暂时不用）
    pub fzmc: Option<String>, // 分组名称，这里当作课程的备注信息
    pub sktime: String,       // 上课时间
    pub skddmc: String,       // 上课地点
    pub skxqmc: String,       // 上课校区
    pub kkyx: String,         // 开课院系（暂时不用）
    pub zhouxs: String,       // 周学时（暂时不用）
    pub xf: f32,              // 学分
    pub zxs: u16,             // 总学时（暂时不用）
    pub khfs: String,         // 考核方式（暂时不用）
                              // 还有其他一些具体学时信息的字段，懒得搞了
}
