#![allow(dead_code)]
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use chrono::{Local, NaiveDate};
use tokio::io::AsyncReadExt;
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
        // 应对极端情况，虽然没有任何可能出现。
        if count == 0 {
            state.count.store(1, Ordering::Relaxed);
        }   
        let err_count = state.err_count.load(Ordering::Relaxed);
        let _res = update_count_file(count, err_count, &last_update).await; // 不去处理这个错误
        if _res.is_err() {
            tracing::error!("更新计数文件失败");
        }
        state.count.store(1, Ordering::Relaxed);
        state.err_count.store(0, Ordering::Relaxed);    // 每日重置错误计数
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
        .read(true)
        .write(true)
        .create(true)
        .append(true)
        .open("count.txt")
        .await?;

    // 先读取文件最后一行的内容，如果日期与当前的last_update相同就跳过修改
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;

    // 获取倒数第二行
    let second_last_line = contents.lines().next_back().unwrap_or("");

    // 如果倒数第二行的日期与last_update相同，就跳过修改
    if second_last_line.starts_with(&last_update.to_string()) {
        return Ok(());
    }

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
