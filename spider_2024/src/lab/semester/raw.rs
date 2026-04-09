use crate::{lab::utils::request_lab, utils::client};
use anyhow::anyhow;
use serde::Deserialize;

const SEM_INFO_URL: &str =
    "http://10.62.106.112/Common/Common/GetSemDropDownList?HasNull=0";

#[derive(Deserialize, Debug)]
pub struct SemesterItem {
    pub id: String,
    pub text: String,
}

pub async fn raw_semester_data(
    stu_id: &str,
) -> Result<Vec<SemesterItem>, crate::Error> {
    let req = client.get(SEM_INFO_URL);
    let raw_res = request_lab(stu_id, req).await?;
    let res: Vec<SemesterItem> =
        serde_json::from_value(raw_res.clone()).map_err(|e| {
            anyhow!("解析数据失败 data = {}, err = {}", raw_res, e)
        })?;
    Ok(res)
}
