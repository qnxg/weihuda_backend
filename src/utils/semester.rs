use super::lazy_cache_cell::{lazy_cache_cell, LazyCacheCell};
use crate::utils::redis::get_redis_conn;
use chrono::Datelike;
use redis::AsyncCommands;
use std::time::Duration;

// 每10分钟重新从Redis获得数组
const REFETCH_TIME: Duration = Duration::from_secs(10 * 60);

static CLASS_START_DATE: LazyCacheCell<Vec<(String, String)>> =
    lazy_cache_cell!(REFETCH_TIME, fetch_class_start_date_table);

const REDIS_START_DATE_TABLE_KEY: &str = "config:semester_start_table";

const REDIS_NEXT_VACATION_DATE_KEY: &str = "config:next_vacation_date";

async fn fetch_class_start_date_table() -> Vec<(String, String)> {
    let mut conn = get_redis_conn().await.unwrap();
    let table_json: String = conn.get(REDIS_START_DATE_TABLE_KEY).await.unwrap();
    let mut table: Vec<(String, String)> =
        serde_json::from_str(&table_json).expect("解析学期开始日期表JSON出错");
    // 按学年学期排序，便于二分查找
    table.sort();
    // 验证日期递增，这样两个字段都能二分查找
    assert!(table.is_sorted_by_key(|(ref _xnxq, ref date)| date));
    table
}

pub async fn get_next_vacation() -> String {
    let mut conn = get_redis_conn().await.unwrap();
    conn.get(REDIS_NEXT_VACATION_DATE_KEY).await.unwrap()
}

pub fn date_today() -> (String, i32, u32, u32) {
    let current_date = chrono::Utc::now();
    let year = current_date.year();
    let month = current_date.month();
    let day = current_date.day();
    (format!("{year}-{month:0>2}-{day:0>2}"), year, month, day)
}

pub fn get_class_start_date_by_xnxq(xn: u32, xq: u32) -> Option<String> {
    let key = format!("{}-{}", xn, xq);
    let table = CLASS_START_DATE.read();
    let idx = table.binary_search_by_key(&&key, |(xn_xq, _)| xn_xq).ok()?;
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
    fn test_date_today() {
        let (s, y, m, d) = date_today();
        assert_eq!(s, "2025-01-24");
        assert_eq!(y, 2025);
        assert_eq!(m, 1);
        assert_eq!(d, 24);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_get_now_xnxq() {
        assert_eq!(get_now_xnxq(), (2024, 1));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_get_this_semester_start_date() {
        assert_eq!(get_this_semester_start_date(), "2024-09-08".to_string());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_get_next_semester_start_date() {
        assert_eq!(get_next_semester_start_date(), "2025-02-16".to_string());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_get_next_vacation() {
        assert_eq!(get_next_vacation().await, "2025-01-19".to_string());        
    }
}
