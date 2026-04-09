use anyhow::anyhow;
use serde_json::Value;
use std::collections::HashMap;
use tokio::try_join;

use crate::{utils::client, xgxt::login::xgxt_headers};

// 个人信息
const USER_INFO_URL: &str = "https://xgxt.hnu.edu.cn/zftal-xgxt-web/dynamic/form/group/userInfo/default.zf?dataId=null";
// 在校信息
const IN_SCHOOL_INFO_URL: &str = "https://xgxt.hnu.edu.cn/zftal-xgxt-web/dynamic/form/group/zxxx/default.zf?dataId=null";
// 联系方式
const CONTACT_INFO_URL: &str = "https://xgxt.hnu.edu.cn/zftal-xgxt-web/dynamic/form/group/lxfs1/default.zf?dataId=null";

pub async fn raw_person_info_data(
    stu_id: &str,
) -> Result<HashMap<String, String>, crate::Error> {
    let xgxt_headers = xgxt_headers(stu_id).await?;
    let user_info_req = client
        .get(USER_INFO_URL)
        .headers(xgxt_headers.clone())
        .send();
    let in_school_info_req = client
        .get(IN_SCHOOL_INFO_URL)
        .headers(xgxt_headers.clone())
        .send();
    let contact_info_req =
        client.get(CONTACT_INFO_URL).headers(xgxt_headers).send();
    let res = try_join!(
        user_info_req,
        in_school_info_req,
        contact_info_req
    )?;
    // 将三部分请求拿到的数据收集起来
    let mut entries = HashMap::<String, String>::new();
    for i in [res.0, res.1, res.2] {
        let data = i.error_for_status()?.json::<Value>().await?;
        data.get("data")
            .and_then(|data| data.get("groupFields"))
            .and_then(|group_field_list| group_field_list.get(0))
            .and_then(|group_field_item| {
                group_field_item.get("fields")
            })
            .and_then(|fields| fields.as_array())
            .ok_or(anyhow!("解析数据失败, data: {:?}", data))?
            .iter()
            .for_each(|field| {
                if let Some(field_name) = field.get("fieldName")
                    && let Some(value) = field.get("defaultValue")
                {
                    let Some(field_name) = field_name.as_str() else {
                        return;
                    };
                    // 这个 value 是什么类型不好说，所以分别考虑
                    if let Some(v) = value.as_str() {
                        entries.insert(
                            field_name.to_string(),
                            v.to_string(),
                        );
                    } else if let Some(v) = value.as_i64() {
                        entries.insert(
                            field_name.to_string(),
                            v.to_string(),
                        );
                    }
                }
            })
    }
    Ok(entries)
}
