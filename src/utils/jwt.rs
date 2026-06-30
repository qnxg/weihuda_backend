use jsonwebtoken::{
    DecodingKey, EncodingKey, Header, Validation, decode, encode,
};
use salvo::Request;
use serde::{Deserialize, Serialize};
use std::{
    sync::LazyLock,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    config::CFG,
    result::{AppError, AppResult},
    utils,
};

#[derive(Serialize, Deserialize, Debug)]
struct Claims {
    iss: String,
    exp: usize,
    sub: String,
    iat: usize,
    platform: u8,
    stu_id: String,
}

static VALIDATION: LazyLock<Validation> = LazyLock::new(|| {
    let mut validation = Validation::default();
    validation.validate_exp = false;
    validation
});

/// 用mini_bind_id和stu_id生成token
pub fn generate_jwt(stu_id: &str) -> AppResult<String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as usize;

    let claims = Claims {
        iss: "qnxg".to_string(),
        exp: now + 60 * 60 * 24 * 90,
        sub: "mini-jwt".to_string(),
        iat: now,
        platform: 0,
        stu_id: stu_id.to_string(),
    };

    let res = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(CFG.secret.jwt.as_bytes()),
    )?;

    Ok(res)
}

pub fn parse(token: &str) -> AppResult<String> {
    let res = decode::<Claims>(
        token,
        &DecodingKey::from_secret(CFG.secret.jwt.as_bytes()),
        &VALIDATION,
    )?;
    let stu_id = utils::format_stuid(&res.claims.stu_id);
    Ok(stu_id)
}

/// 如果验证失败就返回 AppError::Unauthorized
/// 验证成功则返回用户的 stu_id
/// 不验证 token 是否过期，以及 stu_id 是否存在
pub fn auth(req: &mut Request) -> AppResult<String> {
    let jwt = req
        .headers()
        .get("Authorization")
        .ok_or(AppError::Unauthorized)?
        .to_str()
        .map_err(|_| AppError::Unauthorized)?;
    parse(jwt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let token = "";
        let stu_id = parse(token).unwrap();
        assert_eq!(stu_id, "");
    }

    #[test]
    fn test_auth() {
        let stu_id = "";
        let token = generate_jwt(stu_id).unwrap();
        let res_stu_id = parse(&token).unwrap();
        assert_eq!(stu_id, res_stu_id);
    }
}
