use salvo::{Request, handler};

use crate::{app_result::HandlerResult, spiders};

#[handler]
pub async fn get_lab_list_handler(
    req: &mut Request,
) -> HandlerResult {
    let stuid = req
        .query::<String>("stuid")
        .ok_or(anyhow::anyhow!("stuid is required"))?;
    let res = spiders::lab::get_lab_list(&stuid).await?;
    if res.is_null() {
        return Ok(res.into());
    }
    let res = res["rows"]
        .as_array()
        .ok_or(anyhow::anyhow!("意料之外的数据 {}", res))?;
    Ok(res.into())
}

#[handler]
pub async fn check_lab_password_handler(
    req: &mut Request,
) -> HandlerResult {
    let stuid: String = req
        .query("stuid")
        .ok_or(anyhow::anyhow!("stuid is required"))?;
    let password: String = req
        .query("password")
        .ok_or(anyhow::anyhow!("password is required"))?;
    let (res, _) =
        spiders::lab::check_password(&stuid, &password).await?;
    Ok(res.into())
}

#[handler]
pub async fn get_lab_sem_info_handler(
    req: &mut Request,
) -> HandlerResult {
    let stuid: String = req
        .query("stuid")
        .ok_or(anyhow::anyhow!("stuid is required"))?;
    let res = spiders::lab::get_sem_info(&stuid).await?;
    Ok(res.into())
}

#[handler]
pub async fn get_lab_course_list_handler(
    req: &mut Request,
) -> HandlerResult {
    let stuid: String = req
        .query("stuid")
        .ok_or(anyhow::anyhow!("stuid is required"))?;
    let sem: String =
        req.query("sem").ok_or(anyhow::anyhow!("sem is required"))?;
    let res = spiders::lab::get_course_list(&stuid, &sem).await?;
    if res.is_null() {
        return Ok(res.into());
    }
    let res = res["rows"]
        .as_array()
        .ok_or(anyhow::anyhow!("意料之外的数据 {}", res))?;
    Ok(res.into())
}

#[handler]
pub async fn get_lab_score_handler(
    req: &mut Request,
) -> HandlerResult {
    let stuid: String = req
        .query("stuid")
        .ok_or(anyhow::anyhow!("stuid is required"))?;
    let course_id: String = req
        .query("course_id")
        .ok_or(anyhow::anyhow!("course_id is required"))?;
    let sem: String =
        req.query("sem").ok_or(anyhow::anyhow!("sem is required"))?;
    let res =
        spiders::lab::get_lab_score(&stuid, &sem, &course_id).await?;
    if res.is_null() {
        return Ok(res.into());
    }
    let res = res["rows"]
        .as_array()
        .ok_or(anyhow::anyhow!("意料之外的数据 {}", res))?;
    Ok(res.into())
}

#[handler]
pub async fn get_virtual_lab_score_handler(
    req: &mut Request,
) -> HandlerResult {
    let stuid: String = req
        .query("stuid")
        .ok_or(anyhow::anyhow!("stuid is required"))?;
    let res = spiders::lab::get_virtual_lab_score(&stuid).await?;
    if res.is_null() {
        return Ok(res.into());
    }
    let res = res["rows"]
        .as_array()
        .ok_or(anyhow::anyhow!("意料之外的数据 {}", res))?;
    Ok(res.into())
}

#[handler]
pub async fn get_lab_score_structure_handler(
    req: &mut Request,
) -> HandlerResult {
    let stuid: String = req
        .query("stuid")
        .ok_or(anyhow::anyhow!("stuid is required"))?;
    let course_id: String = req
        .query("course_id")
        .ok_or(anyhow::anyhow!("course_id is required"))?;
    let res =
        spiders::lab::get_score_structure(&stuid, &course_id).await?;
    if res.is_null() {
        return Ok(res.into());
    }
    let res = res["Data"]
        .as_array()
        .ok_or(anyhow::anyhow!("意料之外的数据 {}", res))?;
    Ok(res.into())
}

#[handler]
pub async fn get_lab_score_detail_handler(
    req: &mut Request,
) -> HandlerResult {
    let stuid: String = req
        .query("stuid")
        .ok_or(anyhow::anyhow!("stuid is required"))?;
    let course_id: String = req
        .query("course_id")
        .ok_or(anyhow::anyhow!("course_id is required"))?;
    let res =
        spiders::lab::get_score_detail(&stuid, &course_id).await?;
    if res.is_null() {
        return Ok(res.into());
    }
    let res = res
        .get("Data")
        .ok_or(anyhow::anyhow!("意料之外的数据 {}", res))?;
    let res = res["Lablist"]
        .as_array()
        .ok_or(anyhow::anyhow!("意料之外的数据 {}", res))?;
    Ok(res.into())
}
