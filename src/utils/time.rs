use chrono::Datelike;
use once_cell::sync::Lazy;
use regex::Regex;

/// 判断日期格式是否为YYYY-MM-DD
pub fn is_well_formed_date(date: &str) -> bool {
    static PATTERN: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^[0-9]{4}-[0-9]{2}-[0-9]{2}$")
            .expect("构建正则表达式失败")
    });
    PATTERN.is_match(date)
}

/// 判断学年-学期格式是否为YYYY-X
pub fn is_well_formed_xnxq(xnxq: &str) -> bool {
    static PATTERN: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^[0-9]{4}-[0-9]{1}$")
            .expect("构建正则表达式失败")
    });
    PATTERN.is_match(xnxq)
}

/// 获取当前时间信息
/// 返回 (日期字符串，年份，月份，日期)
/// 其中日期字符串格式为YYYY-MM-DD
/// 注意，日期信息为 UTC 时间并非 UTC+8
pub fn date_today() -> (String, i32, u32, u32) {
    let current_date = chrono::Utc::now();
    let year = current_date.year();
    let month = current_date.month();
    let day = current_date.day();
    (format!("{year}-{month:0>2}-{day:0>2}"), year, month, day)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_well_formed_date() {
        assert!(is_well_formed_date("1145-14-19"));
        assert!(is_well_formed_date("1919-81-00"));
        assert!(!is_well_formed_date("1919-1-1"));
        assert!(!is_well_formed_date("1919-1-100"));
        assert!(!is_well_formed_date("919-100-1"));
        assert!(!is_well_formed_date("2O25-O5-O9"));
    }

    #[test]
    fn test_is_well_formed_xnxq() {
        assert!(is_well_formed_xnxq("2077-1"));
        assert!(is_well_formed_xnxq("7707-9"));
        assert!(!is_well_formed_xnxq("7707-90"));
        assert!(!is_well_formed_xnxq("707-90"));
        assert!(!is_well_formed_xnxq("70799-9"));
        assert!(!is_well_formed_xnxq("2O25-1"));
    }

    #[test]
    fn test_date_today() {
        let (s, y, m, d) = date_today();
        assert!(is_well_formed_date(&s));
        assert_eq!(s, "2025-02-20");
        assert_eq!(y, 2025);
        assert_eq!(m, 2);
        assert_eq!(d, 20);
    }
}
