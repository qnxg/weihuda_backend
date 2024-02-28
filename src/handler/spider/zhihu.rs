use crate::{
    app_result::AppResult, extract::Query, model::spider::zhihu::ZhihuList,
    schema::spider::zhihu::GetZhihuListReq, utility::request::spider_data_url,
};

/// FIXME 待知湖修复后再开放
#[allow(unused_variables, unreachable_code)]
pub async fn get_zhihu_list_handler(Query(req): Query<GetZhihuListReq>) -> AppResult {
    return Ok(ZhihuList::default().into());
    let params = [("kind", req.kind.to_string()), ("page", req.page.to_string())];
    let res: ZhihuList = spider_data_url("http://new.qnxg.cn/zhihulist", &params).await?;

    Ok(res.into())
}
