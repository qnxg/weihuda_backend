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

impl Default for ZhihuList {
    fn default() -> Self {
        Self {
            count: 0,
            content: vec![ZhihuListItem {
                id: String::default(),
                typetag: String::default(),
                title: "知湖暂时下线，待完善后再开放".to_string(),
                address: String::default(),
                date_begin: String::default(),
                is_top: String::default(),
                editor: String::default(),
                create_time: "2024-2-26".to_string(),
                image: String::default(),
            }],
        }
    }
}
