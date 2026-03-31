use crate::result::AppResult;

/// 获取主修的中文成绩单
/// 返回的文本是对成绩单提取文本后的结果
pub async fn get_major_report(stu_id: &str) -> AppResult<String> {
    let spider_res =
        spider_2024::hdjw::get_grade_from_ca_handler(stu_id).await?;
    Ok(spider_res)
}
