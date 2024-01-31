use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ZhihuListItem {
    pub id: String,
    pub typetag: String,
    pub title: String,
    pub address: String,
    pub date_begin: String,
    pub is_top: String,
    pub editor: String,
    pub create_time: String,
    pub image: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ZhihuList {
    pub count: u32,
    pub content: Vec<ZhihuListItem>,
}
