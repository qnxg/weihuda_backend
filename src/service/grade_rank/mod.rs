pub mod ca;

use crate::{
    result::AppResult,
    service::user_state::{Hdjw, with_token},
};
use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GradeInfo {
    pub course_id: String,            // 课程代码
    pub course_name: String,          // 课程名称
    pub credit: f32,                  // 学分
    pub course_type1: Option<String>, // 课程性质1（必修还是选修）
    pub course_type2: String, // 课程性质2（通识必修/专业核心等）
    pub gpa: f32,             // 绩点
    pub score: f64,           // 成绩
    pub tags: Vec<String>, // 其他标签，如缓考还是什么（参考 SpiderGradeInfo 的 cjbs 说明），或者辅修等
    pub jx0404id: Option<String>, // 用于获取成绩详情
}
pub async fn get_grade(
    xn: u16,
    xq: u8,
    stu_id: &str,
) -> AppResult<Vec<GradeInfo>> {
    let spider_res =
        with_token(Hdjw::new(stu_id), async move |token| {
            hnu_query::hdjw::get_grade(&token, xn, xq).await
        })
        .await?;
    let mut res = Vec::new();
    for item in spider_res {
        let mut tags = Vec::new();
        if item.grade_type != "主修" {
            tags.push(item.grade_type);
        }
        if let Some(grade_tag) = item.grade_tag {
            tags.push(grade_tag);
        }
        let tmp = GradeInfo {
            course_id: item.course_id,
            course_name: item.course_name,
            credit: item.credit,
            course_type1: item.course_type1,
            course_type2: item.course_type2,
            gpa: item.gpa.unwrap_or(0.0),
            score: item.score,
            tags,
            jx0404id: item.jx0404id,
        };
        res.push(tmp);
    }
    Ok(res)
}

#[derive(Serialize, Debug)]
pub struct HdjwRankDetail {
    /// 算数平均成绩
    pub arithmetic: String,
    pub arithmetic_rank: String,
    pub gpa: String,
    pub gpa_rank: String,
    pub weighted: String,
    pub weighted_rank: String,
}

impl From<hnu_query::hdjw::rank::RankDetail> for HdjwRankDetail {
    fn from(value: hnu_query::hdjw::rank::RankDetail) -> Self {
        Self {
            arithmetic: value.arithmetic,
            arithmetic_rank: value.arithmetic_rank,
            gpa: value.gpa,
            gpa_rank: value.gpa_rank,
            weighted: value.weighted,
            weighted_rank: value.weighted_rank,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct HdjwRank {
    pub all: Option<HdjwRankDetail>,
    pub must: Option<HdjwRankDetail>,
    pub core: Option<HdjwRankDetail>,
}

pub use hnu_query::hdjw::rank::{
    DataSource as HdjwRankDataSource, Display as HdjwRankDisplay,
    Range as HdjwRankRange,
};

/// 从 hdjw 中获取排名信息
///
/// # Arguments
///
/// - `stu_id`: 学号
/// - `range`: 课程范围
/// - `method`: 排名方法
/// - `xn`: 学年。如果为 None 则为所有学年，此时 `xq` 参数无效
/// - `xq`: 学期，如果为 None 则为所有学期
pub async fn get_rank_from_hdjw(
    stu_id: &str,
    xn: Option<u16>,
    xq: Option<u8>,
    range: hnu_query::hdjw::rank::Range,
    data_source: hnu_query::hdjw::rank::DataSource,
    display: hnu_query::hdjw::rank::Display,
) -> AppResult<HdjwRank> {
    let selection = match xn {
        Some(xn) => match xq {
            Some(xq) => {
                vec![(xn, xq)]
            }
            None => {
                vec![(xn, 1), (xn, 2), (xn, 3)]
            }
        },
        None => vec![],
    };
    let spider_res =
        with_token(Hdjw::new(stu_id), async move |token| {
            hnu_query::hdjw::get_rank(
                &token,
                selection.as_slice(),
                range,
                data_source,
                display,
            )
            .await
        })
        .await?;
    let res = HdjwRank {
        all: spider_res.all.map(Into::into),
        must: spider_res.must.map(Into::into),
        core: spider_res.core.map(Into::into),
    };
    Ok(res)
}

#[derive(Serialize, Debug)]
pub struct GradeDetailItem {
    pub name: String,
    pub score: String,
    pub percentage: String,
}
pub async fn get_grade_detail(
    stu_id: &str,
    jx0404id: &str,
) -> AppResult<Vec<GradeDetailItem>> {
    let jx0404id_value = jx0404id.to_string();
    let spider_res =
        with_token(Hdjw::new(stu_id), async move |token| {
            hnu_query::hdjw::get_grade_detail(
                &token,
                jx0404id_value.as_str(),
            )
            .await
        })
        .await?;
    let mut res = Vec::new();
    for item in spider_res {
        let tmp = GradeDetailItem {
            name: item.name,
            score: item.score,
            percentage: item.percentage,
        };
        res.push(tmp);
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    const STUID: &str = "";

    #[tokio::test]
    async fn test_get_grade_detail() {
        let res =
            get_grade_detail(STUID, "TB001TY24I-373").await.unwrap();
        println!("{:#?}", res);
    }

    #[tokio::test]
    async fn test_get_grade() {
        let res = get_grade(2025, 1, STUID).await.unwrap();
        println!("{:#?}", res);
    }
}
