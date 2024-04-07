use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
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
        let _res = update_count_file(count, &last_update).await;
        // 不去处理这个错误
        // if let Err(e) = _res {
        //     return (
        //         axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        //         format!("Failed to update count file: {}", e),
        //     )
        //         .into_response();
        // }
        state.count.store(1, Ordering::Relaxed);
        let mut last_update = state.last_update.write().await;
        *last_update = today;
    } else {
        state.count.fetch_add(1, Ordering::Relaxed);
    }

    let response = next.run(request).await;

    // 如果Response为Error，增加错误计数
    // if let Ok(body) = hyper::body::to_bytes(response.body()).await {
    //     if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&body) {
    //         if let Some(code) = json.get("code").and_then(|v| v.as_u64()) {
    //             if code != 200 {
    //                 state.err_count.fetch_add(1, Ordering::Relaxed);
    //             }
    //         }
    //     }
    // }

    response
}

async fn update_count_file(count: usize, last_update: &NaiveDate) -> tokio::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open("count.txt")
        .await?;
    let data = format!("{} {}\n", last_update, count);

    file.write_all(data.as_bytes()).await?;

    Ok(())
}
