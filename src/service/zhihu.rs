use crate::{
    infra::{self},
    result::AppResult,
};

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

pub use infra::mysql::zhihu::add_zhihu;
pub use infra::mysql::zhihu::delete_zhihu;
pub use infra::mysql::zhihu::get_zhihu_by_id;
pub use infra::mysql::zhihu::update_zhihu;
