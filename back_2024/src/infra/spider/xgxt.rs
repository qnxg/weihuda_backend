use crate::result::AppResult;
use spider_2024::dtos::xgxt::PersonInfo;

pub async fn get_person_info(stu_id: &str) -> AppResult<PersonInfo> {
    let spider_res =
        spider_2024::xgxt::get_person_info_handler(stu_id).await?;
    Ok(spider_res)
}
