use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{
    infra::{self},
    result::AppResult,
};

const REDIS_PERSON_INFO_KEY_PREFIX: &str = "person_info-";

pub use infra::spider::xgxt::get_person_info;

#[derive(Serialize, Deserialize, Debug)]
pub struct Dormitory {
    pub park: String,
    pub build: String,
    pub room: String,
}
/// 会从数据库中获得信息，如果没有或者数据库中的信息解析失败则返回 None
pub async fn get_dormitory(
    stu_id: &str,
) -> AppResult<Option<Dormitory>> {
    if let Some(dormitory) =
        infra::mysql::user::get_room(stu_id).await?
    {
        if dormitory == "0" || dormitory.is_empty() {
            return Ok(None);
        }
        let arr: Vec<&str> = dormitory.split("/").collect();
        let [park, build, room] = arr[..] else {
            return Ok(None);
        };
        Ok(Some(Dormitory {
            park: park.to_string(),
            build: build.to_string(),
            room: room.to_string(),
        }))
    } else {
        Ok(None)
    }
}

/// 重新调用爬虫更新寝室信息
pub async fn update_dormitory(stu_id: &str) -> AppResult<()> {
    // 先删掉 redis 中之前缓存的个人信息数据，防止宿舍信息没有更新
    infra::redis::del(&format!(
        "{}{}",
        REDIS_PERSON_INFO_KEY_PREFIX, stu_id
    ))
    .await?;
    let person_info = get_person_info(stu_id).await?;
    // 解析宿舍信息为我们需要的格式
    let dormitory = parse_dormitory_info(
        &person_info.dormitory,
        &person_info.room,
    );
    let dormitory_str = format!(
        "{}/{}/{}",
        dormitory.park, dormitory.build, dormitory.room
    );
    infra::mysql::user::update_room(stu_id, &dormitory_str).await?;
    Ok(())
}
/// 将 PersonInfo 的 dormitory 和 room 字段解析为 Dormitory 结构体
/// 主要是把 dormitory 字段中的园区和楼栋信息提取出来
fn parse_dormitory_info(dormitory: &str, room: &str) -> Dormitory {
    let mut park = "";
    let mut build = "";
    if dormitory.contains("德智") {
        park = "德智园区";
        let re = Regex::new(r"\d+栋").expect("构建正则表达式失败");
        build = re
            .find_iter(dormitory)
            .map(|mat| mat.as_str())
            .next()
            .unwrap_or("");
    }
    if dormitory.contains("天马") {
        park = "天马园区";
        let re = Regex::new(r"[一二三四]区\d+栋")
            .expect("构建正则表达式失败");
        build = re
            .find_iter(dormitory)
            .map(|mat| mat.as_str())
            .next()
            .unwrap_or("");
    }
    if dormitory.contains("望麓桥") {
        park = "望麓桥学生公寓";
        let re = Regex::new(r"\d+栋").expect("构建正则表达式失败");
        build = re
            .find_iter(dormitory)
            .map(|mat| mat.as_str())
            .next()
            .unwrap_or("");
    }
    if dormitory.contains("牛头山") {
        park = "牛头山学生公寓";
        let re = Regex::new(r"\d+栋").expect("构建正则表达式失败");
        build = re
            .find_iter(dormitory)
            .map(|mat| mat.as_str())
            .next()
            .unwrap_or("");
    }
    if dormitory.contains("财院校区") {
        park = "财院校区";
        let re =
            Regex::new(r"[1-9AB]+栋").expect("构建正则表达式失败");
        build = re
            .find_iter(dormitory)
            .map(|mat| mat.as_str())
            .next()
            .unwrap_or("");
        // TODO 研楼目前还没有样本，不知道怎么搞
    }
    if dormitory.contains("南校区") {
        park = "南校区";
        let re = Regex::new(r"[1-9]+舍").expect("构建正则表达式失败");
        build = re
            .find_iter(dormitory)
            .map(|mat| mat.as_str())
            .next()
            .unwrap_or("");
    }
    Dormitory {
        park: park.to_string(),
        build: build.to_string(),
        room: room.to_string(),
    }
}
