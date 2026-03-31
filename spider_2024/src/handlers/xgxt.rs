use crate::{app_result::AppResult, dtos::xgxt::PersonInfo, spiders};

/// TODO：学工系统获取个人信息的传输的数据量比较大，可能会成为性能瓶颈
pub async fn get_person_info_handler(
    stu_id: &str,
) -> AppResult<PersonInfo> {
    let res = spiders::xgxt::get_person_info(stu_id).await?;
    Ok(res)
}
