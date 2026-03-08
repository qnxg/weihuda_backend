use chrono::Timelike;
use salvo::prelude::*;
use serde_json::{Value, json};
use std::{
    fs::{File, OpenOptions, create_dir},
    io::{Read, Write},
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
use tokio::time::{Duration, interval};

#[derive(Default)]
struct TrafficStats {
    total_requests: AtomicUsize,
    max_qps: AtomicUsize,
    current_qps: AtomicUsize,
    requests_in_current_second: AtomicUsize,
}

pub struct TrafficAnalyzerMiddleware {
    stats: Arc<TrafficStats>,
}

impl TrafficAnalyzerMiddleware {
    #[expect(clippy::unwrap_used)] // FIXME
    pub fn new() -> Self {
        let stats = Arc::new(TrafficStats::default());
        let stats_clone_1 = stats.clone();
        let stats_clone_2 = stats.clone();

        // 每秒更新当前 QPS
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));
            interval.set_missed_tick_behavior(
                tokio::time::MissedTickBehavior::Delay,
            );
            loop {
                interval.tick().await;
                let current_qps = stats_clone_1
                    .requests_in_current_second
                    .swap(0, Ordering::Relaxed);
                stats_clone_1
                    .current_qps
                    .store(current_qps, Ordering::Relaxed);
                let max_qps =
                    stats_clone_1.max_qps.load(Ordering::Relaxed);
                if current_qps > max_qps {
                    stats_clone_1
                        .max_qps
                        .store(current_qps, Ordering::Relaxed);
                }
            }
        });

        // 每小时记录平均 QPS 和最高 QPS
        tokio::spawn(async move {
            // 计算距离下一个整点的秒数，然后等待
            let mut now = chrono::Local::now();
            let sleep_time =
                (3600 - now.minute() * 60 - now.second()) as u64;
            tokio::time::sleep(Duration::from_secs(sleep_time)).await;
            let mut interval = interval(Duration::from_secs(3600));
            interval.set_missed_tick_behavior(
                tokio::time::MissedTickBehavior::Delay,
            );
            loop {
                interval.tick().await;
                let total_requests = stats_clone_2
                    .total_requests
                    .swap(0, Ordering::Relaxed);

                let new_now = chrono::Local::now();
                let new_now_time_str =
                    new_now.format("%Y.%m.%d %H:%M:%S").to_string();
                let old_now_time_str =
                    now.format("%Y.%m.%d %H:%M:%S").to_string();
                let time_str = format!(
                    "{} - {}",
                    old_now_time_str, new_now_time_str
                );
                let seconds = (new_now - now).num_seconds();
                let avg_qps = total_requests / seconds as usize;
                let max_qps =
                    stats_clone_2.max_qps.swap(0, Ordering::Relaxed);

                let log_entry = json!({
                    "time": time_str,
                    "average_qps": avg_qps,
                    "max_qps": max_qps,
                });

                let date_str = now.format("%Y-%m-%d").to_string();
                let dir_name = "traffic_stats";
                let file_path =
                    format!("{}/{}.json", dir_name, date_str);

                // 更新上个时间点
                now = new_now;

                // 检查并创建目录
                if !Path::new(dir_name).exists() {
                    create_dir(dir_name).unwrap();
                }
                let mut file_content = String::new();
                if Path::new(&file_path).exists() {
                    let mut file = File::open(&file_path).unwrap();
                    file.read_to_string(&mut file_content).unwrap();
                }

                let mut log_entries: Vec<Value> =
                    if file_content.is_empty() {
                        Vec::new()
                    } else {
                        serde_json::from_str(&file_content).unwrap()
                    };
                log_entries.push(log_entry);
                let log_entries_pretty =
                    serde_json::to_string_pretty(&log_entries)
                        .unwrap();

                let mut file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&file_path)
                    .expect("打开日志文件失败");
                writeln!(file, "{}", log_entries_pretty)
                    .expect("写入日志失败");
            }
        });
        Self { stats }
    }
}

#[async_trait]
impl Handler for TrafficAnalyzerMiddleware {
    async fn handle(
        &self,
        req: &mut Request,
        depot: &mut Depot,
        res: &mut Response,
        ctrl: &mut FlowCtrl,
    ) {
        self.stats.total_requests.fetch_add(1, Ordering::Relaxed);
        self.stats
            .requests_in_current_second
            .fetch_add(1, Ordering::Relaxed);
        ctrl.call_next(req, depot, res).await;
    }
}
