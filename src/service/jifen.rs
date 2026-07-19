use std::{sync::OnceLock, time::Instant};

use crate::{
    error::{AppError, AppResult, ThrowInternalErrorMsg},
    infra::{self},
    service, utils,
};

use dashmap::DashMap;
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
use tokio::sync::Mutex;

pub struct JifenLockGuard(String);
impl JifenLockGuard {
    pub fn new(key: String) -> Self {
        Self(key)
    }
}
impl Drop for JifenLockGuard {
    fn drop(&mut self) {
        let lock = JIFEN_LOCK.get_or_init(DashMap::new);
        lock.remove(&self.0);
    }
}
static JIFEN_LOCK: OnceLock<DashMap<String, ()>> = OnceLock::new();
/// 尝试获得某 key 对应的锁，如果已经有线程持有锁，则返回 None
fn get_jifen_lock(key: String) -> Option<JifenLockGuard> {
    let lock = JIFEN_LOCK.get_or_init(DashMap::new);
    let mut inserted = false; // 是否已经有线程持有锁
    lock.entry(key.clone()).or_insert_with(|| {
        inserted = true;
    });
    if inserted {
        Some(JifenLockGuard::new(key))
    } else {
        None
    }
}

type GoodsLock = [Mutex<()>; 64];
static GOODS_LOCK: OnceLock<GoodsLock> = OnceLock::new();
fn goods_lock() -> &'static GoodsLock {
    GOODS_LOCK.get_or_init(|| std::array::from_fn(|_| Mutex::new(())))
}

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
        .ok_or_else(|| "积分获取到了空值".internal_err())?;
    Ok(res)
}

/// 兑换商品，返回减少后的积分数值
#[tracing::instrument(
    fields(
        otel.kind = "internal", 
        event_type = "service", 
        // 物品锁等待时间，单位：毫秒
        lock_wait = tracing::field::Empty,
        // 物品锁持有时间，只在函数成功执行后记录，单位：毫秒
        lock_hold = tracing::field::Empty,
    ),
    err
)]
pub async fn exchange_goods(
    stu_id: &str,
    goods_id: u32,
) -> AppResult<i32> {
    const EXCHANGE_GOODS_KEY: &str = "exchange";
    let goods = service::jifen::get_goods(goods_id)
        .await?
        .ok_or_else(|| {
            AppError::customized(format!(
                "没有找到商品：{}",
                goods_id
            ))
        })?;
    // 学号和 goods_id 均加锁
    let Some(_guard1) =
        get_jifen_lock(format!("{}-{}", EXCHANGE_GOODS_KEY, stu_id))
    else {
        return Err(AppError::customized(
            "请求过于频繁，请稍后再试(NO_TOAST)",
        ));
    };
    let timer = Instant::now();
    let _guard2 = goods_lock()[goods_id as usize % 64].lock().await;
    utils::record!(lock_wait = timer.elapsed().as_millis());
    let timer = Instant::now();
    // 检查商品库存
    if !goods.enabled {
        return Err(AppError::customized("商品已下架"));
    }
    if goods.count == 0 {
        return Err(AppError::customized("商品库存不足"));
    }
    let user_jifen = service::jifen::get_jifen(stu_id)
        .await?
        .ok_or_else(|| "积分获取到了空值".internal_err())?;
    if user_jifen < goods.price {
        return Err(AppError::customized("积分不足"));
    }
    // 减少库存
    infra::mysql::jifen::update_goods_count(goods.id, 1).await?;
    // 添加商品兑换记录
    infra::mysql::jifen::add_exchange_record(stu_id, goods.id)
        .await?;
    // 减少用户积分
    let res = add_jifen(
        stu_id,
        EXCHANGE_GOODS_KEY,
        &goods.id.to_string(),
        &format!("兑换商品{}", goods.name),
        -goods.price,
    )
    .await?;
    utils::record!(lock_hold = timer.elapsed().as_millis());
    Ok(res)
}

pub async fn get_jifen_desc() -> AppResult<String> {
    const JIFEN_DESC_CONFIG_KEY: &str = "jifenDesc";
    let res = service::config::get_config(JIFEN_DESC_CONFIG_KEY)
        .await?
        .expect("积分额外描述信息配置不存在");
    Ok(res.value)
}

/// 检查积分记录是否存在，以及是否超出规则限制
/// 返回 true 说明积分记录不存在且没超出限制，可以添加
async fn check_jifen_record(
    stu_id: &str,
    key: &str,
    param: &str,
) -> AppResult<bool> {
    let jifen_rule =
        service::jifen::get_jifen_rule(key).await?.ok_or_else(
            || AppError::customized(format!("没有积分规则：{}", key)),
        )?;
    // 查询是否重复添加
    if service::jifen::get_jifen_record(stu_id, key, param)
        .await?
        .is_some()
    {
        return Ok(false);
    }
    // 查询周期内的积分记录
    let base_time = utils::time::now_time()
        .date()
        .and_hms_opt(0, 0, 0)
        .expect("获取当日零点失败");
    // 取当日零点就相当于已经过了一个周期，所以只需要再去减去 cycle - 1 天
    let create_time_greater_than = base_time
        - chrono::Duration::days(jifen_rule.cycle as i64 - 1);
    let count = service::jifen::get_jifen_record_count(
        stu_id,
        key,
        create_time_greater_than,
    )
    .await?;
    if count >= jifen_rule.max_count {
        return Ok(false);
    }
    Ok(true)
}

/// 签到，返回增加后，当前的积分
pub async fn sign_in(stu_id: &str) -> AppResult<i32> {
    const SIGN_IN_KEY: &str = "qiandao";
    let lock_key = format!("{}-{}", SIGN_IN_KEY, stu_id);
    let param =
        utils::time::now_time().format("%Y-%m-%d").to_string();
    let Some(_guard) = get_jifen_lock(lock_key) else {
        return Err(AppError::customized(
            "请求过于频繁，请稍后再试(NO_TOAST)",
        ));
    };
    if !check_jifen_record(stu_id, SIGN_IN_KEY, &param).await? {
        return Err(AppError::customized("已经签到过了"));
    }
    let rule = service::jifen::get_jifen_rule(SIGN_IN_KEY)
        .await?
        .ok_or_else(|| "没有 qiandao 规则".internal_err())?;
    let res = add_jifen(
        stu_id,
        SIGN_IN_KEY,
        &param,
        &rule.name,
        rule.jifen,
    )
    .await?;
    Ok(res)
}

/// 阅读知湖，返回本次积分增量
pub async fn read_zhihu(stu_id: &str, url: &str) -> AppResult<i32> {
    const READ_ZHIHU_KEY: &str = "yuedu";
    let lock_key = format!("{}-{}", READ_ZHIHU_KEY, stu_id);
    let Some(_guard) = get_jifen_lock(lock_key) else {
        return Err(AppError::customized(
            "请求过于频繁，请稍后再试(NO_TOAST)",
        ));
    };
    if !check_jifen_record(stu_id, READ_ZHIHU_KEY, url).await? {
        // 前端会特判 NO_TOAST，然后就不会弹出错误提示框
        return Err(AppError::customized(
            "已经阅读或超过单日上限(NO_TOAST)",
        ));
    }
    let rule = service::jifen::get_jifen_rule(READ_ZHIHU_KEY)
        .await?
        .ok_or_else(|| "没有 yuedu 规则".internal_err())?;
    add_jifen(stu_id, READ_ZHIHU_KEY, url, &rule.name, rule.jifen)
        .await?;
    Ok(rule.jifen)
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
