use std::collections::HashMap;

use crate::{
    infra::{self},
    result::AppResult,
};
use anyhow::anyhow;
use regex::RegexBuilder;
use serde::Serialize;

pub use infra::spider::hdjw::RankMethod as HdjwRankMethod;
pub use infra::spider::hdjw::RankRange as HdjwRankRange;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GradeInfo {
    pub course_id: String,            // 课程代码
    pub course_name: String,          // 课程名称
    pub credit: f32,                  // 学分
    pub course_type1: Option<String>, // 课程性质1（必修还是选修）
    pub course_type2: String, // 课程性质2（通识必修/专业核心等）
    pub gpa: f32,             // 绩点
    pub score: u8,            // 成绩
    pub tags: Vec<String>, // 其他标签，如缓考还是什么（参考 SpiderGradeInfo 的 cjbs 说明），或者辅修等
    pub jx0404id: Option<String>, // 用于获取成绩详情
}
pub async fn get_grade(
    xn: u32,
    xq: u32,
    stu_id: &str,
) -> AppResult<Vec<GradeInfo>> {
    let spider_res =
        infra::spider::hdjw::get_grade(xn, xq, stu_id).await?;
    let mut res = Vec::new();
    for item in spider_res {
        let mut tags = Vec::new();
        if item.falb != "主修" {
            tags.push(item.falb);
        }
        if let Some(cjbs) = item.cjbs {
            tags.push(cjbs);
        }
        let tmp = GradeInfo {
            course_id: item.kch,
            course_name: item.kc_mc,
            credit: item.xf,
            course_type1: item.kcsx,
            course_type2: item.kcxzmc,
            gpa: item.jd,
            score: item.zcj,
            tags,
            jx0404id: item.jx0404id,
        };
        res.push(tmp);
    }
    Ok(res)
}

pub use infra::spider::hdjw::get_rank as get_rank_from_hdjw;
use serde_json::Value;

// 这样做为了做到 ca::Rank 的效果
pub mod ca {
    use serde::Serialize;

    #[derive(Serialize, Debug)]
    pub struct Rank {
        pub all_gpa: String,
        pub all_gpa_rank: String,
        pub all_weighted: String,
        pub all_weighted_rank: String,
        pub all_arithmetic: String,
        pub all_arithmetic_rank: String,
        pub must_gpa: String,
        pub must_weighted: String,
        pub must_arithmetic: String,
        pub core_gpa_rank: String,
        pub core_weighted_rank: String,
        pub core_arithmetic_rank: String,
    }
}
pub async fn get_rank_from_ca(stu_id: &str) -> AppResult<ca::Rank> {
    let spider_res =
        infra::spider::ca::get_major_report(stu_id).await?;
    let regex = RegexBuilder::new(r"平均学分绩点排名 ([0-9/]+).*平均学分绩点 ([0-9.]+).*核心课程平均学分绩点排名 ([0-9/]+).*必修课平均学分绩点 ([0-9.]+).*课程算术平均成绩排名 ([0-9/]+).*算术平均分 ([0-9.]+).*核心课程算术平均成绩排名 ([0-9/]+).*必修课算术平均分 ([0-9.]+).*学分加权平均成绩排名 ([0-9/]+).*加权平均分 ([0-9.]+).*核心课程学分加权平均成绩排名 ([0-9/]+).*必修课加权平均分 ([0-9.]+)")
        .dot_matches_new_line(true)
        .build()
        .expect("构建正则表达式失败");
    let caps = regex
        .captures(&spider_res)
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
    let res = ca::Rank {
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
    let spider_res =
        infra::spider::hdjw::get_grade_detail(stu_id, jx0404id)
            .await?;
    let regex = RegexBuilder::new(
        r"let\sarr\s=\s(.*);.*window.initQzTable\(\{.*cols:\s\[(.*)\].*\}\);",
    )
    .dot_matches_new_line(true)
    .build()
    .expect("构建正则表达式失败");
    let caps = regex
        .captures(&spider_res)
        .ok_or(anyhow!("解析成绩详情数据失败"))?
        .iter()
        .map(|c| {
            c.map(|v| v.as_str().to_string())
                .ok_or(anyhow!("解析成绩详情数据失败: 字段为空"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let [_, data, map] = caps
        .try_into()
        .map_err(|_| anyhow!("解析成绩详情数据失败: 匹配数量错误"))?;
    let data = serde_json::from_str::<Vec<Value>>(&data).ok();
    let data = data.as_ref()
        .and_then(|v| v.first())
        .and_then(|v| v.as_object()).map(|v|
            v.iter().map(|(key, value)|{
                value.as_str().map(|s| s.to_string()).or(
                        value
                            .as_number().map(|num| num.to_string()),
                    ).ok_or(anyhow!("解析成绩详情数据失败: 字段不是字符串或数字"))
                    .map(|ok_value| (key, ok_value))
            }
            ).collect::<Result<HashMap<_, _>,_>>())
        .ok_or(anyhow!("解析成绩详情数据失败: data"))??;
    // map 是 js obj 格式，不是标准 json，我们需要进行一些处理
    let map = map
        .replace("//表头", "")
        .replace("'", "\"")
        .replace("field", "\"field\"")
        .replace("title", "\"title\"")
        .replace("type", "\"type\"");
    let map = serde_json::from_str::<Value>(map.as_str()).ok();
    let map = map
        .as_ref()
        .and_then(|v| v.as_array())
        .map(|v| {
            v.iter()
                .filter(|item| {
                    item.get("field")
                        .and_then(|f| f.as_str())
                        .is_some()
                })
                .map(|item| {
                    let key =
                        item.get("field").and_then(|f| f.as_str());
                    item.get("title")
                        .and_then(|f| f.as_str())
                        .and_then(|value| key.map(|key| (key, value)))
                        .ok_or(anyhow!("解析成绩详情数据失败: map"))
                })
                .collect::<Result<HashMap<_, _>, _>>()
        })
        .ok_or(anyhow!("解析成绩详情数据失败: map"))??;
    let res = data
        .iter()
        .filter(|(k, _)| k.ends_with("bl"))
        .map(|(k, v)| {
            let score = data
                .get(&k.trim_end_matches("bl").to_string())
                .ok_or(anyhow!(
                    "解析成绩详情数据失败: data 缺失 {}",
                    k.trim_end_matches("bl")
                ))?;
            let name = map.get(k.trim_end_matches("bl")).ok_or(
                anyhow!("解析成绩详情数据失败: map 缺失 {}", k),
            )?;
            let percentage = v;
            Ok::<_, anyhow::Error>(GradeDetailItem {
                score: score.to_string(),
                name: name.to_string(),
                percentage: percentage.to_string(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|item| item.percentage != "0%")
        .collect::<Vec<_>>();
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
