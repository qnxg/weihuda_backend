pub mod cache;
pub mod crypto;
pub mod jwt;
pub mod seg_lock;
pub mod serde;
pub mod time;

pub fn format_stuid(stuid: &str) -> String {
    stuid.trim().to_uppercase()
}
