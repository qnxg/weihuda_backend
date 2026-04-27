use crate::{
    infra,
    result::{AppResult, ThrowError},
    service::user_state::{Ca, with_token},
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use tokio::sync::{Mutex, OnceCell, Semaphore};

use spider_2024::ca::get_grade_rank as get_rank_from_ca;

const CA_RANK_KEY: &str = "ca_rank";
const CA_TASK_WORKER_COUNT: usize = 4;

#[derive(Serialize, Deserialize, Debug)]
pub struct CaRankDetail {
    pub all_gpa: String,
    pub all_gpa_rank: String,
    pub all_weighted: String,
    pub all_weighted_rank: String,
    pub all_arithmetic: String,
    pub all_arithmetic_rank: String,
    pub must_gpa: String,
    pub must_weighted: String,
    pub must_arithmetic: String,
    pub core_gpa_rank: String,
    pub core_weighted_rank: String,
    pub core_arithmetic_rank: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct CaRank {
    pub detail: CaRankDetail,
    pub update_at: NaiveDateTime,
}
struct CaTaskQueue {
    /// 队列是等待被 worker 处理的任务
    /// HashMap 是当前正在处理的任务
    queue: Mutex<(VecDeque<String>, HashMap<String, ()>)>,
    sem: Semaphore,
}
impl CaTaskQueue {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new((VecDeque::new(), HashMap::new())),
            sem: Semaphore::new(0),
        }
    }
    pub async fn push(&self, stu_id: &str) {
        let mut guard = self.queue.lock().await;
        guard.0.push_back(stu_id.to_string());
        self.sem.add_permits(1);
    }
    pub async fn contains(&self, stu_id: &str) -> bool {
        let guard = self.queue.lock().await;
        guard.0.contains(&stu_id.to_string())
            || guard.1.contains_key(stu_id)
    }
    /// 获取到的元素还是会位于队列中，需要再调用 remove 才能完全从队列中删除
    pub async fn pop(&self) -> String {
        let rem = self.sem.acquire().await.expect("获取信号量失败");
        // tokio 的信号量是 RAII 风格的，如果没有 .forget() 的话，信号量在 drop 后会被归还
        rem.forget();
        let mut guard = self.queue.lock().await;
        let stu_id = guard
            .0
            .pop_front()
            .expect("获取到了信号量，但是队列为空");
        guard.1.insert(stu_id.clone(), ());
        stu_id
    }
    pub async fn remove(&self, stu_id: &str) {
        let mut guard = self.queue.lock().await;
        guard.1.remove(stu_id);
    }
}
static CA_TASK_QUEUE: OnceCell<CaTaskQueue> = OnceCell::const_new();
async fn ca_task_worker() {
    async fn fetch(stu_id: &str) -> AppResult<()> {
        let rank = with_token(Ca::new(stu_id), async |token| {
            get_rank_from_ca(&token).await
        })
        .await?;
        let value = serde_json::to_string(&rank)
            .throw_error("序列化可信电子凭证数据失败")?;
        infra::mysql::kv_cache::insert(
            &format!("{}:{}", CA_RANK_KEY, stu_id),
            value.as_str(),
        )
        .await?;
        Ok(())
    }
    let queue = ca_task_queue().await;
    loop {
        let stu_id = queue.pop().await;
        tracing::info!(stu_id = %stu_id, "获取 CA 排名任务");
        let res = fetch(&stu_id).await;
        if let Err(e) = res {
            tracing::error!(error = ?e, stu_id = %stu_id, "获取 CA 排名失败");
        }
        // 无论成功与否，都要从队列中移除，防止重复处理
        queue.remove(&stu_id).await;
    }
}
async fn ca_task_queue() -> &'static CaTaskQueue {
    CA_TASK_QUEUE
        .get_or_init(|| async { CaTaskQueue::new() })
        .await
}
pub async fn start_ca_task_worker() {
    for _ in 0..CA_TASK_WORKER_COUNT {
        tokio::spawn(ca_task_worker());
    }
}

/// 直接从数据库中获取，如果返回 None，那么就说明正在获取中
/// 如果数据库中也不存在，且当前学号没有位于获取队列中，那么该函数将自动发起获取
pub async fn get_ca_rank(stu_id: &str) -> AppResult<Option<CaRank>> {
    let cache = infra::mysql::kv_cache::get(&format!(
        "{}:{}",
        CA_RANK_KEY, stu_id
    ))
    .await?;
    if let Some((value, update_at)) = cache {
        let detail: CaRankDetail = serde_json::from_str(&value)
            .throw_error("反序列化可信电子凭证数据失败")?;
        Ok(Some(CaRank { detail, update_at }))
    } else {
        refresh_ca_rank(stu_id).await?;
        Ok(None)
    }
}
/// 发起一个获取任务，刷新数据库中保存的信息
/// 如果已经存在于获取队列中，那么就不会重复发起
pub async fn refresh_ca_rank(stu_id: &str) -> AppResult<()> {
    let queue = ca_task_queue().await;
    if queue.contains(stu_id).await {
        return Ok(());
    }
    infra::mysql::kv_cache::delete(&format!(
        "{}:{}",
        CA_RANK_KEY, stu_id
    ))
    .await?;
    queue.push(stu_id).await;
    Ok(())
}
