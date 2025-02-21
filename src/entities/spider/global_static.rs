#![allow(non_upper_case_globals)]
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    /// 课程开始时间
    pub static ref StartMap: HashMap<u32, &'static str> = [
        (1, "8:00"),
        (2, "8:55"),
        (3, "10:00"),
        (4, "10:55"),
        (5, "14:30"),
        (6, "15:15"),
        (7, "16:10"),
        (8, "16:55"),
        (9, "19:00"),
        (10, "19:55"),
        (11, "20:50"),
        (12, "21:35"),
    ]
    .into_iter()
    .collect();
    /// 课程结束时间
    pub static ref EndMap: HashMap<u32, &'static str> = [
        (1, "8:45"),
        (2, "9:40"),
        (3, "10:45"),
        (4, "11:40"),
        (5, "15:15"),
        (6, "16:00"),
        (7, "16:55"),
        (8, "17:40"),
        (9, "19:45"),
        (10, "20:40"),
        (11, "21:35"),
        (12, "22:20"),
    ]
    .into_iter()
    .collect();
}
