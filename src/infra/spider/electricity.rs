use crate::{infra::spider::spider_data, result::AppResult};

pub async fn get_electricity(
    park: &str,
    build: &str,
    room: &str,
    refresh: bool,
) -> AppResult<String> {
    let params = [
        ("park", park),
        ("build", build),
        ("room", room),
        ("refresh", if refresh { "1" } else { "0" }),
    ];
    let res: String =
        spider_data("/electricity/query", &params).await?;
    Ok(res)
}
