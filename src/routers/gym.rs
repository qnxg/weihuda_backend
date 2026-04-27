use salvo::{Request, Router, handler, macros::Extractible};
use serde::Deserialize;

use crate::{
    result::{AppError, RouterResult},
    service, utils,
};

pub fn routers() -> Router {
    Router::with_path("pt")
        .push(Router::with_path("fitness").get(get_fitness_grade))
        .push(
            Router::with_path("fitness-appoint")
                .get(get_fitness_appoint),
        )
}

#[handler]
async fn get_fitness_grade(req: &mut Request) -> RouterResult {
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct GetFitnessReq {
        pub xn: String,
    }
    let stu_id = utils::jwt::auth(req)?;
    let GetFitnessReq { xn } = req.extract().await?;
    let xn = xn.parse::<u16>().map_err(|_| AppError::ParseError)?;
    let res = service::gym::get_fitness_grade(&stu_id, xn).await?;
    Ok(res.into())
}

#[handler]
async fn get_fitness_appoint(req: &mut Request) -> RouterResult {
    let stu_id = utils::jwt::auth(req)?;
    let res = service::gym::get_fitness_appoint(&stu_id).await?;
    Ok(res.into())
}
