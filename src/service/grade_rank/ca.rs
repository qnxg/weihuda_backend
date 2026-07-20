use crate::{
    error::{AppResult, ThrowInternalErrorResult},
    infra,
    service::user_state::{Ca, with_token},
    utils::{self, task_queue::UniqueTaskQueue},
};
use chrono::NaiveDateTime;
use hnu_query::ca::get_grade_rank as get_rank_from_ca;
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;
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

async fn ca_task_queue() -> &'static UniqueTaskQueue<String, CaTask> {
    static CA_TASK_QUEUE: OnceCell<UniqueTaskQueue<String, CaTask>> =
        OnceCell::const_new();
    CA_TASK_QUEUE
        .get_or_init(|| async { UniqueTaskQueue::new() })
        .await
}

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
    }
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
        duplicate = tracing::field::Empty,
    ),
    err
)]
/// 发起一个获取任务，刷新数据库中保存的信息
/// 如果已经存在于获取队列中，那么就不会重复发起
pub async fn refresh_ca_rank(stu_id: &str) -> AppResult<()> {
    let queue = ca_task_queue().await;
    // 捕获当前请求的 OTLP trace 上下文，随任务入队，供后台 ca_task span 关联回触发请求
    let context = utils::tracing::current_trace_context();
    queue
        .push(
            stu_id.to_string(),
            CaTask {
                stu_id: stu_id.to_string(),
                context,
            },
            async || {
                utils::record!(duplicate = true);
            },
            async || {
                utils::record!(duplicate = false);
                infra::mysql::kv_cache::delete(&format!(
                    "{}:{}",
                    CA_RANK_KEY, stu_id
                ))
                .await
            },
        )
        .await?;
    Ok(())
}
