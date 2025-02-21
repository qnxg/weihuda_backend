//! 解决一些请求不带参数，需要为这些请求参数设置默认值的问题

use super::semester::get_now_xnxq;

pub fn default_xn() -> u32 {
    get_now_xnxq().0
}

pub fn default_xq() -> u32 {
    get_now_xnxq().1
}

/// 默认反馈的页码
pub fn default_page() -> u32 {
    1
}
