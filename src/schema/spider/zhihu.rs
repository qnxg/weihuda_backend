use serde::Deserialize;

/// 获取月流量明细
#[derive(Deserialize, Debug)]
pub struct GetZhihuListReq {
    pub kind: u8, // 0-3之间的整数
    pub page: u32,
}
