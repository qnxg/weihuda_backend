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
    jc: &str,
    week: u32,
    xn: u32,
    xq: u32,
) -> AppResult<Vec<EmptyRoom>> {
    let spider_res = infra::spider::hdjw::get_empty_room(
        stu_id, build_id, day, jc, week, xn, xq,
    )
    .await?;
    let data = spider_res
        .as_array()
        .ok_or(anyhow!("解析空教室数据失败"))?
        .get(4)
        .ok_or(anyhow!("解析空教室数据失败"))?
        .as_array()
        .ok_or(anyhow!("解析空教室数据失败"))?;
    let mut res = Vec::new();
    for item in data {
        let item =
            item.as_array().ok_or(anyhow!("解析空教室数据失败"))?;
        let is_free = item
            .get(1)
            .ok_or(anyhow!("解析空教室数据失败"))?
            .is_null();
        if !is_free {
            continue;
        }
        let name = item
            .first()
            .ok_or(anyhow!("解析空教室数据失败"))?
            .as_str()
            .ok_or(anyhow!("解析空教室数据失败"))?;
        let capacity = item
            .get(3)
            .ok_or(anyhow!("解析空教室数据失败"))?
            .as_str()
            .ok_or(anyhow!("解析空教室数据失败"))?;
        if capacity.len() < 3
            || !capacity.starts_with('(')
            || !capacity.ends_with(')')
        {
            return Err(anyhow!("解析空教室数据失败").into());
        }
        let _type = item
            .get(4)
            .ok_or(anyhow!("解析空教室数据失败"))?
            .as_str()
            .ok_or(anyhow!("解析空教室数据失败"))?;
        let mut capacity = capacity[1..capacity.len() - 1].split('/');
        let seat = capacity
            .next()
            .ok_or(anyhow!("解析空教室数据失败"))?
            .parse::<u32>()
            .map_err(|e| anyhow!("解析空教室数据失败 {}", e))?;
        let exam_seat = capacity
            .next()
            .ok_or(anyhow!("解析空教室数据失败"))?
            .parse::<u32>()
            .map_err(|e| anyhow!("解析空教室数据失败 {}", e))?;
        let temp = EmptyRoom {
            name: name.to_string(),
            seat,
            examSeat: exam_seat,
            _type: _type.to_string(),
        };
        res.push(temp);
    }
    Ok(res)
}
