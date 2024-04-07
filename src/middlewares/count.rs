#![allow(dead_code)]
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use chrono::{Local, NaiveDate};
use tokio::{fs::OpenOptions, io::AsyncWriteExt, sync::RwLock};

pub struct Count {
    pub count: AtomicUsize,
    pub err_count: AtomicUsize,
    pub last_update: RwLock<NaiveDate>,
}

pub async fn count_middleware(
    State(state): State<Arc<Count>>,
    request: Request,
    next: Next,
) -> Response {
    let today = Local::now().naive_local().date();

    let last_update = *state.last_update.read().await;

    if today != last_update {
        let count = state.count.load(Ordering::Relaxed);
        let err_count = state.err_count.load(Ordering::Relaxed);
        let _res = update_count_file(count, err_count, &last_update).await; // 不去处理这个错误
        if _res.is_err() {
            tracing::error!("更新计数文件失败");
        }
        state.count.store(1, Ordering::Relaxed);
        let mut last_update = state.last_update.write().await;
        *last_update = today;
    } else {
        state.count.fetch_add(1, Ordering::Relaxed);
    }

    let response = next.run(request).await;

    // 如果Response为Error，增加错误计数，因为错误远比正确少，使用错误来计数减少性能消耗
    if !response.status().is_success() {
        state.err_count.fetch_add(1, Ordering::Relaxed);
    }

    response
}

async fn update_count_file(
    count: usize,
    err_count: usize,
    last_update: &NaiveDate,
) -> tokio::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open("count.txt")
        .await?;
    let data = format!(
        "{} total: {}, success: {}, rate: {}%\n",
        last_update,
        count,
        count - err_count,
        100 - err_count * 100 / count
    );

    file.write_all(data.as_bytes()).await?;

    Ok(())
}
