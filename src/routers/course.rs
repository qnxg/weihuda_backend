use crate::utils::serde::empty_string_as_none;
use salvo::{Request, Router, handler};
use serde::Deserialize;

use crate::{
    result::RouterResult,
    service::{self, course::CustomizeCourseInfo},
    utils,
};

pub fn routers() -> Router {
    Router::new()
        .push(Router::with_path("flex-time").get(get_flex_time))
        .push(
            Router::with_path("course")
                .post(add_course) // 添加自定义课表课程
                .delete(delete_course), // 删除自定义课表课程
        )
        .push(
            Router::with_path("hdjw/class-table").get(get_classtable),
        )
}

#[derive(Deserialize, Debug)]
#[expect(dead_code)]
struct GetCourseReq {
    pub xn: u32,
    pub xq: u32,
}
#[handler]
async fn get_customize_course(req: &mut Request) -> RouterResult {
    let GetCourseReq { xn, xq } = req.parse_queries()?;
    let (mini_bind_id, _) = utils::jwt::auth(req)?;

    let res = crate::service::course::get_customize_course(
        xn,
        xq,
        mini_bind_id,
    )
    .await?;

    Ok(res.into())
}

#[derive(Deserialize, Debug)]
struct AddCourseReq {
    pub classname: String,
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub location: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub teachers: Option<String>,
    pub week: String,
    pub section: String,
    pub day: String,
    pub xn: u32,
    pub xq: u32,
}
#[handler]
async fn add_course(req: &mut Request) -> RouterResult {
    let course: AddCourseReq = req.parse_json().await?;
    let (mini_bind_id, _) = utils::jwt::auth(req)?;
    service::course::add_customize_course(
        CustomizeCourseInfo {
            classname: course.classname,
            location: course.location,
            teachers: course.teachers,
            week: course.week,
            day: course.day,
            section: course.section,
            id: 0,
        },
        course.xn,
        course.xq,
        mini_bind_id,
    )
    .await?;
    Ok("添加成功".into())
}

#[derive(Deserialize, Debug)]
struct DeleteCourseReq {
    pub id: u32,
}
#[handler]
async fn delete_course(req: &mut Request) -> RouterResult {
    let DeleteCourseReq { id } = req.parse_queries()?;
    let (mini_bind_id, _) = utils::jwt::auth(req)?;
    service::course::delete_customize_course(id, mini_bind_id)
        .await?;
    Ok("删除成功".into())
}

#[handler]
async fn get_flex_time() -> RouterResult {
    let flex_time = service::course::get_flex_time_list().await?;
    Ok(flex_time.into())
}

/// 获取课表
#[derive(Deserialize, Debug)]
struct GetClasstableReq {
    pub xn: u32,
    pub xq: u32,
}
#[handler]
async fn get_classtable(req: &mut Request) -> RouterResult {
    let GetClasstableReq { xn, xq } = req.parse_queries()?;
    let (mini_bind_id, stu_id) = utils::jwt::auth(req)?;
    let classtable = service::course::get_classtable(
        &stu_id,
        mini_bind_id,
        xn,
        xq,
    )
    .await?;
    Ok(classtable.into())
}
