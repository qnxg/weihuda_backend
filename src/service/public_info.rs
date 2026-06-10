use crate::{
    result::{AppError, AppResult},
    service::{
        user_info::is_graduate,
        user_state::{HDJW_TOKEN_POOL, Hdjw, with_token},
    },
};
use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EmptyRoom {
    pub name: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub seat: u32,
    pub exam_seat: u32,
}

pub async fn get_empty_room(
    stu_id: &str,
    build_id: &str,
    day: u8,
    jc: Vec<u8>,
    week: u8,
    xn: u16,
    xq: u8,
) -> AppResult<Vec<EmptyRoom>> {
    // 是否是硕士/博士
    let is_graduate = is_graduate(stu_id).await?;

    // 尝试请求
    let try_query = async |stu_id: &str| {
        let build_id = build_id.to_string();
        let jc = jc.clone();
        with_token(Hdjw::new(stu_id), |token| {
            let build_id = &build_id;
            let jc = &jc;
            async move {
                hnu_query::hdjw::get_empty_classroom(
                    &token, build_id, week, day, jc, xn, xq,
                )
                .await
            }
        })
        .await
        .map(|v| {
            v.into_iter()
                .map(|item| EmptyRoom {
                    name: item.room_name,
                    seat: item.seat_count,
                    exam_seat: item.exam_seat_count,
                    _type: item.room_type,
                })
                .collect::<Vec<_>>()
        })
    };

    // 去重 + 尝试把 id 加入池子（超出 5 个就算了）
    let try_add_to_pool = async |id: &str| {
        let id = id.to_string();
        let mut pool = HDJW_TOKEN_POOL.lock().await;
        // 因为号池上限就小，所以直接遍历一遍来判断有没有重复
        if !pool.iter().any(|x| x == &id) && pool.len() < 5 {
            pool.push_back(id);
        }
    };

    for _ in 0..5 {
        // 从号池取最新的号并踢出
        let Some(stu_id_pool) =
            HDJW_TOKEN_POOL.lock().await.pop_back()
        else {
            break;
        };

        match try_query(&stu_id_pool).await {
            Ok(res) => {
                try_add_to_pool(&stu_id_pool).await; // 把借出来的账号放回去
                return Ok(res);
            }
            Err(e) => {
                tracing::error!(err = ?e, tried = ?stu_id_pool, "使用号池查询空教室失败");
                // 本科生：失败后直接用自己账号，如果自己账号也是失败就算了
                // 不一口气把号池消耗完
                if !is_graduate {
                    return match try_query(stu_id).await {
                        Ok(res) => {
                            try_add_to_pool(stu_id).await;
                            Ok(res)
                        }
                        Err(e) => Err(e),
                    };
                }
            }
        }
    }

    if !is_graduate {
        // 号池本来就是空的，则本科生用自己的账号再试一次
        return match try_query(stu_id).await {
            Ok(res) => {
                try_add_to_pool(stu_id).await;
                Ok(res)
            }
            Err(e) => Err(e),
        };
    }
    // 研究生就没辙了
    tracing::error!(tried = ?stu_id, "号池为空，研究生空教室请求失败");
    Err(AppError::Text("请求失败，请稍后重试".to_string()))
}
