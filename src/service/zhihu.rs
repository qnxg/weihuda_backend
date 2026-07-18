use crate::{
    error::AppResult,
    infra::{self},
    service,
};
use serde::{Deserialize, Serialize};

pub use infra::mysql::zhihu::ZhihuListItem;

/// 返回值的第一个元素表示当前条件下一共有多少个文章
pub async fn get_zhihu_list(
    title: Option<String>,
    typ: Option<String>,
    tags: Option<String>,
    stu_id: &str,
    offset: u32,
    count: u32,
) -> AppResult<(u32, Vec<ZhihuListItem>)> {
    let list = infra::mysql::zhihu::get_zhihu_list(
        title.clone(),
        typ.clone(),
        tags.clone(),
        stu_id,
        offset,
        count,
    )
    .await?;
    let total = infra::mysql::zhihu::get_zhihu_count(
        title, typ, tags, stu_id,
    )
    .await?;
    Ok((total, list))
}

pub use infra::mysql::zhihu::get_zhihu_by_id;

const ZHIHU_TAGS_CONFIG_KEY: &str = "zhihuTags";

#[derive(Serialize, Deserialize, Debug)]
pub struct ZhihuTagItem {
    pub label: String,
    pub value: String,
}
pub async fn get_zhihu_tags() -> AppResult<Vec<ZhihuTagItem>> {
    let tags = service::config::get_config(ZHIHU_TAGS_CONFIG_KEY)
        .await?
        .expect("知湖标签配置不存在")
        .value;
    let tags: Vec<ZhihuTagItem> =
        serde_json::from_str(&tags).expect("知湖标签配置有误");
    Ok(tags)
}
