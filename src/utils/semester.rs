use crate::entities::spider::global_static::ClassStartDateTable;
use chrono::Datelike;

pub fn date_today() -> (String, i32, u32, u32) {
    let current_date = chrono::Utc::now();
    let year = current_date.year();
    let month = current_date.month();
    let day = current_date.day();
    (format!("{year}-{month:0>2}-{day:0>2}"), year, month, day)
}

pub fn get_class_start_date_by_xnxq(xn: u32, xq: u32) -> Option<String> {
    let key = format!("{}-{}", xn, xq);
    let idx = ClassStartDateTable.binary_search_by_key(&&key, |(xn_xq, _)| xn_xq).ok()?;
    Some(ClassStartDateTable[idx].1.clone())
}

pub fn get_now_xnxq() -> (u32, u32) {
    fn bad_xnxq() -> ! {
        panic!("学期开始日期表的学年-学期格式不正确")
    }
    let [xn, xq]: &[u32] = &(ClassStartDateTable[get_now_table_index()]
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

/// TODO: 处理硬编码的假期
pub fn get_next_vacation() -> String {
    "2025-01-19".into()
}

fn get_now_table_index() -> usize {
    let date = date_today().0;
    ClassStartDateTable
        .binary_search_by_key(&&date, |(_, start_date)| start_date)
        .unwrap_or_else(|idx| idx)
        - 1
}

pub fn get_this_semester_start_date() -> String {
    ClassStartDateTable[get_now_table_index()].1.clone()
}

pub fn get_next_semester_start_date() -> String {
    ClassStartDateTable[get_now_table_index() + 1].1.clone()
}

/// 更改对应数据后可以测试
// #[cfg(test)]
// mod test {
//     use super::*;
//     #[test]
//     fn test_date_today() {
//         let (s, y, m, d) = date_today();
//         assert_eq!(s, "2025-01-20");
//         assert_eq!(y, 2025);
//         assert_eq!(m, 1);
//         assert_eq!(d, 20);
//     }

//     #[test]
//     fn test_get_now_xnxq() {
//         assert_eq!(get_now_xnxq(), (2024, 1));
//     }

//     #[test]
//     fn test_get_this_semester_start_date() {
//         assert_eq!(get_this_semester_start_date(), "2024-09-08".to_string());
//     }

//     #[test]
//     fn test_get_next_semester_start_date() {
//         assert_eq!(get_next_semester_start_date(), "2025-02-16".to_string());
//     }
// }
