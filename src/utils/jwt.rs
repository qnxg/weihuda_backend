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
};

#[derive(Serialize, Deserialize, Debug)]
struct Claims {
    iss: String,
    exp: usize,
    sub: String,
    iat: usize,
    platform: u8,
    id: u32,
    stu_id: String,
}

static VALIDATION: LazyLock<Validation> = LazyLock::new(|| {
    let mut validation = Validation::default();
    validation.validate_exp = false;
    validation
});

/// 用mini_bind_id和stu_id生成token
pub fn generate_jwt(id: u32, stu_id: &str) -> AppResult<String> {
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
        id,
        stu_id: stu_id.to_string(),
    };

    let res = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(CFG.jwt.secret.as_bytes()),
    )?;

    Ok(res)
}

pub fn parse(token: &str) -> AppResult<(u32, String)> {
    let res = decode::<Claims>(
        token,
        &DecodingKey::from_secret(CFG.jwt.secret.as_bytes()),
        &VALIDATION,
    )?;

    Ok((res.claims.id, res.claims.stu_id))
}

/// 如果验证失败就返回 AppError::Unauthorized
pub fn auth(req: &mut Request) -> AppResult<(u32, String)> {
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
        let token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJxbnhnIiwiZXhwIjoxNzE2ODA1MzgxLCJzdWIiOiJtaW5pLWp3dCIsImlhdCI6MTcwOTAyOTM4MSwicGxhdGZvcm0iOjAsImlkIjo0NDk3MSwic3R1X2lkIjoiMjAyMTA0MDYxMzE0In0.xfG3LjhZPgKSstoVKy4ISvp6ZgwJrfjURK2SSipbBTc";
        let (id, stu_id) = parse(token).unwrap();
        assert_eq!(id, 44971);
        assert_eq!(stu_id, "202104061314");
    }

    #[test]
    fn test_auth() {
        let id = 44971;
        let stu_id = "202104061314";
        let token = generate_jwt(id, stu_id).unwrap();
        let (res_id, _res_stu_id) = parse(&token).unwrap();
        assert_eq!(id, res_id);
    }
}
