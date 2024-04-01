//! 解决一些请求不带参数，需要为这些请求参数设置默认值的问题

///TODO 注意及时修改默认学年
pub fn default_xn() -> u32 {
    2023
}

///TODO 注意及时修改默认学期
pub fn default_xq() -> u32 {
    2
}

/// 默认反馈的页码
pub fn default_page() -> u32 {
    1
}
