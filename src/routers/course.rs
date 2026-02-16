use crate::utils::serde::empty_string_as_none;
use salvo::{Request, Router, handler, macros::Extractible};
use serde::Deserialize;

use crate::{
    result::RouterResult,
    service::{self, course::CustomizeCourseInfo},
    utils,
};

pub fn routers() -> Router {
    Router::new()
        .push(
            Router::with_path("course")
                .post(add_course) // 添加自定义课表课程
                .delete(delete_course), // 删除自定义课表课程
        )
        .push(
            Router::with_path("hdjw/class-table").get(get_classtable),
        )
}

#[handler]
async fn add_course(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "body")))]
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
    let course: AddCourseReq = req.extract().await?;
    let stu_id = utils::jwt::auth(req)?;
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
        &stu_id,
    )
    .await?;
    Ok("添加成功".into())
}

#[handler]
async fn delete_course(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct DeleteCourseReq {
        pub id: u32,
    }
    let DeleteCourseReq { id } = req.extract().await?;
    let stu_id = utils::jwt::auth(req)?;
    service::course::delete_customize_course(id, &stu_id).await?;
    Ok("删除成功".into())
}

/// 获取课表
#[handler]
async fn get_classtable(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct GetClasstableReq {
        pub xn: u32,
        pub xq: u32,
    }
    let GetClasstableReq { xn, xq } = req.extract().await?;
    let stu_id = utils::jwt::auth(req)?;
    let classtable =
        service::course::get_classtable(&stu_id, xn, xq).await?;
    Ok(classtable.into())
}
