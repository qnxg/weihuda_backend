use crate::{infra, result::AppResult};
use anyhow::anyhow;
use serde::Serialize;

#[derive(Serialize, Debug)]
#[expect(non_snake_case)]
pub struct EmptyRoom {
    pub name: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub seat: u32,
    pub examSeat: u32,
}

pub async fn get_empty_room(
    stu_id: &str,
    build_id: &str,
    day: &str,
    jc: Vec<&str>,
    week: u32,
    xn: u32,
    xq: u32,
) -> AppResult<Vec<EmptyRoom>> {
    let spider_res = infra::spider::hdjw::get_empty_room(
        stu_id, build_id, day, &jc, week, xn, xq,
    )
    .await?;
    let data = spider_res
        .as_array()
        .and_then(|v| v.get(4))
        .and_then(|v| v.as_array())
        .ok_or(anyhow!("解析空教室数据失败"))?;
    let mut res = Vec::new();
    for item in data {
        let item =
            item.as_array().ok_or(anyhow!("解析空教室数据失败"))?;
        let mut is_free = true;
        // 需要每一节课均为空才会被认为是空教室
        for i in 1..1 + jc.len() {
            if !item
                .get(i)
                .ok_or(anyhow!("解析空教室数据失败"))?
                .is_null()
            {
                is_free = false;
                break;
            }
        }
        if !is_free {
            continue;
        }
        let name = item
            .first()
            .and_then(|v| v.as_str())
            .ok_or(anyhow!("解析空教室数据失败"))?;
        let capacity = item
            .get(2 + jc.len())
            .and_then(|v| v.as_str())
            .ok_or(anyhow!("解析空教室数据失败"))?;
        if capacity.len() < 3
            || !capacity.starts_with('(')
            || !capacity.ends_with(')')
        {
            return Err(anyhow!("解析空教室数据失败").into());
        }
        let _type = item
            .get(3 + jc.len())
            .and_then(|v| v.as_str())
            .ok_or(anyhow!("解析空教室数据失败"))?;
        let [seat, exam_seat] = capacity[1..capacity.len() - 1]
            .split('/')
            .collect::<Vec<_>>()[..]
        else {
            return Err(anyhow!("解析空教室数据失败").into());
        };
        let temp = EmptyRoom {
            name: name.to_string(),
            seat: seat
                .parse::<u32>()
                .map_err(|e| anyhow!("解析空教室数据失败 {}", e))?,
            examSeat: exam_seat
                .parse::<u32>()
                .map_err(|e| anyhow!("解析空教室数据失败 {}", e))?,
            _type: _type.to_string(),
        };
        res.push(temp);
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_empty_room() {
        let res = get_empty_room(
            "",
            "106",
            "4",
            vec!["0304", "091011"],
            11,
            2025,
            1,
        )
        .await
        .unwrap();
        println!("{:#?}", res);
    }
}
