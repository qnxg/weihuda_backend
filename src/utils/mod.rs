pub mod crypto;
pub mod jwt;
pub mod serde;
pub mod time;

pub fn format_stuid(stuid: &str) -> String {
    stuid.trim().to_uppercase()
}
