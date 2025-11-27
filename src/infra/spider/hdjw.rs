use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{infra::spider::spider_data, result::AppResult};

#[derive(Deserialize, Debug)]
pub struct SpiderCourseInfo {
    pub kch: String,      // 课程代码
    pub kc_mc: String,    // 课程名称
    pub jg0101mc: String, // 教师名称
    #[expect(unused)]
    pub jsgh: String, // 教师工号（暂时不用）
    pub kt_mc: String,    // 上课班级
    #[expect(unused)]
    pub pkrs: u16, // 课堂容量（暂时不用）
    pub xkrs: u16,        // 上课人数
    pub kcxz: String,     // 课程性质（通识必修/专业核心等）
    #[expect(unused)]
    pub kclb: String, // 课程类别（必修/选修等）
    #[expect(unused)]
    pub jx0404id: String, // 通知单编号（暂时不用）
    pub fzmc: Option<String>, // 分组名称，这里当作课程的备注信息
    pub sktime: String,   // 上课时间
    pub skddmc: String,   // 上课地点
    pub skxqmc: String,   // 上课校区
    #[expect(unused)]
    pub kkyx: String, // 开课院系（暂时不用）
    #[expect(unused)]
    pub zhouxs: String, // 周学时（暂时不用）
    pub xf: f32,          // 学分
    #[expect(unused)]
    pub zxs: u16, // 总学时（暂时不用）
    #[expect(unused)]
    pub khfs: String, // 考核方式（暂时不用）
                          // 还有其他一些具体学时信息的字段，懒得搞了
}
pub async fn get_course(
    xn: u32,
    xq: u32,
    stu_id: &str,
) -> AppResult<Vec<SpiderCourseInfo>> {
    let params = [
        ("xn", xn.to_string()),
        ("xq", xq.to_string()),
        ("stuid", stu_id.to_string()),
    ];
    let spider_res: Vec<SpiderCourseInfo> =
        spider_data("/bks/classtable", &params).await?;
    Ok(spider_res)
}

#[derive(Deserialize, Debug)]
pub struct SpiderGradeInfo {
    // pub cj0708id: String, // 未知字段
    // pub xnxqid: String,   // 学年学期信息（暂时不用）
    pub kch: String,   // 课程代码
    pub kc_mc: String, // 课程名称
    // pub ksdw: String,     // 开课学院（暂时不用）
    // pub xqmc: String, // 似乎和 xnxqid 重复
    pub xf: f32, // 学分
    // pub zxs: u32,      // 总学时（暂时不用）
    // pub ksfs: String,  // 考试方式（暂时不用）
    pub kcsx: String, // 课程属性（必修/选修等）
    // pub xqstr: String, // 似乎又和 xnxqid 重复
    pub zcj: u8, // 总成绩
    // pub zcjstr: String,   // 总成绩字符串形式（暂时不用）
    // pub kz: u8,           // 未知字段
    pub kcxzmc: String, // 课程性质（通识必修/专业核心等）
    // pub xs0101id: String, // 未知字段
    // pub jx0404id: Option<String>, // 似乎和 kch 重复，部分成绩没有该字段
    pub jd: f32, // 绩点
    // pub ksxz: String,         // 考试性质（暂时不用）
    pub falb: String,         // 主修还是辅修
    pub cjbs: Option<String>, // 成绩标识（缓考/重修等，注意这个标识是挂在为 0 分的那个成绩 item 上）
}
pub async fn get_grade(
    xn: u32,
    xq: u32,
    stu_id: &str,
) -> AppResult<Vec<SpiderGradeInfo>> {
    let params = [
        ("xn", xn.to_string()),
        ("xq", xq.to_string()),
        ("stuid", stu_id.to_string()),
    ];
    let spider_res: Vec<SpiderGradeInfo> =
        spider_data("/bks/grade", &params).await?;

    Ok(spider_res)
}

#[derive(Serialize, Debug, Deserialize)]
pub struct Rank {
    pub score: String,
    pub rank: String,
}
// 排名的课程范围
pub enum RankRange {
    All,  // 全部课程
    Must, // 必修课程
    Core, // 核心课程
}
// 排名方式
pub enum RankMethod {
    ArithmeticAvg, // 算数平均分
    WeightedAvg,   // 加权平均分
    Gpa,           // 绩点
}
// xn 提供 None 表示获取从入学到现在的所有学期
// xn 提供但是 xq 不提供表示获取该学年所有学期
pub async fn get_rank(
    stu_id: &str,
    range: RankRange,
    method: RankMethod,
    xn: Option<u32>,
    xq: Option<u32>,
) -> AppResult<Rank> {
    let mut params = vec![
        ("stuid", stu_id.to_string()),
        (
            "course",
            match range {
                RankRange::All => "1",
                RankRange::Must => "2",
                RankRange::Core => "3",
            }
            .to_string(),
        ),
        (
            "rank",
            match method {
                RankMethod::ArithmeticAvg => "1",
                RankMethod::WeightedAvg => "2",
                RankMethod::Gpa => "3",
            }
            .to_string(),
        ),
    ];
    if let Some(xn) = xn {
        params.push(("year", xn.to_string()));
    }
    if let Some(xq) = xq {
        params.push(("term", xq.to_string()));
    }
    let spider_res: Rank = spider_data("/bks/rank", &params).await?;
    Ok(spider_res)
}

#[derive(Deserialize, Debug)]
pub struct SpiderExamArrangeItem {
    pub kch: String,         // 课程代码
    pub kskcmc: String,      // 课程名称
    pub ksxq: String,        // 考试校区
    pub js_mc: String,       // 考试的教室
    pub kssj: String,        // 考试时间（已经是一个时间区间了）
    pub zwh: Option<String>, // 座位号
}
pub async fn get_exam_arrange(
    stu_id: &str,
    xn: u32,
    xq: u32,
) -> AppResult<Vec<SpiderExamArrangeItem>> {
    let params = [
        ("xn", xn.to_string()),
        ("xq", xq.to_string()),
        ("stuid", stu_id.to_string()),
    ];
    let spider_res: Vec<SpiderExamArrangeItem> =
        spider_data("/bks/exam/schedule", &params).await?;
    Ok(spider_res)
}
pub async fn get_empty_room(
    stu_id: &str,
    build_id: &str,
    day: &str,
    jc: &str,
    week: u32,
    xn: u32,
    xq: u32,
) -> AppResult<Value> {
    let params = [
        ("build_id", build_id.to_string()),
        ("day", day.to_string()),
        ("jc", jc.to_string()),
        ("week", week.to_string()),
        ("xn", xn.to_string()),
        ("xq", xq.to_string()),
        ("stuid", stu_id.to_string()),
    ];
    let spider_res: Value =
        spider_data("/freeroom/list", &params).await?;
    Ok(spider_res)
}
