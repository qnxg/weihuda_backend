use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AuthReq {
    pub code: String,
}

#[derive(Deserialize, Debug)]
pub struct OpenID {
    pub session_key: String,
    pub openid: String,
}

#[derive(Deserialize, Debug)]
pub struct FlutterReq {
    pub stu_id: String,
    pub stu_pwd: String,
}
