mod raw;

use crate::ca::grade_rank::raw::{
    UNDERGRADUATE_MAJOR_ALL_TEMPLATE_ID, raw_certification_data,
};
use anyhow::anyhow;
use regex::RegexBuilder;
use serde::{Deserialize, Serialize};

/// 可信电子凭证中的排名
#[derive(Serialize, Deserialize, Debug)]
pub struct Rank {
    /// 全部课程的平均学分绩点
    pub all_gpa: String,
    /// 全部课程的平均学分绩点排名
    ///
    /// 格式为 `排名/总人数`，例如 `1/100`
    pub all_gpa_rank: String,
    /// 全部课程的加权平均分
    pub all_weighted: String,
    /// 全部课程的加权平均分排名
    ///
    /// 格式为 `排名/总人数`，例如 `1/100`
    pub all_weighted_rank: String,
    /// 全部课程的算术平均分
    pub all_arithmetic: String,
    /// 全部课程的算术平均分排名
    ///
    /// 格式为 `排名/总人数`，例如 `1/100`
    pub all_arithmetic_rank: String,
    /// 必修课的平均学分绩点
    pub must_gpa: String,
    /// 必修课的加权平均分
    pub must_weighted: String,
    /// 必修课的算术平均分
    pub must_arithmetic: String,
    /// 核心课程的平均学分绩点排名
    ///
    /// 格式为 `排名/总人数`，例如 `1/100`
    pub core_gpa_rank: String,
    /// 核心课程的加权平均分排名
    ///
    /// 格式为 `排名/总人数`，例如 `1/100`
    pub core_weighted_rank: String,
    /// 核心课程的算术平均分排名
    ///
    /// 格式为 `排名/总人数`，例如 `1/100`
    pub core_arithmetic_rank: String,
}

/// 获取本科生可信电子凭证中的成绩排名
///
/// 仅计算主修课
///
/// # Arguments
///
/// - `stu_id`: 学号
///
/// # Returns
///
/// 可信电子凭证中的成绩排名
pub async fn get_grade_rank(
    stu_id: &str,
) -> Result<Rank, crate::Error> {
    let raw_data = raw_certification_data(
        stu_id,
        UNDERGRADUATE_MAJOR_ALL_TEMPLATE_ID,
    )
    .await?;
    let regex = RegexBuilder::new(r"平均学分绩点排名 ([0-9/]+).*平均学分绩点 ([0-9.]+).*核心课程平均学分绩点排名 ([0-9/]+).*必修课平均学分绩点 ([0-9.]+).*课程算术平均成绩排名 ([0-9/]+).*算术平均分 ([0-9.]+).*核心课程算术平均成绩排名 ([0-9/]+).*必修课算术平均分 ([0-9.]+).*学分加权平均成绩排名 ([0-9/]+).*加权平均分 ([0-9.]+).*核心课程学分加权平均成绩排名 ([0-9/]+).*必修课加权平均分 ([0-9.]+)")
        .dot_matches_new_line(true)
        .build()
        .expect("构建正则表达式失败");
    let caps = regex
        .captures(&raw_data)
        .ok_or(anyhow!("解析可信电子凭证失败"))?
        .iter()
        .map(|c| {
            c.map(|v| v.as_str().to_string())
                .ok_or(anyhow!("解析可信电子凭证失败: 字段为空"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    // 12 个捕获组，caps[0] 是完整匹配，共 13 个
    let [
        _,
        all_gpa_rank,
        all_gpa,
        core_gpa_rank,
        must_gpa,
        all_arithmetic_rank,
        all_arithmetic,
        core_arithmetic_rank,
        must_arithmetic,
        all_weighted_rank,
        all_weighted,
        core_weighted_rank,
        must_weighted,
    ] = caps
        .try_into()
        .map_err(|_| anyhow!("解析可信电子凭证失败: 匹配数量错误"))?;
    let res = Rank {
        all_gpa,
        all_gpa_rank,
        all_weighted,
        all_weighted_rank,
        all_arithmetic,
        all_arithmetic_rank,
        must_gpa,
        must_weighted,
        must_arithmetic,
        core_gpa_rank,
        core_arithmetic_rank,
        core_weighted_rank,
    };
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TEST_STU_ID;

    #[tokio::test]
    async fn test_get_rank() {
        let res = get_grade_rank(&TEST_STU_ID).await.unwrap();
        println!("{:#?}", res);
    }
}
