use crate::utils::jwt::parse_stu_id;
use axum::body::{to_bytes, Body};
use axum::extract::{Request, State};
use axum::http::header::CONTENT_TYPE;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use chrono::{Duration, Local, NaiveDateTime};
use flurry::HashMap;
use std::sync::Arc;

/// 鉴于用户量，不去考虑设置上限
pub struct Cache {
    map: HashMap<String, String>,
    counters: HashMap<String, u32>,
    last_request: HashMap<String, NaiveDateTime>, // 上一次请求的时间
    last_source: HashMap<String, NaiveDateTime>, // 上一次请求爬虫的时间
}

impl Cache {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            counters: HashMap::new(),
            last_request: HashMap::new(),
            last_source: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let map_guard = self.map.guard();
        let last_request_guard = self.last_request.guard();
        let last_source_guard = self.last_source.guard();

        let last_request =
            self.last_request.get(key, &last_request_guard).cloned();
        self.last_request.insert(
            key.to_string(),
            Local::now().naive_local(),
            &last_request_guard,
        );
        if let Some(last_request) = last_request {
            let now_request = Local::now().naive_local();
            let duration = now_request - last_request;
            // 两次请求间隔小于1小时，不使用缓存
            if duration < Duration::hours(1) {
                self.last_source.insert(
                    key.to_string(),
                    now_request,
                    &last_source_guard,
                );
                return None;
            }
        }

        let last_source =
            self.last_source.get(key, &last_source_guard).cloned();
        if let Some(last_source) = last_source {
            let now_request = Local::now().naive_local();
            let duration = now_request - last_source;
            // 两次请求爬虫间隔大于24小时，不使用缓存
            if duration > Duration::hours(24) {
                self.last_source.insert(
                    key.to_string(),
                    now_request,
                    &last_source_guard,
                );
                return None;
            }
        }

        self.map.get(key, &map_guard).cloned()
    }

    pub fn set(&self, key: String, value: String) {
        let map_guard = self.map.guard();
        let counters_guard = self.counters.guard();
        self.map.insert(key.clone(), value, &map_guard);
        self.counters.insert(key, 0, &counters_guard);
    }

    #[expect(unused)]
    pub fn increment_counter(&self, key: &str) -> Option<u32> {
        let counters_guard = self.counters.guard();
        if let Some(counter) = self.counters.get(key, &counters_guard)
        {
            let new_counter = counter + 1;
            self.counters.insert(
                key.to_string(),
                new_counter,
                &counters_guard,
            );
            Some(new_counter)
        } else {
            None
        }
    }

    #[expect(unused)]
    pub fn reset(&self, key: &str) {
        let map_guard = self.map.guard();
        let counters_guard = self.counters.guard();
        let last_request_guard = self.last_request.guard();
        self.map.remove(key, &map_guard);
        self.counters.remove(key, &counters_guard);
        self.last_request.remove(key, &last_request_guard);
    }

    /// 按照字符串前缀模糊删除缓存
    pub fn reset_prefix(&self, prefix: &str) {
        let map_guard = self.map.guard();
        let counters_guard = self.counters.guard();
        let last_request_guard = self.last_request.guard();
        for i in self.map.keys(&map_guard) {
            if i.starts_with(prefix) {
                self.map.remove(i, &map_guard);
                self.counters.remove(i, &counters_guard);
                self.last_request.remove(i, &last_request_guard);
            }
        }
    }
}

const CACHE_PATHS: [&str; 8] = [
    "/hdjw/grade",
    // "/hdjw/grade-rank",
    "/hdjw/raw-grade",
    "/hdjw/grade-rank-from-ca",
    "/hdjw/must-grade",
    "/hdjw/chart",
    "/hdjw/class-table",
    "/netflow",
    "/course", // 这是一个特殊情况，不缓存请求，需要删除class-table的缓存
];

pub async fn cache_middleware(
    State(cache): State<Arc<Cache>>,
    request: Request,
    next: Next,
) -> Response {
    // let jwt = request
    //     .headers()
    //     .get("Authorization")
    //     .unwrap()
    //     .to_str()
    //     .unwrap()
    //     .to_string();
    // request.extensions_mut().insert(jwt);
    let uri = request.uri();
    let path = uri.path();
    if CACHE_PATHS.contains(&path) {
        let jwt = request
            .headers()
            .get("Authorization")
            .map(|t| t.to_str().unwrap());
        let stu_id = parse_stu_id(jwt.unwrap()).unwrap();
        let index = format!("{stu_id}{uri}");
        // 特殊情况处理，检查是否是course请求，需要删除class-table的缓存
        if path == "/course" {
            cache.reset_prefix(&format!("{stu_id}/hdjw/class-table"));
            let response = next.run(request).await;
            return response;
        }

        return if let Some(value) = cache.get(&index) {
            // 取消掉计数更新，只采用内置的请求间隔判断
            // let counter = cache.increment_counter(&index).unwrap();
            // if counter >= 6 {
            //     cache.reset(&index);
            // }
            Response::builder()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, "application/json")
                .body(value.into())
                .unwrap()
        } else {
            let response = next.run(request).await;
            if response.status().is_success() {
                let (parts, body) = response.into_parts();
                let body_bytes =
                    to_bytes(body, usize::MAX).await.unwrap();
                let body_string =
                    String::from_utf8(body_bytes.to_vec()).unwrap();
                cache.set(index, body_string);
                return Response::from_parts(
                    parts,
                    Body::from(body_bytes),
                );
            }
            response
        };
    }
    next.run(request).await
}
