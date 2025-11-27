use crate::infra;
use crate::result::AppResult;
use crate::utils;

/// 获取学期开始日期表
pub async fn get_class_start_date_table()
-> AppResult<Vec<(String, String)>> {
    let mut table: Vec<(String, String)> =
        infra::mysql::semester::get_class_start_date_table().await?;
    // 按学年学期排序，便于二分查找
    table.sort();
    // 验证日期递增，这样两个字段都能二分查找
    assert!(table.is_sorted_by_key(|(_xnxq, date)| date));
    Ok(table)
}

pub use infra::mysql::semester::get_vacation_date;

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
        .unwrap_or_else(|idx| idx)
        - 1;
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
        assert_eq!(get_now_xnxq().await.unwrap(), (2024, 2));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_get_this_semester_start_date() {
        assert_eq!(
            get_this_semester_start_date().await.unwrap(),
            "2025-02-16".to_string()
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_get_next_semester_start_date() {
        assert_eq!(
            get_next_semester_start_date().await.unwrap(),
            "2025-06-22".to_string()
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_get_vacation() {
        assert_eq!(
            get_vacation_date().await.unwrap(),
            "2025-01-19".to_string()
        );
    }
}
