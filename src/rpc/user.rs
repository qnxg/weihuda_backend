use hnu_query::xgxt::personal_info::{
    Gender as SourceGender, Level as SourceLevel,
    PersonalInfo as SourcePersonalInfo,
};
use volo::FastStr;
use volo_thrift::{MaybeException, ServerError};

use crate::{error::AppError, service, utils};
use volo_gen::weihuda::rpc::{
    Dormitory, Gender, Level, PersonalInfo, RpcError, UserService,
    UserServiceGetUserInfoException,
};

#[derive(Clone, Copy)]
pub struct UserServer;

impl UserService for UserServer {
    async fn get_user_info(
        &self,
        jwt: FastStr,
    ) -> Result<
        MaybeException<PersonalInfo, UserServiceGetUserInfoException>,
        ServerError,
    > {
        let stu_id = match utils::jwt::parse(&jwt) {
            Ok(stu_id) => stu_id,
            Err(e) => {
                tracing::error!(
                    ?e,
                    error_chain = utils::debug_error_chain(&e),
                    "Failed to parse JWT"
                );
                return Ok(rpc_error(
                    AppError::unauthorized().to_string(),
                ));
            }
        };
        let user_info =
            match service::user_info::get_person_info(&stu_id, false)
                .await
            {
                Ok(user_info) => user_info,
                Err(e) => return Ok(rpc_error(e.to_string())),
            };
        match convert_personal_info(user_info) {
            Ok(user_info) => Ok(MaybeException::Ok(user_info)),
            Err(e) => Ok(rpc_error(e)),
        }
    }
}

fn rpc_error(
    message: String,
) -> MaybeException<PersonalInfo, UserServiceGetUserInfoException> {
    MaybeException::Exception(UserServiceGetUserInfoException::Error(
        RpcError {
            message: message.into(),
        },
    ))
}

fn convert_personal_info(
    value: SourcePersonalInfo,
) -> Result<PersonalInfo, String> {
    let enter_year =
        i16::try_from(value.enter_year).map_err(|e| e.to_string())?;
    let dormitory =
        value.dormitory.as_ref().map(|dormitory| Dormitory {
            park: dormitory
                .park()
                .map(|park| park.to_string().into()),
            build: dormitory
                .build()
                .map(|build| build.to_string().into()),
            room: dormitory.room().to_string().into(),
            raw_dormitory: dormitory
                .raw_dormitory()
                .to_string()
                .into(),
        });
    Ok(PersonalInfo {
        name: value.name.into(),
        enter_year,
        xz: value.xz.map(i16::from),
        stu_id: value.stu_id.into(),
        gender: match value.gender {
            SourceGender::Male => Gender::MALE,
            SourceGender::Female => Gender::FEMALE,
        },
        level: match value.level {
            SourceLevel::Undergraduate => Level::UNDERGRADUATE,
            SourceLevel::Postgraduate => Level::POSTGRADUATE,
            SourceLevel::Doctoral => Level::DOCTORAL,
        },
        academy: value.academy.into(),
        major: value.major.into(),
        class: value.class.into(),
        dormitory,
        politic: value.politic.map(Into::into),
        race: value.race.map(Into::into),
        hometown: value.hometown.map(Into::into),
        phone: value.phone.map(Into::into),
        wechat: value.wechat.map(Into::into),
        qq: value.qq.map(Into::into),
        email: value.email.map(Into::into),
    })
}
