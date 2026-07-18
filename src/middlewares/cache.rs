use chrono::{Duration, Local, NaiveDateTime};
use flurry::HashMap;
use reqwest::StatusCode;
use salvo::{
    Depot, FlowCtrl, Request, Response, handler, http::ResBody,
};
use tokio::sync::OnceCell;

use crate::utils;

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

const CACHE_PATHS: [&str; 6] = [
    "/hdjw/grade",
    // "/hdjw/grade-rank",
    "/hdjw/raw-grade",
    "/hdjw/must-grade",
    "/hdjw/class-table",
    "/netflow",
    "/course", // 这是一个特殊情况，不缓存请求，需要删除class-table的缓存
];

static CACHE: OnceCell<Cache> = OnceCell::const_new();

#[handler]
pub async fn cache_middleware(
    req: &mut Request,
    resp: &mut Response,
    ctrl: &mut FlowCtrl,
    depot: &mut Depot,
) {
    // let jwt = request
    //     .headers()
    //     .get("Authorization")
    //     .unwrap()
    //     .to_str()
    //     .unwrap()
    //     .to_string();
    // request.extensions_mut().insert(jwt);
    let uri = req.uri();
    let path = uri.path();
    if CACHE_PATHS.contains(&path) {
        let Some(jwt) = req
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
        else {
            ctrl.call_next(req, depot, resp).await;
            return;
        };
        let Ok(stu_id) = utils::jwt::parse(jwt) else {
            ctrl.call_next(req, depot, resp).await;
            return;
        };
        let index = format!("{stu_id}{uri}");
        let cache = CACHE.get_or_init(async || Cache::new()).await;
        if path == "/course" {
            // 特殊情况处理，检查是否是course请求，需要删除class-table的缓存
            cache.reset_prefix(&format!("{stu_id}/hdjw/class-table"));
            ctrl.call_next(req, depot, resp).await;
            return;
        }
        if let Some(cached_value) = cache.get(&index) {
            utils::record!(cache_result = "hit");
            resp.headers_mut().insert(
                salvo::http::header::CONTENT_TYPE,
                salvo::http::HeaderValue::from_static(
                    "application/json",
                ),
            );
            resp.render(cached_value);
            ctrl.skip_rest();
        } else {
            utils::record!(cache_result = "miss");
            ctrl.call_next(req, depot, resp).await;
            if let Some(StatusCode::OK) = resp.status_code {
                // if let Ok(body) = .await {
                //     cache.set(index, body);
                // }
                let body = match resp.body {
                    ResBody::Once(ref data) => {
                        let bytes = data.to_vec();
                        String::from_utf8(bytes.to_vec())
                    }
                    _ => return,
                };
                if let Ok(body) = body {
                    cache.set(index, body);
                }
            }
        }
    } else {
        ctrl.call_next(req, depot, resp).await;
    }
}
