use salvo::{Request, Router, handler};
use serde::Deserialize;

use crate::{result::RouterResult, service, utils};

pub fn routers() -> Router {
    Router::with_path("pt")
        .push(Router::with_path("fitness").get(get_fitness_grade))
        .push(
            Router::with_path("fitness-appoint")
                .get(get_fitness_appoint),
        )
}

#[derive(Deserialize, Debug)]
struct GetFitnessReq {
    pub xn: String,
}
#[handler]
async fn get_fitness_grade(req: &mut Request) -> RouterResult {
    let (_, stu_id) = utils::jwt::auth(req)?;
    let GetFitnessReq { xn } = req.parse_queries()?;
    let res = service::gym::get_fitness_grade(&stu_id, &xn).await?;
    Ok(res.into())
}

#[handler]
async fn get_fitness_appoint(req: &mut Request) -> RouterResult {
    let (_, stu_id) = utils::jwt::auth(req)?;
    let res = service::gym::get_fitness_appoint(&stu_id).await?;
    Ok(res.into())
}
