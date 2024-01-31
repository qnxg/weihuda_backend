use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
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
        &Validation::default(),
    )?;

    Ok(res.claims.id)
}

/// 返回stu_id，用于爬虫请求
pub fn parse_stu_id(token: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let res = decode::<Claims>(
        token,
        &DecodingKey::from_secret(CFG.jwt.secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(res.claims.stu_id)
}

/// 用于即返回mini_bind_id，又返回stu_id的情况
pub fn parse(token: &str) -> Result<(u32, String), jsonwebtoken::errors::Error> {
    let res = decode::<Claims>(
        token,
        &DecodingKey::from_secret(CFG.jwt.secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok((res.claims.id, res.claims.stu_id))
}
