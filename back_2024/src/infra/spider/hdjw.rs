use anyhow::anyhow;
use serde_json::Value;
use spider_2024::dtos::hdjw::{
    CourseInfoRes, EmptyRoomReq, ExamArrangeItemRes,
    ExtraCourseInfoRes, GradeInfoRes, GradeReq, HdjwGradeRankReq,
    RankRes,
};

use crate::result::AppResult;

pub async fn get_course(
    xn: u32,
    xq: u32,
    stu_id: &str,
) -> AppResult<Vec<CourseInfoRes>> {
    let spider_res =
        spider_2024::hdjw::get_class_table_handler(GradeReq {
            stu_id: stu_id.to_string(),
            xn: xn as u16,
            xq: xq as u8,
        })
        .await?;

    Ok(spider_res)
}

pub async fn get_grade(
    xn: u32,
    xq: u32,
    stu_id: &str,
) -> AppResult<Vec<GradeInfoRes>> {
    let spider_res = spider_2024::hdjw::get_grade_handler(GradeReq {
        stu_id: stu_id.to_string(),
        xn: xn as u16,
        xq: xq as u8,
    })
    .await?;

    Ok(spider_res)
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
) -> AppResult<RankRes> {
    let spider_res = spider_2024::hdjw::get_rank_from_hdjw_handler(
        HdjwGradeRankReq {
            stu_id: stu_id.to_string(),
            course: match range {
                RankRange::All => 1,
                RankRange::Must => 2,
                RankRange::Core => 3,
            },
            rank: match method {
                RankMethod::ArithmeticAvg => 1,
                RankMethod::WeightedAvg => 2,
                RankMethod::Gpa => 3,
            },
            year: xn.map(|v| v as u16),
            term: xq.map(|v| v as u8),
        },
    )
    .await?;
    Ok(spider_res)
}

pub async fn get_exam_arrange(
    stu_id: &str,
    xn: u32,
    xq: u32,
) -> AppResult<Vec<ExamArrangeItemRes>> {
    let spider_res =
        spider_2024::hdjw::get_exam_schedule_handler(GradeReq {
            stu_id: stu_id.to_string(),
            xn: xn as u16,
            xq: xq as u8,
        })
        .await?;
    Ok(spider_res)
}
pub async fn get_empty_room(
    stu_id: &str,
    build_id: &str,
    day: &str,
    jc: &Vec<&str>,
    week: u32,
    xn: u32,
    xq: u32,
) -> AppResult<Value> {
    let spider_res: Value =
        spider_2024::hdjw::get_empty_classroom_handler(
            EmptyRoomReq {
                stu_id: stu_id.to_string(),
                xn: xn as u16,
                xq: xq as u8,
                week: week.to_string(),
                day: day
                    .parse::<u8>()
                    .map_err(|e| anyhow!("星期解析失败 {}", e))?,
                jc: jc.join(","),
                build_id: build_id.to_string(),
            },
        )
        .await?;
    Ok(spider_res)
}

pub async fn get_grade_detail(
    stu_id: &str,
    jx0404id: &str,
) -> AppResult<String> {
    let spider_res: String =
        spider_2024::hdjw::get_grade_detail_handler(stu_id, jx0404id)
            .await?;
    Ok(spider_res)
}

pub async fn get_class_table_extra(
    stu_id: &str,
    xn: u32,
    xq: u32,
) -> AppResult<Vec<ExtraCourseInfoRes>> {
    let spider_res =
        spider_2024::hdjw::get_class_table_extra_handler(GradeReq {
            stu_id: stu_id.to_string(),
            xn: xn as u16,
            xq: xq as u8,
        })
        .await?;
    Ok(spider_res)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_get_course() {
        let res = get_empty_room(
            "",
            "106",
            "4",
            &vec!["0102", "0304"],
            11,
            2025,
            1,
        )
        .await
        .unwrap();
        println!("{:#?}", res);
    }
}
