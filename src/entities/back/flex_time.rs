use serde::{Deserialize, Serialize};

/// 调休的结构体
/// 将会将 from 的课程全部转移到 to 上去，且 to 的课程全部毙掉
#[derive(Debug, Serialize, Deserialize)]
pub struct FlexTime {
    pub from: Option<FlexDay>,
    pub to: FlexDay,
    pub desc: String, // 描述，将会返回给前端用作展示
    pub time: XnXq,   // 学年学期
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FlexDay {
    pub week: u8, // 第几周
    pub day: u8,  // 星期几
}

#[derive(Debug, Serialize, Deserialize)]
pub struct XnXq {
    pub xn: u32, // 学年
    pub xq: u32, // 学期
}
