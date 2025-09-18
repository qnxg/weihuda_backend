use crate::app_result::{AppResult, AppState};
use crate::dtos::spider::xgxt::{Dormitory, PersonInfo};
use crate::extractors::Query;
use crate::utils::jwt::parse_stu_id;
use crate::utils::redis::get_redis_conn;
use crate::utils::request::spider_data;
use anyhow::{anyhow, Result};
use axum::extract::State;
use axum::Extension;
use redis::AsyncCommands;
use regex::Regex;
use serde::Deserialize;
use sqlx::MySqlPool;

/// 解析宿舍信息，返回 (park, build, room)
/// 这个函数目前来说还容易漏情况，所以单独拿出来便于测试
fn parse_dormitory(dormitory: Dormitory) -> Result<(String, String, String)> {
    let park = match dormitory.park.as_str() {
        "南校区" => 1,
        "财院校区" => 2,
        "天马园区" => 3,
        "德智园区" => 4,
        "德智留学生公寓" => 5,
        "望麓桥学生公寓" => 6,
        "牛头山学生公寓" => 7,
        _ => return Err(anyhow!("解析宿舍信息失败，未知园区")),
    };
    let build = match (park, dormitory.build.as_str()) {
        // 南校区
        (1, "7舍") => "19-1",
        (1, "8舍") => "20",
        (1, "10舍") => "21",
        (1, "11舍") => "21-0",
        (1, "12舍") => "21-1",
        (1, "13舍") => "21-2",
        (1, "14舍") => "22",
        (1, "15舍") => "23",
        (1, "17舍") => "24",
        (1, "18舍") => "25",
        // 19舍比较特殊，常见情况：南校区/19舍/1-附204，南校区/19舍/2-320
        (1, "19舍") => {
            let no = dormitory
                .room
                .chars()
                .next()
                .ok_or(anyhow!("解析宿舍信息失败：{}", dormitory.room))?;
            match no {
                '1' => "25-1",
                '2' => "25-2",
                '3' => "25-3",
                '4' => "25-4",
                _ => return Err(anyhow!("解析宿舍信息失败：{}", dormitory.room)),
            }
        }
        (1, "南楼") => "26",
        (1, "培训小楼") => "27",
        // 财院校区
        (2, "1栋") => "01",
        (2, "2栋") => "02",
        (2, "3栋") => "02-01",
        (2, "5栋") => "03",
        (2, "6栋") => "04",
        (2, "12栋") => "05",
        (2, "A栋") => "06",
        (2, "B栋") => "07",
        (2, "研楼7栋") => "08",
        // 天马园区
        (3, "一区1栋") => "28",
        (3, "一区2栋") => "29",
        (3, "一区3栋") => "30",
        (3, "一区4栋") => "30-1",
        (3, "二区1栋") => "31",
        (3, "二区2栋") => "32",
        (3, "二区3栋") => "33",
        (3, "二区4栋") => "34",
        (3, "二区5栋") => "35",
        (3, "二区6栋") => "36",
        (3, "二区7栋") => "37",
        (3, "三区9栋") => "38",
        (3, "三区10栋") => "39",
        (3, "三区11栋") => "40",
        (3, "三区12栋") => "41",
        (3, "三区13栋") => "42",
        (3, "三区16栋") => "43",
        (3, "三区17栋") => "44",
        (3, "三区18栋") => "45",
        (3, "三区19栋") => "46",
        (3, "三区20栋") => "46-1",
        (3, "四区1栋") => "47",
        (3, "四区2栋") => "48",
        (3, "四区3栋") => "49",
        (3, "四区4栋") => "50",
        // (3, "事务大楼") => "56", // 目前代码这里应该是解析不到的，等待数据修复
        // 德智园区
        (4, "2栋") => "09",
        (4, "5栋") => "10",
        (4, "6栋") => "11",
        (4, "7栋") => "12",
        (4, "8栋") => "13",
        (4, "9栋") => "14",
        (4, "10栋") => "15",
        (4, "11栋") => "16",
        (4, "13栋") => "17",
        (4, "15栋") => "17-01",
        (4, "16栋") => "17-02",
        // 德智留学生公寓，似乎只有一个
        (5, _) => "18",
        // 望麓桥学生公寓
        (6, "1栋") => "51",
        (6, "2栋") => "#2栋",
        (6, "3栋") => "#3栋",
        (6, "4栋") => "57",
        // 牛头山学生公寓
        (7, "2栋") => "60",
        (7, "3栋") => "61",
        (7, "4栋") => "62",
        (7, "5栋") => "63",
        (7, "6栋") => "64",
        (7, "7栋") => "65",
        _ => return Err(anyhow!("解析宿舍信息失败，未知园区")),
    };
    let room = match (park, dormitory.build.as_str(), dormitory.room.as_str()) {
        // 财院校区A栋，请在房间号首位加上A、B、C，暂时不知道怎么处理
        // 德智园区，在房间号前加上楼栋号：
        (4, "2栋", r) => format!("2{}", r),
        (4, "5栋", r) => format!("5{}", r),
        (4, "6栋", r) => format!("6{}", r),
        (4, "7栋", r) => format!("7{}", r),
        (4, "8栋", r) => format!("8{}", r),
        (4, "9栋", r) => format!("9{}", r),
        (4, "10栋", r) => format!("10{}", r),
        (4, "11栋", r) => format!("11{}", r),
        (4, "13栋", r) => format!("13{}", r),
        (4, "15栋", r) => format!("15{}", r),
        (4, "16栋", r) => format!("16{}", r),
        // 南校区19舍附楼，请在房间号前加上F：
        // 常见情况：南校区/19舍/1-附204，南校区/19舍/2-320
        (1, "19舍", r) => {
            let parts = r.split('-').collect::<Vec<&str>>();
            if parts.len() != 2 {
                return Err(anyhow!("解析宿舍信息失败：{}", dormitory.room));
            }
            if parts[1].starts_with('附') {
                format!("F{}", parts[1].replace('附', ""))
            } else {
                parts[1].to_string()
            }
        }
        (_, _, r) => r.to_string(),
    };
    Ok((park.to_string(), build.to_string(), room))
}

#[allow(clippy::get_first)]
async fn get_dormitory(stu_id: &str, db: &MySqlPool) -> Result<Option<Dormitory>> {
    let text = sqlx::query_scalar!("select room from mini_bind where stuID = ?", stu_id)
        .fetch_one(db)
        .await?;
    if text == "0" || text.is_empty() {
        return Ok(None);
    }
    let arr = text.split("/").collect::<Vec<&str>>();
    if arr.len() != 3 {
        return Ok(None);
    }
    for i in &arr {
        if i.is_empty() {
            return Ok(None);
        }
    }
    Ok(Some(Dormitory {
        park: arr.get(0).unwrap().to_string(),
        build: arr.get(1).unwrap().to_string(),
        room: arr.get(2).unwrap().to_string(),
    }))
}

#[derive(Deserialize, Debug)]
pub struct Req {
    pub refresh: u8,
}

// 获取电量信息
pub async fn get_electricity_handler(
    State(data): AppState,
    Query(req): Query<Req>,
    Extension(token): Extension<String>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    // 拉取
    let mut dormitory = get_dormitory(stu_id.as_str(), &data.db).await?;
    if dormitory.is_none() {
        update_dormitory_handler(State(data.clone()), Extension(token)).await?;
        dormitory = get_dormitory(stu_id.as_str(), &data.db).await?;
    }
    // 还为空就摆烂
    let dormitory = dormitory.ok_or(anyhow!("获取宿舍信息失败"))?;
    let (park, build, room) = parse_dormitory(dormitory)?;
    match build.as_str() {
        // 望麓桥学生公寓的2栋和3栋无法区分南边还是北面
        // 考虑到同一个宿舍号不可能既是南又是北，所以我们两个都试试，取成功的
        "#2栋" | "#3栋" => {
            let params_north = [
                ("park", park.clone()),
                (
                    "build",
                    match build.as_str() {
                        "#2栋" => "52",
                        "#3栋" => "54",
                        _ => unreachable!(),
                    }
                    .to_string(),
                ),
                ("room", room.clone()),
                ("refresh", req.refresh.to_string()),
            ];
            let params_south = [
                ("park", park),
                (
                    "build",
                    match build.as_str() {
                        "#2栋" => "53",
                        "#3栋" => "55",
                        _ => unreachable!(),
                    }
                    .to_string(),
                ),
                ("room", room),
                ("refresh", req.refresh.to_string()),
            ];
            let res_north = spider_data("/electricity/query", &params_north).await;
            let res_south = spider_data("/electricity/query", &params_south).await;
            match (res_north, res_south) {
                (Ok(n), Err(_)) => {
                    // 这样做是为了给编译器类型推断提示
                    let n: String = n;
                    Ok(n.into())
                }
                (Err(_), Ok(s)) => {
                    let s: String = s;
                    Ok(s.into())
                }
                _ => Err(anyhow!("获取电量信息失败，无法区分宿舍南北").into()),
            }
        }
        _ => {
            let params = [
                ("park", park),
                ("build", build),
                ("room", room),
                ("refresh", req.refresh.to_string()),
            ];
            let res: String = spider_data("/electricity/query", &params).await?;
            Ok(res.into())
        }
    }
}

// 更新个人宿舍信息到数据库
pub async fn update_dormitory_handler(
    State(data): AppState,
    Extension(token): Extension<String>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    // 先删掉 redis 中的缓存
    let _: () = get_redis_conn()
        .await?
        .del(format!("person_info-{}", stu_id))
        .await
        .map_err(|e| anyhow!("删除 Redis 缓存失败：{}", e))?;
    let person_info: PersonInfo =
        spider_data("/xgxt/person_info", &[("stuid", stu_id.clone())]).await?;
    // 将学工系统里的住宿信息解析
    // dbg!(&person_info);
    let mut park = "";
    let mut build = "";
    let room = person_info.room;
    if person_info.dormitory.contains("德智") {
        park = "德智园区";
        let re = Regex::new(r"\d+栋").unwrap();
        build = re
            .find_iter(person_info.dormitory.as_str())
            .map(|mat| mat.as_str())
            .next()
            .unwrap_or("");
    }
    if person_info.dormitory.contains("天马") {
        park = "天马园区";
        let re = Regex::new(r"[一二三四]区\d+栋").unwrap();
        build = re
            .find_iter(person_info.dormitory.as_str())
            .map(|mat| mat.as_str())
            .next()
            .unwrap_or("");
    }
    if person_info.dormitory.contains("望麓桥") {
        park = "望麓桥学生公寓";
        let re = Regex::new(r"\d+栋").unwrap();
        build = re
            .find_iter(person_info.dormitory.as_str())
            .map(|mat| mat.as_str())
            .next()
            .unwrap_or("");
    }
    if person_info.dormitory.contains("牛头山") {
        park = "牛头山学生公寓";
        let re = Regex::new(r"\d+栋").unwrap();
        build = re
            .find_iter(person_info.dormitory.as_str())
            .map(|mat| mat.as_str())
            .next()
            .unwrap_or("");
    }
    if person_info.dormitory.contains("财院校区") {
        park = "财院校区";
        let re = Regex::new(r"[1-9AB]+栋").unwrap();
        build = re
            .find_iter(person_info.dormitory.as_str())
            .map(|mat| mat.as_str())
            .next()
            .unwrap_or("");
        // TODO 研楼目前还没有样本，不知道怎么搞
    }
    if person_info.dormitory.contains("南校区") {
        park = "南校区";
        let re = Regex::new(r"[1-9]+舍").unwrap();
        build = re
            .find_iter(person_info.dormitory.as_str())
            .map(|mat| mat.as_str())
            .next()
            .unwrap_or("");
    }
    // 其他的后续再添加
    sqlx::query!(
        r#"
        update mini_bind set room = ? where stuID = ?
        "#,
        format!("{}/{}/{}", park, build, room),
        stu_id
    )
    .execute(&data.db)
    .await?;
    Ok(().into())
}

pub async fn get_dormitory_handler(
    State(data): AppState,
    Extension(token): Extension<String>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let res = get_dormitory(stu_id.as_str(), &data.db).await?;
    Ok(res.into())
}

#[cfg(test)]
mod tests {
    use crate::run;
    use crate::utils::jwt::auth;
    use reqwest::Client;

    #[tokio::test]
    async fn test_get_elecricity() {
        tokio::spawn(async {
            run().await;
        });
        // 生成jwt
        let jwt = auth(0, "202504100328").unwrap();
        // 发送网络请求
        let url = "http://127.0.0.1:8000";
        let res = Client::new()
            .get(format!("{}/electricity", url))
            .header("Authorization", jwt)
            .query(&[("refresh", 1)])
            .send()
            .await
            .unwrap();
        println!("{:?}", res.text().await.unwrap());
    }

    #[tokio::test]
    async fn test_all_dormitory() {
        // 这个测试将根据数据集测试对所有宿舍信息的解析，耗时会比较长
        // 数据集在 testdata/electricity/weihuda_mini_bind_[1-9].json
        // 数据集中第一个字段为 id，第二个字段为 room。id 便于出错后继续排查
        for i in 1..=9 {
            let text = std::fs::read_to_string(format!(
                "testdata/electricity/weihuda_mini_bind_{}.json",
                i
            ))
            .unwrap();
            let arr: Vec<serde_json::Value> = serde_json::from_str(&text).unwrap();
            for item in arr {
                let id = item.get("id").unwrap().as_i64().unwrap();
                let room = item.get("room").unwrap().as_str().unwrap();
                let arr = room.split("/").collect::<Vec<&str>>();
                if arr.len() != 3 {
                    continue;
                }
                // 如果有字段为空的话说明是从学工系统解析出的锅，跳过
                let mut flag = false;
                for i in &arr {
                    if i.is_empty() {
                        flag = true;
                        break;
                    }
                }
                if flag {
                    continue;
                }
                let dormitory = crate::dtos::spider::xgxt::Dormitory {
                    park: arr.get(0).unwrap().to_string(),
                    build: arr.get(1).unwrap().to_string(),
                    room: arr.get(2).unwrap().to_string(),
                };
                let res = super::parse_dormitory(dormitory);
                if res.is_err() {
                    eprintln!("id={} 解析失败，room={}", id, room);
                }
            }
        }
    }
}
