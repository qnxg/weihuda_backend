use crate::database::get_db_pool;

use super::lazy_cache_cell::{lazy_cache_cell, LazyCacheCell};
use chrono::Datelike;
use once_cell::sync::Lazy;
use regex::Regex;
use std::time::Duration;

// 每10分钟重新从Redis获得数组
const REFETCH_TIME: Duration = Duration::from_secs(10 * 60);

static CLASS_START_DATE: LazyCacheCell<Vec<(String, String)>> =
    lazy_cache_cell!(REFETCH_TIME, fetch_class_start_date_table);

const SQL_START_DATE_TABLE_KEY: &str = "classStartDateTable";

const SQL_VACATION_DATE_KEY: &str = "nextVacationDate";

/// 获取学期开始日期表
///
/// Redis中JSON格式为：
/// `[["xxxx-n", "yyyy-mm-dd"], ...]`
/// 每一项的前一项为`学年-学期`，后一项为学期开始日期。
/// 数字位数要确认相同。
async fn fetch_class_start_date_table() -> Vec<(String, String)> {
    let table_json: String = sqlx::query!(
        r#"
            SELECT
                value
            FROM
                mini_configs
            WHERE
                `key` = ? AND enabled = 1
            "#,
        SQL_START_DATE_TABLE_KEY
    )
    .fetch_one(&get_db_pool().await)
    .await
    .expect("学期日期表不见了")
    .value;
    let mut table: Vec<(String, String)> =
        serde_json::from_str(&table_json)
            .expect("解析学期开始日期表JSON出错");
    for (xnxq, date) in &table {
        assert!(is_well_formed_xnxq(xnxq));
        assert!(is_well_formed_date(date));
    }
    // 按学年学期排序，便于二分查找
    table.sort();
    // 验证日期递增，这样两个字段都能二分查找
    assert!(table.is_sorted_by_key(|(ref _xnxq, ref date)| date));
    table
}

pub fn is_well_formed_date(date: &str) -> bool {
    static PATTERN: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^[0-9]{4}-[0-9]{2}-[0-9]{2}$").unwrap()
    });
    PATTERN.is_match(date)
}

pub fn is_well_formed_xnxq(xnxq: &str) -> bool {
    static PATTERN: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^[0-9]{4}-[0-9]{1}$").unwrap());
    PATTERN.is_match(xnxq)
}

pub async fn get_vacation_date() -> String {
    let res = sqlx::query!(
        r#"
            SELECT
                value
            FROM
                mini_configs
            WHERE
                `key` = ? AND enabled = 1
            "#,
        SQL_VACATION_DATE_KEY
    )
    .fetch_one(&get_db_pool().await)
    .await
    .expect("假期时间数据不见了")
    .value;
    assert!(is_well_formed_date(&res));
    res
}

pub fn date_today() -> (String, i32, u32, u32) {
    let current_date = chrono::Utc::now();
    let year = current_date.year();
    let month = current_date.month();
    let day = current_date.day();
    (format!("{year}-{month:0>2}-{day:0>2}"), year, month, day)
}

pub fn get_class_start_date_by_xnxq(
    xn: u32,
    xq: u32,
) -> Option<String> {
    let key = format!("{}-{}", xn, xq);
    let table = CLASS_START_DATE.read();
    let idx =
        table.binary_search_by_key(&&key, |(xn_xq, _)| xn_xq).ok()?;
    Some(table[idx].1.clone())
}

pub fn get_now_xnxq() -> (u32, u32) {
    fn bad_xnxq() -> ! {
        panic!("学期开始日期表的学年-学期格式不正确")
    }
    let table = CLASS_START_DATE.read();
    let [xn, xq]: &[u32] = &(table[get_now_table_index()]
        .0
        .split('-')
        .map(|data| match data.parse() {
            Ok(u) => u,
            _ => bad_xnxq(),
        })
        .collect::<Vec<_>>())[..]
    else {
        bad_xnxq();
    };
    (*xn, *xq)
}

fn get_now_table_index() -> usize {
    let date = date_today().0;
    CLASS_START_DATE
        .read()
        .binary_search_by_key(&&date, |(_, start_date)| start_date)
        .unwrap_or_else(|idx| idx)
        - 1
}

pub fn get_this_semester_start_date() -> String {
    CLASS_START_DATE.read()[get_now_table_index()].1.clone()
}

pub fn get_next_semester_start_date() -> String {
    CLASS_START_DATE.read()[get_now_table_index() + 1].1.clone()
}

/// 更改对应数据后可以测试
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

    #[tokio::test(flavor = "multi_thread")]
    async fn test_get_now_xnxq() {
        assert_eq!(get_now_xnxq(), (2024, 2));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_get_this_semester_start_date() {
        assert_eq!(
            get_this_semester_start_date(),
            "2025-02-16".to_string()
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_get_next_semester_start_date() {
        assert_eq!(
            get_next_semester_start_date(),
            "2025-06-22".to_string()
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_get_vacation() {
        assert_eq!(
            get_vacation_date().await,
            "2025-01-19".to_string()
        );
    }
}
