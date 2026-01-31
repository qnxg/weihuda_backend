use std::collections::{HashMap, VecDeque};

use anyhow::anyhow;
use chrono::NaiveDateTime;
use regex::RegexBuilder;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, OnceCell, Semaphore};

use crate::{infra, result::AppResult};

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
        let rank = get_rank_from_ca(stu_id).await?;
        let value = serde_json::to_string(&rank)?;
        infra::mysql::kv_cache::insert(
            &format!("{}:{}", CA_RANK_KEY, stu_id),
            value.as_str(),
        )
        .await
    }
    let queue = ca_task_queue().await;
    loop {
        let stu_id = queue.pop().await;
        tracing::debug!("获取 CA 排名任务: {}", stu_id);
        let res = fetch(&stu_id).await;
        if let Err(e) = res {
            tracing::error!("获取 CA 排名失败: {:?}", e);
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
/// 从 CA 电子凭证获取排名
/// 直接获取，没有任何缓存
pub async fn get_rank_from_ca(
    stu_id: &str,
) -> AppResult<CaRankDetail> {
    let spider_res =
        infra::spider::ca::get_major_report(stu_id).await?;
    let regex = RegexBuilder::new(r"平均学分绩点排名 ([0-9/]+).*平均学分绩点 ([0-9.]+).*核心课程平均学分绩点排名 ([0-9/]+).*必修课平均学分绩点 ([0-9.]+).*课程算术平均成绩排名 ([0-9/]+).*算术平均分 ([0-9.]+).*核心课程算术平均成绩排名 ([0-9/]+).*必修课算术平均分 ([0-9.]+).*学分加权平均成绩排名 ([0-9/]+).*加权平均分 ([0-9.]+).*核心课程学分加权平均成绩排名 ([0-9/]+).*必修课加权平均分 ([0-9.]+)")
        .dot_matches_new_line(true)
        .build()
        .expect("构建正则表达式失败");
    let caps = regex
        .captures(&spider_res)
        .ok_or(anyhow!("解析可信电子凭证失败"))?
        .iter()
        .map(|c| {
            c.map(|v| v.as_str().to_string())
                .ok_or(anyhow!("解析可信电子凭证失败: 字段为空"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    // 12 个捕获组，caps[0] 是完整匹配，共 13 个
    let [
        _,
        all_gpa_rank,
        all_gpa,
        core_gpa_rank,
        must_gpa,
        all_arithmetic_rank,
        all_arithmetic,
        core_arithmetic_rank,
        must_arithmetic,
        all_weighted_rank,
        all_weighted,
        core_weighted_rank,
        must_weighted,
    ] = caps
        .try_into()
        .map_err(|_| anyhow!("解析可信电子凭证失败: 匹配数量错误"))?;
    let res = CaRankDetail {
        all_gpa,
        all_gpa_rank,
        all_weighted,
        all_weighted_rank,
        all_arithmetic,
        all_arithmetic_rank,
        must_gpa,
        must_weighted,
        must_arithmetic,
        core_gpa_rank,
        core_arithmetic_rank,
        core_weighted_rank,
    };
    Ok(res)
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
        let detail: CaRankDetail = serde_json::from_str(&value)?;
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
