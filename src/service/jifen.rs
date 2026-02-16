use crate::{
    infra::{self},
    result::AppResult,
    service,
};
use anyhow::anyhow;

pub use infra::mysql::jifen::get_exchange_record_list;
pub use infra::mysql::jifen::get_goods;
pub use infra::mysql::jifen::get_goods_list;
pub use infra::mysql::jifen::get_jifen;
pub use infra::mysql::jifen::get_jifen_record;
pub use infra::mysql::jifen::get_jifen_record_count;
pub use infra::mysql::jifen::get_jifen_record_list;
pub use infra::mysql::jifen::get_jifen_rule;
pub use infra::mysql::jifen::get_jifen_rule_list;
pub use infra::mysql::jifen::{JifenGoods, JifenRecord, JifenRule};

const JIFEN_DESC_CONFIG_KEY: &str = "jifenDesc";

/// 增加积分，返回添加后用户的积分
/// 调用前请确保学号是存在的，否则会抛出错误
pub async fn add_jifen(
    stu_id: &str,
    key: &str,
    param: &str,
    desc: &str,
    jifen: i32,
) -> AppResult<i32> {
    // 添加积分记录
    infra::mysql::jifen::add_jifen_record(
        stu_id, key, param, jifen, desc,
    )
    .await?;
    // 更新积分
    infra::mysql::jifen::update_jifen(stu_id, jifen).await?;
    // 获取增加后的积分数值
    let res = infra::mysql::jifen::get_jifen(stu_id)
        .await?
        .ok_or(anyhow!("学号不存在"))?;
    Ok(res)
}

pub async fn exchange_goods(
    stu_id: &str,
    goods: JifenGoods,
) -> AppResult<()> {
    // 减少库存
    infra::mysql::jifen::update_goods_count(goods.id, 1).await?;
    // 减少用户积分
    infra::mysql::jifen::update_jifen(stu_id, -goods.price).await?;
    // 添加兑换记录
    let key = "exchange".to_string();
    let param = goods.id.to_string();
    let desc = format!("兑换商品{}", goods.name);
    infra::mysql::jifen::add_jifen_record(
        stu_id,
        &key,
        &param,
        -goods.price,
        &desc,
    )
    .await?;
    // 添加商品兑换记录
    infra::mysql::jifen::add_exchange_record(stu_id, goods.id)
        .await?;
    Ok(())
}

pub async fn get_jifen_desc() -> AppResult<String> {
    let res = service::config::get_config(JIFEN_DESC_CONFIG_KEY)
        .await?
        .expect("积分额外描述信息配置不存在");
    Ok(res.value)
}

#[cfg(test)]
mod tests {
    use crate::infra;

    #[tokio::test]
    async fn check_jifen() {
        let stu_id = "202326010115";
        let mut records = infra::mysql::jifen::get_jifen_record_list(
            stu_id, 1, 100000, None, None,
        )
        .await
        .unwrap();
        records.sort_by_key(|r| r.id);
        let mut now = 0;
        for record in records.iter() {
            let flag = now >= 0;
            now += record.jifen;
            let flag2 = now <= 0;
            if flag && flag2 {
                println!(
                    "Negative jifen at record id {}: {}\n{:#?}",
                    record.id, now, record
                );
            }
        }
        println!("Final jifen: {}", now);
    }
}
