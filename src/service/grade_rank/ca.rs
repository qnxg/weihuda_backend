use crate::{
    error::{AppResult, ThrowInternalErrorResult},
    infra,
    service::user_state::{Ca, with_token},
    utils,
};
use chrono::NaiveDateTime;
use hnu_query::ca::get_grade_rank as get_rank_from_ca;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use tokio::sync::{Mutex, OnceCell, Semaphore};
use tracing::Instrument;

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

struct CaTask {
    stu_id: String,
    /// 触发该任务的请求的 OTLP trace 上下文
    ///
    /// 后台 ca_task span 用此关联回触发的请求
    context: Option<(String, String)>,
}

struct CaTaskQueue {
    /// 队列是等待被 worker 处理的任务
    /// HashMap 是当前正在处理的学号（去重用）
    queue: Mutex<(VecDeque<CaTask>, HashMap<String, ()>)>,
    sem: Semaphore,
}
impl CaTaskQueue {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new((VecDeque::new(), HashMap::new())),
            sem: Semaphore::new(0),
        }
    }

    pub async fn push(&self, task: CaTask) {
        let mut guard = self.queue.lock().await;
        guard.0.push_back(task);
        self.sem.add_permits(1);
    }
    pub async fn contains(&self, stu_id: &str) -> bool {
        let guard = self.queue.lock().await;
        guard.0.iter().any(|t| t.stu_id == stu_id)
            || guard.1.contains_key(stu_id)
    }
    /// 获取到的元素还是会位于队列中，需要再调用 remove 才能完全从队列中删除
    pub async fn pop(&self) -> CaTask {
        let rem = self.sem.acquire().await.expect("获取信号量失败");
        // tokio 的信号量是 RAII 风格的，如果没有 .forget() 的话，信号量在 drop 后会被归还
        rem.forget();
        let mut guard = self.queue.lock().await;
        let task = guard
            .0
            .pop_front()
            .expect("获取到了信号量，但是队列为空");
        guard.1.insert(task.stu_id.clone(), ());
        task
    }
    pub async fn remove(&self, stu_id: &str) {
        let mut guard = self.queue.lock().await;
        guard.1.remove(stu_id);
    }
}
static CA_TASK_QUEUE: OnceCell<CaTaskQueue> = OnceCell::const_new();
async fn ca_task_worker(worker_id: u8) {
    async fn fetch(stu_id: &str) -> AppResult<()> {
        let rank = with_token(Ca::new(stu_id), async |token| {
            get_rank_from_ca(&token).await
        })
        .await?;
        let value = serde_json::to_string(&rank).internal_err()?;
        infra::mysql::kv_cache::insert(
            &format!("{}:{}", CA_RANK_KEY, stu_id),
            value.as_str(),
        )
        .await?;
        Ok(())
    }
    let queue = ca_task_queue().await;
    loop {
        let CaTask { stu_id, context } = queue.pop().await;
        // ca_task 宽事件 span（INTERNAL，新 trace 根）。originating_trace_id/span_id 把它
        // 关联回触发请求；其内的 with_token/hnu_call 自动嵌在本 span 之下。
        let span = tracing::info_span!(
            "ca_task",
            otel.kind = "internal",
            event_type = "ca_task",
            stu_id = %stu_id,
            worker_id = worker_id,
            originating_trace_id =
                %context.as_ref().map(|(trace_id, _)| trace_id.clone()).unwrap_or_default(),
            originating_span_id =
                %context.as_ref().map(|(_, span_id)| span_id.clone()).unwrap_or_default(),
            otel.status_code = tracing::field::Empty,
            otel.status_description = tracing::field::Empty,
        );
        if let Err(e) = fetch(&stu_id).instrument(span.clone()).await
        {
            span.record("otel.status_code", "error");
            span.record("otel.status_description", format!("{e}"));
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
    for worker_id in 0..CA_TASK_WORKER_COUNT {
        tokio::spawn(ca_task_worker(worker_id as u8));
    }
}

/// 直接从数据库中获取，如果返回 None，那么就说明正在获取中
/// 如果数据库中也不存在，且当前学号没有位于获取队列中，那么该函数将自动发起获取
#[tracing::instrument(
    fields(
        otel.kind = "internal", 
        event_type = "service", 
        cache_result = tracing::field::Empty,
    ),
    err
)]
pub async fn get_ca_rank(stu_id: &str) -> AppResult<Option<CaRank>> {
    let cache = infra::mysql::kv_cache::get(&format!(
        "{}:{}",
        CA_RANK_KEY, stu_id
    ))
    .await?;
    if let Some((value, update_at)) = cache {
        let detail: CaRankDetail =
            serde_json::from_str(&value).internal_err()?;
        utils::record!(cache_result = "hit");
        Ok(Some(CaRank { detail, update_at }))
    } else {
        refresh_ca_rank(stu_id).await?;
        utils::record!(cache_result = "miss");
        Ok(None)
    }
}

// 这个也加一个 span，可以用来统计任务队列长度的变化
#[tracing::instrument(
    fields(
        name = "ca_task_push",
        otel.kind = "internal", 
        event_type = "ca_task",
        // 是否学号已经在队列了
        duplicate = false,
    ),
    err
)]
/// 发起一个获取任务，刷新数据库中保存的信息
/// 如果已经存在于获取队列中，那么就不会重复发起
pub async fn refresh_ca_rank(stu_id: &str) -> AppResult<()> {
    let queue = ca_task_queue().await;
    if queue.contains(stu_id).await {
        utils::record!(duplicate = "true");
        return Ok(());
    }
    infra::mysql::kv_cache::delete(&format!(
        "{}:{}",
        CA_RANK_KEY, stu_id
    ))
    .await?;
    // 捕获当前请求的 OTLP trace 上下文，随任务入队，供后台 ca_task span 关联回触发请求
    let context = utils::tracing::current_trace_context();
    queue
        .push(CaTask {
            stu_id: stu_id.to_string(),
            context,
        })
        .await;
    Ok(())
}
