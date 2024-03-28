pub mod card;
pub mod class_table;
pub mod email;
pub mod empty_room;
pub mod exam;
pub mod fitness;
pub mod global_static;
pub mod grade;
pub mod info;
pub mod lab;
pub mod library;
pub mod netflow;
pub mod raw_grade;

/// 当学分为整数时，序列化为整数，而不是显示类似4.0的浮点数
/// 使用方式：use super::serialize_f64;
pub fn serialize_f64<S>(x: &f64, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if x.fract() == 0.0 {
        s.serialize_u64(*x as u64)
    } else {
        s.serialize_f64(*x)
    }
}
