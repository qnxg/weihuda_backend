use std::time::Duration;

use crate::{
    infra::spider::spider_data_with_timeout, result::AppResult,
};

/// 获取主修的中文成绩单
/// 返回的文本是对成绩单提取文本后的结果
pub async fn get_major_report(stu_id: &str) -> AppResult<String> {
    let params = [("stuid", stu_id)];
    let spider_res: String = spider_data_with_timeout(
        "/bks/grade-from-ca",
        &params,
        Duration::from_secs(60),
    )
    .await?;
    Ok(spider_res)
}
