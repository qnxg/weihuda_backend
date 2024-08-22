use crate::app_error::AppError;
use crate::app_result::AppResult;
use crate::dtos::back::survey::PostQueryResultReq;
use crate::extractors::Json;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

pub async fn post_query_result_handler(Json(req): Json<PostQueryResultReq>) -> AppResult {
    // 读取当前目录的临时文件，如果存在将其中内容读取成json解释到结构体
    // 将req中的results与json中的内容进行合并
    // 将合并后的内容写入临时文件
    // 返回成功
    let file_path = "temp_file.json";
    let mut results: Vec<PostQueryResultReq> = vec![];

    if let Ok(mut file) = File::open(file_path) {
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| anyhow::anyhow!("读取文件失败, {e}"))?;
        results =
            serde_json::from_str(&content).map_err(|e| AppError::JsonError(e.to_string()))?;
    }
    results.push(req);

    let merged_content =
        serde_json::to_string(&results).map_err(|e| AppError::JsonError(e.to_string()))?;
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(file_path)
        .map_err(|e| anyhow::anyhow!("打开文件失败, {e}"))?;
    file.write_all(merged_content.as_bytes())
        .map_err(|e| anyhow::anyhow!("写入文件失败, {e}"))?;

    Ok("提交成功".into())
}
