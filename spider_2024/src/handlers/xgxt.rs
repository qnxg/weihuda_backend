use crate::{app_result::HandlerResult, spiders};
use salvo::{Request, handler};

// TODO ：学工系统获取个人信息的传输的数据量比较大，可能会成为性能瓶颈
#[handler]
pub async fn get_person_info_handler(
    req: &mut Request,
) -> HandlerResult {
    let stuid = req
        .query::<String>("stuid")
        .ok_or(anyhow::anyhow!("stuid is required"))?;
    let res = spiders::xgxt::get_person_info(&stuid).await?;
    Ok(res.into())
}
