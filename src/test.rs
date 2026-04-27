//! 包含测试需要需要用到的常量
//!
//! 本地测试时请注意：确认你使用的是校园网环境，
//!
//! 没有开启代理，且DNS使用了校园网DNS（例如202.197.96.1）

use std::sync::LazyLock;

// TODO 改为从环境变量中读取
pub static TEST_STU_ID: LazyLock<String> =
    LazyLock::new(|| "".to_string());

/// 学年，如 2025
pub static TEST_XN: u16 = 2025;
/// 学期，如 1
pub static TEST_XQ: u8 = 1;
