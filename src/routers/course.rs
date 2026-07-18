use crate::{
    error::RouterResult,
    routers::{
        ThrowParseError,
        demo::{DEMO_COURSE_ID, DEMO_COURSE_NAME, DEMO_STU_ID},
    },
    service::{
        self,
        course::{CourseInfo, CustomizeCourseInfo},
    },
    utils::{self, serde::empty_string_as_none},
};
use salvo::{Request, Router, handler, macros::Extractible};
use serde::Deserialize;

pub fn routers() -> Router {
    Router::new()
        .push(
            Router::with_path("course")
                .post(add_course) // 添加自定义课表课程
                .put(update_course) //添加修改自定义课表课程
                .delete(delete_course) // 删除自定义课表课程
                .push(
                    Router::with_path("get-custom-details-by-id")
                        .get(get_custom_course_details_by_id), // 根据id查询自定义课程详情
                ),
        )
        .push(
            Router::with_path("hdjw/class-table").get(get_classtable),
        )
        .push(
            Router::with_path("hdjw/class-table-extra")
                .get(get_extra_course),
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
    let course: AddCourseReq = req.extract().await.parse_error()?;
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
    let DeleteCourseReq { id } = req.extract().await.parse_error()?;
    let stu_id = utils::jwt::auth(req)?;
    service::course::delete_customize_course(id, &stu_id).await?;
    Ok("删除成功".into())
}

#[handler]
async fn get_custom_course_details_by_id(
    req: &mut Request,
) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct GetCustomCourseDetailsReq {
        pub id: u32,
    }
    let GetCustomCourseDetailsReq { id } =
        req.extract().await.parse_error()?;
    let stu_id = utils::jwt::auth(req)?;
    let details =
        service::course::get_custom_course_details_by_id(id, &stu_id)
            .await?;
    Ok(details.into())
}

/// 更新自定义课程
#[handler]
async fn update_course(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "body")))]
    struct UpdateCourseReq {
        pub id: u32,
        pub classname: String,
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub location: Option<String>,
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub teachers: Option<String>,
        pub week: String,
        pub day: String,
        pub section: String,
    }

    let body: UpdateCourseReq = req.extract().await.parse_error()?;
    let stu_id = utils::jwt::auth(req)?;
    service::course::update_customize_course(
        body.id,
        &stu_id,
        CustomizeCourseInfo {
            classname: body.classname,
            location: body.location,
            teachers: body.teachers,
            week: body.week,
            day: body.day,
            section: body.section,
            id: body.id,
        },
    )
    .await?;
    Ok("更新成功".into())
}

/// 获取课表
#[handler]
async fn get_classtable(req: &mut Request) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    if stu_id == DEMO_STU_ID {
        return Ok(vec![CourseInfo {
            course_name: DEMO_COURSE_NAME.to_string(),
            course_id: Some(DEMO_COURSE_ID.to_string()),
            _type: "必修".to_string(),
            class_name: None,
            place: None,
            area: None,
            teacher: None,
            weeks: vec![
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
            ],
            day: 2,
            time: 2,
            credit: Some(2.0),
            extra: None,
            customize_id: -1,
            people: 42,
        }]
        .into());
    }

    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct GetClasstableReq {
        pub xn: u16,
        pub xq: u8,
    }
    let GetClasstableReq { xn, xq } =
        req.extract().await.parse_error()?;
    let classtable =
        service::course::get_classtable(&stu_id, xn, xq).await?;
    Ok(classtable.into())
}

#[handler]
async fn get_extra_course(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct GetExtraCourseReq {
        pub xn: u16,
        pub xq: u8,
    }
    let GetExtraCourseReq { xn, xq } =
        req.extract().await.parse_error()?;
    let stu_id = utils::jwt::auth(req)?;
    let extra_course =
        service::course::get_extra_course(&stu_id, xn, xq).await?;
    Ok(extra_course.into())
}
