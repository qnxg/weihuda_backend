use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::CFG;

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

// 把validation设置为const常量
lazy_static! {
    pub static ref VALIDATION: Validation = {
        let mut validation = Validation::default();
        validation.validate_exp = false;
        validation
    };
}

/// 用mini_bind_id和stu_id生成token
pub fn auth(id: u32, stu_id: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;

    let claims = Claims {
        iss: "qnxg".to_string(),
        exp: now + 60 * 60 * 24 * 90,
        sub: "mini-jwt".to_string(),
        iat: now,
        platform: 0,
        id,
        stu_id: stu_id.to_string(),
    };

    let res =
        encode(&Header::default(), &claims, &EncodingKey::from_secret(CFG.jwt.secret.as_bytes()))?;

    Ok(res)
}

/// 返回mini_bind_id，用于数据库操作
pub fn parse_id(token: &str) -> Result<u32, jsonwebtoken::errors::Error> {
    let res = decode::<Claims>(
        token,
        &DecodingKey::from_secret(CFG.jwt.secret.as_bytes()),
        &VALIDATION,
    )?;

    Ok(res.claims.id)
}

/// 返回stu_id，用于爬虫请求
pub fn parse_stu_id(token: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let res = decode::<Claims>(
        token,
        &DecodingKey::from_secret(CFG.jwt.secret.as_bytes()),
        &VALIDATION,
    )?;

    Ok(res.claims.stu_id)
}

/// 用于即返回mini_bind_id，又返回stu_id的情况
pub fn parse(token: &str) -> Result<(u32, String), jsonwebtoken::errors::Error> {
    let res = decode::<Claims>(
        token,
        &DecodingKey::from_secret(CFG.jwt.secret.as_bytes()),
        &VALIDATION,
    )?;

    Ok((res.claims.id, res.claims.stu_id))
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
        let token = auth(id, stu_id).unwrap();
        let (res_id, _res_stu_id) = parse(&token).unwrap();
        assert_eq!(id, res_id);
    }
}
