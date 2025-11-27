use super::get_db_pool;
use crate::result::AppResult;
use serde::{Deserialize, Serialize};

/// 调休的结构体
/// 将会将 from 的课程全部转移到 to 上去，且 to 的课程全部毙掉
#[derive(Debug, Serialize, Deserialize)]
pub struct FlexTime {
    // 如果这里是 None，表示仅 to 那天的课停上，不会有课程转移
    pub from: Option<FlexDay>,
    pub to: FlexDay,
    pub desc: String, // 描述，将会返回给前端用作展示
    pub time: XnXq,   // 学年学期
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FlexDay {
    pub week: u8, // 第几周
    pub day: u8,  // 星期几
}

#[derive(Debug, Serialize, Deserialize)]
pub struct XnXq {
    pub xn: u32, // 学年
    pub xq: u32, // 学期
}

pub async fn get_flex_time_list() -> AppResult<Vec<FlexTime>> {
    let flex_time = sqlx::query!(
        "SELECT value FROM mini_configs WHERE `key` = ? AND enabled = 1",
        "flexTime"
    )
    .fetch_one(get_db_pool().await)
    .await?
    .value;
    // 解析到json
    let flex_time: Vec<FlexTime> =
        serde_json::from_str(&flex_time)
            .map_err(|_| anyhow::anyhow!("解析调休信息失败"))?;
    Ok(flex_time)
}
