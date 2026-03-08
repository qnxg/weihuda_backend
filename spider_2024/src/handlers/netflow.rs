use anyhow::anyhow;
use salvo::{Request, handler};
use serde_json::{Value, json};

use crate::{
    app_result::HandlerResult,
    dtos::netflow::{NetflowDayDetailReq, NetflowMonthDetailReq},
    spiders,
};

#[handler]
pub async fn get_netflow_handler(req: &mut Request) -> HandlerResult {
    let stuid = req
        .query::<String>("stuid")
        .ok_or(anyhow!("stuid is required"))?;
    let mut res = spiders::netflow::get_netflow(&stuid).await?;
    let data =
        res["data"].as_object_mut().expect("netflow data不是对象");

    // 给流量数据加上单位
    // 流量字符串可能返回"小于0.01GB"，此时不重复添加单位

    fn try_add_suffix(s: &mut String, suffix: &str) {
        if !s.ends_with(suffix) {
            *s += suffix;
        }
    }

    fn try_add_gb_to_str_value(place: &mut Value) {
        let mut orig = place.as_str().unwrap().to_string();
        try_add_suffix(&mut orig, "GB");
        *place = Value::String(orig);
    }

    try_add_gb_to_str_value(&mut data["downloadTraffic"]);
    try_add_gb_to_str_value(&mut data["uploadTraffic"]);
    try_add_gb_to_str_value(&mut data["allTraffic"]);

    Ok(data.into())
}

#[handler]
pub async fn get_netflow_pay_info_handler(
    req: &mut Request,
) -> HandlerResult {
    let stuid = req
        .query::<String>("stuid")
        .ok_or(anyhow!("stuid is required"))?;
    let res = spiders::netflow::get_pay_status(&stuid).await?;
    let res = &res["data"];
    Ok(res.into())
}

#[handler]
pub async fn get_unlock_status_handler(
    req: &mut Request,
) -> HandlerResult {
    let stuid = req
        .query::<String>("stuid")
        .ok_or(anyhow!("stuid is required"))?;
    let res = spiders::netflow::get_user_status(&stuid).await?;
    let status = res["data"]["IsLocked"].as_i64().unwrap();
    let status = match status {
        0 => "未锁定",
        1 => "已锁定",
        _ => "未知",
    };
    let res = json!({
        "status": status,
    });
    Ok(res.into())
}

#[handler]
pub async fn get_netflow_month_detail_handler(
    req: &mut Request,
) -> HandlerResult {
    let req: NetflowMonthDetailReq = req.parse_queries()?;
    let res = spiders::netflow::get_netflow_month_detail(
        &req.stuid, &req.year, &req.month,
    )
    .await?;
    let res = &res["data"];
    Ok(res.into())
}

#[handler]
pub async fn get_netflow_day_detail_handler(
    req: &mut Request,
) -> HandlerResult {
    let req: NetflowDayDetailReq = req.parse_queries()?;
    let res = spiders::netflow::get_netflow_day_detail(
        &req.stuid, &req.year, &req.month, &req.day,
    )
    .await?;
    let res = &res["data"];
    Ok(res.into())
}

#[handler]
pub async fn get_netflow_order_handler(
    req: &mut Request,
) -> HandlerResult {
    let stuid = req
        .query::<String>("stuid")
        .ok_or(anyhow!("stuid is required"))?;
    let mut res = spiders::netflow::get_order(&stuid).await?;
    let res = res["data"].as_array_mut().unwrap();
    for item in res.iter_mut() {
        let upload = item["Upload"].as_f64().unwrap_or_default();
        let upload_name = if upload == 0.0 {
            Value::String("0 GB".to_string())
        } else {
            Value::String(format!(
                "{:.2} GB",
                upload / 1024.0 / 1024.0 / 1024.0
            ))
        };

        let download = item["Download"].as_f64().unwrap_or_default();
        let download_name = if download == 0.0 {
            Value::String("0 GB".to_string())
        } else {
            Value::String(format!(
                "{:.2} GB",
                download / 1024.0 / 1024.0 / 1024.0
            ))
        };

        item["UploadName"] = upload_name;
        item["DownloadName"] = download_name;
    }
    Ok(res.into())
}
