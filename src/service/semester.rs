use crate::result::AppResult;
use crate::service;
use crate::utils;

const SEMESTER_CONFIG_KEY: &str = "classStartDateTable";
const VACATION_DATE_CONFIG_KEY: &str = "nextVacationDate";

/// 获取学期开始日期表
pub async fn get_class_start_date_table()
-> AppResult<Vec<(String, String)>> {
    let config = service::config::get_config(SEMESTER_CONFIG_KEY)
        .await?
        .expect("学期开始日期表配置不存在");
    let mut table: Vec<(String, String)> =
        serde_json::from_str(&config.value)
            .expect("学期开始日期表配置有误");
    for (xnxq, date) in &table {
        assert!(utils::time::is_well_formed_xnxq(xnxq));
        assert!(utils::time::is_well_formed_date(date));
    }
    // 按学年学期排序，便于二分查找
    table.sort();
    // 验证日期递增，这样两个字段都能二分查找
    assert!(table.is_sorted_by_key(|(_xnxq, date)| date));
    Ok(table)
}

/// 获取下一假期时间
pub async fn get_vacation_date() -> AppResult<String> {
    let config =
        service::config::get_config(VACATION_DATE_CONFIG_KEY)
            .await?
            .expect("假期时间配置不存在");
    let res = config.value;
    assert!(utils::time::is_well_formed_date(&res));
    Ok(res)
}

/// 给定某个学年-学期，获取该学期的开始日期
pub async fn get_class_start_date_by_xnxq(
    xn: u32,
    xq: u32,
) -> AppResult<Option<String>> {
    let key = format!("{}-{}", xn, xq);
    let table = get_class_start_date_table().await?;
    let idx =
        table.binary_search_by_key(&&key, |(xn_xq, _)| xn_xq).ok();
    Ok(idx.map(|idx| table[idx].1.clone()))
}

/// 获取当前的学年-学期
pub async fn get_now_xnxq() -> AppResult<(u32, u32)> {
    fn bad_xnxq() -> ! {
        panic!("学期开始日期表的学年-学期格式不正确")
    }
    let table = get_class_start_date_table().await?;
    let [xn, xq]: &[u32] = &(table[get_now_table_index().await?]
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
    Ok((*xn, *xq))
}

/// 获取当前学期在学期开始日期表中的索引
async fn get_now_table_index() -> AppResult<usize> {
    let date = utils::time::date_today().0;
    let res = get_class_start_date_table()
        .await?
        .binary_search_by_key(&&date, |(_, start_date)| start_date)
        .unwrap_or_else(|idx| idx - 1);
    Ok(res)
}

/// 获取本学期开始日期
pub async fn get_this_semester_start_date() -> AppResult<String> {
    let res = get_class_start_date_table().await?
        [get_now_table_index().await?]
        .1
        .clone();
    Ok(res)
}

/// 获取下学期开始日期
pub async fn get_next_semester_start_date() -> AppResult<String> {
    let res = get_class_start_date_table().await?
        [get_now_table_index().await? + 1]
        .1
        .clone();
    Ok(res)
}

/// 更改对应数据后可以测试
#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_get_now_xnxq() {
        assert_eq!(get_now_xnxq().await.unwrap(), (2025, 2));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_get_this_semester_start_date() {
        assert_eq!(
            get_this_semester_start_date().await.unwrap(),
            "2026-03-01".to_string()
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_get_next_semester_start_date() {
        assert_eq!(
            get_next_semester_start_date().await.unwrap(),
            "2026-07-05".to_string()
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_get_vacation() {
        assert_eq!(
            get_vacation_date().await.unwrap(),
            "2026-07-05".to_string()
        );
    }
}
