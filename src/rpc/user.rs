use hnu_query::xgxt::personal_info::PersonalInfo;
use tarpc::context;

use crate::{error::AppError, service, utils};

#[tarpc::service]
pub trait User {
    async fn get_user_info(
        jwt: String,
    ) -> Result<PersonalInfo, String>;
}

#[derive(Clone)]
pub struct UserServer;

impl User for UserServer {
    async fn get_user_info(
        self,
        _: context::Context,
        jwt: String,
    ) -> Result<PersonalInfo, String> {
        let stu_id = utils::jwt::parse(&jwt).map_err(|e| {
            tracing::error!(
                ?e,
                error_chain = utils::debug_error_chain(&e),
                "Failed to parse JWT"
            );
            AppError::unauthorized().to_string()
        })?;
        let user_info =
            service::user_info::get_person_info(&stu_id, false)
                .await
                .map_err(|e| e.to_string())?;
        Ok(user_info)
    }
}
