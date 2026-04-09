use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use crate::{hdjw::utils::request_hdjw, utils::client};

// 该 URL 缺少学期的参数，需要后续再用 format 拼接
const EXAM_SCHEDULE_URL: &str = "http://hdjw.hnu.edu.cn/jsxsd/xsks/xsksap_list?pageNum=1&pageSize=20&xqlb=";

/// 考试安排单项
/// 带 Option 的字段应该是类似于体育理论这样考试安排信息很不全的课程
#[derive(Serialize, Deserialize, Debug)]
pub struct ExamScheduleItem {
    /// 课程代码
    pub kch: String,
    /// 课程名称
    pub kskcmc: String,
    /// 考试校区
    pub ksxq: Option<String>,
    /// 考试的教室
    pub js_mc: Option<String>,
    /// 考试时间（已经是一个时间区间了）
    pub kssj: Option<String>,
    /// 座位号
    pub zwh: Option<String>,
}

pub async fn raw_exam_schedule_data(
    stu_id: &str,
    xn: u16,
    xq: u8,
) -> Result<Vec<ExamScheduleItem>, crate::Error> {
    let req = client.get(format!(
        "{}&xnxqid={}-{}-{}",
        EXAM_SCHEDULE_URL,
        xn,
        xn + 1,
        xq
    ));
    let raw_data = request_hdjw(stu_id, req).await?;
    let res = raw_data
        .get("data")
        .ok_or(anyhow!("解析考试安排数据失败: {:?}", raw_data))?;
    let res: Vec<ExamScheduleItem> =
        serde_json::from_value(res.clone())
            .map_err(|e| anyhow!("解析考试安排数据失败: {}", e))?;
    Ok(res)
}
