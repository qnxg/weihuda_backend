use serde::{Deserialize, Serialize};

/// 调休的结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct FlexTime {
    pub from: Option<FlexDay>, // 调休开始时间
    pub to: FlexDay,           // 调休结束时间
    pub desc: String,          // 描述，将会返回给前端用作展示
    pub time: XnXq,            // 学年学期
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
