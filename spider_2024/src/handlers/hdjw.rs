use anyhow::anyhow;
use chrono::Datelike;
use salvo::{Request, handler};
use serde_json::json;

use crate::{
    app_result::HandlerResult,
    dtos::hdjw::{EmptyRoomReq, GradeReq, HdjwGradeRankReq},
    spiders::{self},
};

#[handler]
pub async fn get_grade_handler(req: &mut Request) -> HandlerResult {
    let req: GradeReq = req.parse_queries()?;
    let res =
        spiders::hdjw::get_grade(&req.stuid, req.xn, req.xq).await?;
    let res = &res["data"]; // 取里面的data字段返回，引用减少开销
    Ok(res.into())
}

#[handler]
pub async fn get_empty_classroom_handler(
    req: &mut Request,
) -> HandlerResult {
    let req: EmptyRoomReq = req.parse_queries()?;
    let res = spiders::hdjw::get_empty_classroom(
        &req.stuid,
        req.xn,
        req.xq,
        &req.week,
        req.day,
        &req.jc,
        &req.build_id,
    )
    .await?;
    Ok(res.into())
}

#[handler]
pub async fn get_class_table_handler(
    req: &mut Request,
) -> HandlerResult {
    let req: GradeReq = req.parse_queries()?;
    // 如果学号第一个是S或者B，就是属于研究生系统
    // TODO： 研究生系统
    if req.stuid.starts_with('S') || req.stuid.starts_with('B') {
        let res = spiders::graduate::get_class_table(
            &req.stuid, req.xn, req.xq,
        )
        .await?;
        return Ok(res.into());
    }
    let res =
        spiders::hdjw::get_class_table(&req.stuid, req.xn, req.xq)
            .await?;
    match res.get("count").and_then(|c| c.as_u64()) {
        None => Err(anyhow!("获取课表数据失败").into()),
        Some(0) => Ok(json!([]).into()), // 有可能 count 是 0 但是不带 data 字段
        Some(_) => Ok(res
            .get("data")
            .ok_or(anyhow!("获取课表数据失败"))?
            .into()),
    }
}

#[handler]
pub async fn get_exam_schedule_handler(
    req: &mut Request,
) -> HandlerResult {
    let req: GradeReq = req.parse_queries()?; // 复用结构体，因为字段完成相同
    let res =
        spiders::hdjw::get_exam_schedule(&req.stuid, req.xn, req.xq)
            .await?;
    let res = res.get("data").ok_or(anyhow!("获取考试安排失败"))?;
    Ok(res.into())
}

#[handler]
pub async fn get_rank_from_hdjw_handler(
    req: &mut Request,
) -> HandlerResult {
    let req: HdjwGradeRankReq = req.parse_queries()?;
    let enter_year = req.stuid[0..4]
        .parse::<u16>()
        .map_err(|_| anyhow!("暂时仅支持本科生"))?;
    let mut selection = Vec::new();
    if let Some(year) = req.year {
        if let Some(term) = req.term {
            selection.push(format!("{}-{}-{}", year, year + 1, term));
        } else {
            selection.push(format!("{}-{}-1", year, year + 1));
            selection.push(format!("{}-{}-2", year, year + 1));
            selection.push(format!("{}-{}-3", year, year + 1));
        }
    } else {
        // 从入学年份查到当前年份，多查了没关系
        let current_year = chrono::Local::now().year() as u16;
        for i in enter_year..=current_year {
            selection.push(format!("{}-{}-1", i, i + 1));
            selection.push(format!("{}-{}-2", i, i + 1));
            selection.push(format!("{}-{}-3", i, i + 1));
        }
    }
    let range = match req.course {
        1 => {
            "01,02,03,04,05,06,07,08,09,10,11,12,13,14,15,16,17,18,88"
        }
        2 => "01,02,03,04,08,10,11,12,16",
        3 => {
            if enter_year >= 2024 {
                // 2024 级开始实行不同的核心课方案
                "03,16"
            } else {
                "08,12,16"
            }
        }
        _ => return Err(anyhow!("course参数错误").into()),
    };
    let rank = match req.rank {
        1 => 4,
        2 => 2,
        3 => 3,
        _ => return Err(anyhow!("rank参数错误").into()),
    };
    let res = spiders::hdjw::get_grade_rank_common(
        &req.stuid,
        &selection,
        range.to_string(),
        rank,
    )
    .await?;
    let res = json!({
        "score": res.1,
        "rank": res.0
    });
    Ok(res.into())
}

#[handler]
pub async fn get_grade_from_ca_handler(
    req: &mut Request,
) -> HandlerResult {
    let stuid = req
        .query::<String>("stuid")
        .ok_or(anyhow!("stuid is required"))?;
    let res = spiders::hdjw::get_grade_from_ca(&stuid).await?;
    Ok(res.into())
}

#[handler]
pub async fn get_grade_detail_handler(
    req: &mut Request,
) -> HandlerResult {
    let stuid = req
        .query::<String>("stuid")
        .ok_or(anyhow!("stuid is required"))?;
    let jx0404id = req
        .query::<String>("jx0404id")
        .ok_or(anyhow!("jx0404id is required"))?;
    let res =
        spiders::hdjw::get_grade_detail(&stuid, &jx0404id).await?;
    Ok(res.into())
}

#[handler]
pub async fn get_class_table_extra_handler(
    req: &mut Request,
) -> HandlerResult {
    let req: GradeReq = req.parse_queries()?;
    let res = spiders::hdjw::get_class_table_extra(
        &req.stuid, req.xn, req.xq,
    )
    .await?;
    match res.get("count").and_then(|c| c.as_u64()) {
        None => Err(anyhow!("获取无课表课程失败").into()),
        Some(0) => Ok(json!([]).into()), // 有可能 count 是 0 但是不带 data 字段
        Some(_) => Ok(res
            .get("data")
            .ok_or(anyhow!("获取无课表课程失败"))?
            .into()),
    }
}
