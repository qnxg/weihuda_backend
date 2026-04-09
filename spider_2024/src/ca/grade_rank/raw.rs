use anyhow::anyhow;
use serde_json::Value;
use std::time::Duration;

use crate::{ca::login::ca_headers, utils::client};

/// 本科生主修所有课程的中文成绩单
pub const UNDERGRADUATE_MAJOR_ALL_TEMPLATE_ID: &str =
    "02a70e11bc89b40dc2ef6ed14851ce25";

/// 获取可信电子凭证文件的文本原始数据
///
/// # Arguments
///
/// - `stu_id`: 学号
/// - `template_id`: 模板 id
///
/// # Returns
///
/// 可信电子凭证文件的 pdf 文本原始数据
pub async fn raw_certification_data(
    stu_id: &str,
    template_id: &str,
) -> Result<String, crate::Error> {
    let ca_headers = ca_headers(stu_id).await?;
    let template_url = format!(
        "https://ca.hnu.edu.cn/student/student/caTemplate/preview_file?templateId={}&isbzf=0&kcxz=&xfjd=&xzkc=",
        template_id
    );
    let res: Value = client
        .get(&template_url)
        .timeout(Duration::from_secs(60))
        .headers(ca_headers.clone())
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    if res.get("code").and_then(|v| v.as_u64()) != Some(200) {
        return Err(anyhow!("获取文件失败").into());
    }
    let Some(file_name) = res.get("message").and_then(|v| v.as_str())
    else {
        return Err(anyhow!("获取文件失败").into());
    };
    let file_url = format!(
        "https://ca.hnu.edu.cn/student/sys/common/view/{}",
        file_name
    );
    // 下载文件
    let res = client
        .get(&file_url)
        .timeout(Duration::from_secs(60))
        .headers(ca_headers)
        .send()
        .await?
        .error_for_status()?;
    let bytes = res.bytes().await?;
    let pdf = pdf_extract::extract_text_from_mem(&bytes).unwrap();
    Ok(pdf)
}
