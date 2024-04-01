//! 对Axum库的Json和Query进行了改写，使错误返回格式与项目的格式保持一致
mod json;
mod query;

pub use json::Json;
pub use query::Query;
