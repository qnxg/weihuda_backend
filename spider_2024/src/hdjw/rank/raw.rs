use anyhow::anyhow;
use serde_json::Value;

use crate::{hdjw::utils::request_hdjw, utils::client};

// 该 URL 缺少学期、课程类型，排名方式的参数，需要后续再用 format 拼接
const GRADE_RANK_URL: &str = "http://hdjw.hnu.edu.cn/jsxsd/xscjsq/cjpmcx_list.do?&pageNum=1&pageSize=20&kclx=&kcly=1";

pub async fn raw_rank_data(
    stu_id: &str,
    selection: &str,
    range: &str,
    rank_method: &str,
) -> Result<Option<Value>, crate::Error> {
    let req = client.get(format!(
        "{}&xnxq={}&kkxz={}&pmfs={}",
        GRADE_RANK_URL, selection, range, rank_method
    ));
    let res = request_hdjw(stu_id, req).await?;
    let Some(data) = res["data"].as_array() else {
        return Err(anyhow!("解析排名数据失败: {:?}", res).into());
    };
    Ok(data.first().cloned())
}
