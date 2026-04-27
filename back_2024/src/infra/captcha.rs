use serde::Deserialize;
use spider_2024::lab::login::CaptchaResolver;

use crate::config::CFG;

#[derive(Deserialize)]
struct CaptchaResponse {
    error: Option<String>,
    result: Option<String>,
}

pub struct LabCaptchaResolver;

impl CaptchaResolver for LabCaptchaResolver {
    async fn resolve(
        &self,
        data: &[u8],
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>
    {
        let url = format!(
            "{}/ocr?type={}",
            CFG.captcha.captcha_url, "default"
        );
        let form = reqwest::multipart::Form::new().part(
            "file",
            reqwest::multipart::Part::bytes(data.to_vec())
                .file_name("captcha.jpg"),
        );
        let client = reqwest::Client::new();
        let res = client.post(&url).multipart(form).send().await?;
        let res = res.error_for_status()?;
        let body = res.text().await?;
        let res: CaptchaResponse = serde_json::from_str(&body)?;
        if let Some(result) = res.result {
            Ok(result)
        } else {
            Err(format!(
                "验证码服务错误: {}",
                res.error.unwrap_or("未知错误".to_string())
            )
            .into())
        }
    }
}
