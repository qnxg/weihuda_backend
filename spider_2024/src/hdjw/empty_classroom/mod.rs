mod raw;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

/// 空教室信息
#[derive(Serialize, Deserialize, Debug)]
pub struct EmptyClassroom {
    /// 教室名称，如 `综105`
    pub room_name: String,
    /// 教室类型
    // TODO 添加示例
    pub room_type: String,
    /// 座位数
    pub seat_count: u32,
    /// 考试座位数
    pub exam_seat_count: u32,
}

/// 获取空教室信息
///
/// # Arguments
///
/// - `stu_id`: 学号
/// - `building_id`: 楼栋id，参考 `docs/hdjw/building.md` 的 `楼栋 id` 一栏
/// - `week`: 第几周
/// - `day`: 周几，星期一为 `1`，星期日为 `7`
/// - `time`: 节次信息。切片内的元素需要是大节次，参考 `docs/hdjw/time.md` 的 `大节次` 一栏。注意，不支持第 6 大节。
/// - `xn`: 学年
/// - `xq`: 学期
///
/// # Returns
///
/// 空教室列表
///
/// # Panics
///
/// `time` 必须位于区间 [1, 5] 内，否则会 panic
pub async fn get_empty_classroom(
    stu_id: &str,
    building_id: &str,
    week: u8,
    day: u8,
    time: &[u8],
    xn: u16,
    xq: u8,
) -> Result<Vec<EmptyClassroom>, crate::Error> {
    let time_str = time
        .iter()
        .map(|&x| match x {
            1 => "0102",
            2 => "0304",
            3 => "0506",
            4 => "0708",
            5 => "091011",
            _ => panic!("不支持第 {} 大节", x),
        })
        .collect::<Vec<_>>()
        .join(",");
    let raw_data = raw::raw_empty_classroom_data(
        stu_id,
        xn,
        xq,
        week,
        day,
        &time_str,
        building_id,
    )
    .await?;
    let data = raw_data
        .as_array()
        .and_then(|v| v.get(4))
        .and_then(|v| v.as_array())
        .ok_or(anyhow!("解析空教室数据失败 {:?}", raw_data))?;
    let mut res = Vec::new();
    for item in data {
        let item = item
            .as_array()
            .ok_or(anyhow!("解析空教室数据失败 {:?}", item))?;
        let mut is_free = true;
        // 需要每一节课均为空才会被认为是空教室
        for i in 1..=time.len() {
            if !item
                .get(i)
                .ok_or(anyhow!("解析空教室数据失败 {:?}", item))?
                .is_null()
            {
                is_free = false;
                break;
            }
        }
        if !is_free {
            continue;
        }

        let (Some(room_name), Some(seat_count_str), Some(room_type)) = (
            item.first().and_then(|v| v.as_str()),
            item.get(2 + time.len()).and_then(|v| v.as_str()),
            item.get(3 + time.len()).and_then(|v| v.as_str()),
        ) else {
            return Err(
                anyhow!("解析空教室数据失败 {:?}", item).into()
            );
        };

        if seat_count_str.len() < 3
            || !seat_count_str.starts_with('(')
            || !seat_count_str.ends_with(')')
        {
            return Err(anyhow!(
                "解析空教室座位数据失败 {:?}",
                seat_count_str
            )
            .into());
        }
        let [Ok(seat_count), Ok(exam_seat_count)] = seat_count_str
            [1..seat_count_str.len() - 1]
            .split('/')
            .map(|x| x.parse::<u32>())
            .collect::<Vec<_>>()[..]
        else {
            return Err(anyhow!(
                "解析空教室座位数据失败 {:?}",
                seat_count_str
            )
            .into());
        };
        res.push(EmptyClassroom {
            room_name: room_name.to_string(),
            room_type: room_type.to_string(),
            seat_count,
            exam_seat_count,
        });
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::{TEST_STU_ID, TEST_XN, TEST_XQ};

    #[tokio::test]
    async fn test_get_empty_classroom() {
        let building_id = "106"; // 综合楼
        let week = 7;
        let day = 3;
        let time = &[1, 2];
        let res = get_empty_classroom(
            &TEST_STU_ID,
            building_id,
            week,
            day,
            time,
            TEST_XN,
            TEST_XQ,
        )
        .await
        .unwrap();
        println!("{:#?}", res);
    }
}
