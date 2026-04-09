use crate::{
    login::get_sticket,
    utils::{
        cache::{CACHE, CacheEnum::*},
        client,
        redis::fetch_password,
        request::cookie_parser,
    },
};
use anyhow::{Result, anyhow};
use reqwest::header::{COOKIE, HeaderMap, SET_COOKIE};
use serde_json::Value;

const GYM_URL_DIRECT_LOGIN: &str = "http://gymos.hnu.edu.cn/bdlp_api_fitness_test_student_h5/public/index.php/index/Login/login";
const GYM_URL_FROM_CAS: &str = "http://cas.hnu.edu.cn/application/sso.zf?login=898A822E9695C137E053026B3E0A65D7";

/// 直接在体测系统的登录页面登录
pub async fn gym_headers(
    stu_id: &str,
) -> Result<HeaderMap, crate::Error> {
    let cookies = if let Some(v) =
        CACHE.get(&(GymCookie, stu_id.into())).await
    {
        v
    } else {
        let password = fetch_password(stu_id).await?;
        let res = client
            .post(GYM_URL_DIRECT_LOGIN)
            .form(&[("student_num", stu_id), ("password", &password)])
            .send()
            .await?
            .error_for_status()?;
        let cookies =
            cookie_parser(res.headers().get_all(SET_COOKIE))
                .join("; ");
        let res: Value = res.json().await?;
        if res["info"] != "登录成功" {
            return Err(anyhow!("登录失败").into());
        }
        CACHE
            .insert((GymCookie, stu_id.into()), cookies.clone())
            .await;
        cookies
    };
    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, cookies.parse()?);
    Ok(headers)
}

/// 从cas登录体测系统
pub async fn gym_headers_from_cas(
    stu_id: &str,
) -> Result<HeaderMap, crate::Error> {
    let cookies = CACHE.try_get_with((GymCookie, stu_id.into()), async {
        let (s_ticket, _) =
                get_sticket(stu_id, GYM_URL_FROM_CAS).await?;
        // 发送请求
        let _res = client
            .get("http://gymos.hnu.edu.cn/bdlp_api_fitness_test_student_h5/view/login/loginPage.html")
            .query(&[("s_ticket", s_ticket.as_str()), ("login_id", stu_id)])
            .send()
            .await?
            .error_for_status()?;
        let res = client
            .post("http://gymos.hnu.edu.cn/bdlp_api_fitness_test_student_h5/public/index.php/index/Login/ticketLogin")
            .form(&[("s_ticket", s_ticket.as_str()), ("login_id", stu_id)])
            .send()
            .await?
            .error_for_status()?;
        // 解除并发锁
        // map_remove(GYM_LOCK, stu_id);
        let cookie = cookie_parser(res.headers().get_all(SET_COOKIE));
        let res: Value = res.json().await?;
        if res["info"] != "登录成功" {
            return Err(anyhow!("登录失败").into());
        }
        let res = cookie.join("; ");
        CACHE.insert((GymCookie, stu_id.into()), res.clone()).await;
        Ok(res)
    }).await?;
    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, cookies.parse()?);
    Ok(headers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TEST_STU_ID;

    #[tokio::test]
    async fn test_gym() {
        let (gym1, gym2) = tokio::join!(
            gym_headers(&TEST_STU_ID),
            gym_headers(&TEST_STU_ID)
        );
        println!("{:#?} {:#?}", gym1.unwrap(), gym2.unwrap());
    }

    #[tokio::test]
    async fn test_gym_from_cas() {
        let gym = gym_headers_from_cas(&TEST_STU_ID).await;
        println!("{:#?}", gym.unwrap());
    }
}
