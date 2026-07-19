//! SegLock vs NewSegLock 并发对比。
//!
//! 运行：
//! ```text
//! cargo bench --bench seg_lock_bench
//! ```

#![allow(deprecated)] // 本 bench 刻意对比已弃用的 SegLock

#[path = "../src/utils/seg_lock.rs"]
mod seg_lock;

use std::sync::Arc;
use std::time::{Duration, Instant};

use rand::Rng;
use seg_lock::{NewSegLock, SegLock};
use tokio::sync::Barrier;

const CONCURRENCIES: &[usize] = &[1, 2, 3, 10, 50, 100, 300];
const HOLD_MS_MIN: u64 = 100;
const HOLD_MS_MAX: u64 = 200;
const ROUNDS: usize = 5;
/// 与线上 `USER_LOCK` 一致
const SEG_SHARDS: usize = 1000;

const YEARS: &[&str] = &["2023", "2024", "2025", "2026"];
const COLLEGES: &[&str] = &[
    "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11",
    "12", "13", "14",
];
const MAJORS: &[&str] =
    &["01", "02", "03", "04", "05", "06", "07", "08", "09"];
const CLASSES: &[&str] = &[
    "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11",
    "12",
];

fn random_stu_id(rng: &mut impl Rng) -> String {
    let year = YEARS[rng.gen_range(0..YEARS.len())];
    let college = COLLEGES[rng.gen_range(0..COLLEGES.len())];
    let major = MAJORS[rng.gen_range(0..MAJORS.len())];
    let class = CLASSES[rng.gen_range(0..CLASSES.len())];
    let seq = rng.gen_range(1..=30);
    format!("{year}{college}{major}{class}{seq:02}")
}

/// 生成 `count` 个互不重复的学号（模拟同时在线的不同用户）。
fn unique_stu_ids(count: usize) -> Vec<String> {
    let mut rng = rand::thread_rng();
    let mut ids = Vec::with_capacity(count);
    let mut seen = std::collections::HashSet::with_capacity(count);
    while ids.len() < count {
        let id = random_stu_id(&mut rng);
        if seen.insert(id.clone()) {
            ids.push(id);
        }
    }
    ids
}

fn hold_duration(rng: &mut impl Rng) -> Duration {
    Duration::from_millis(rng.gen_range(HOLD_MS_MIN..=HOLD_MS_MAX))
}

async fn bench_seg_lock(
    concurrency: usize,
    ids: &[String],
) -> Duration {
    let lock = Arc::new(SegLock::<SEG_SHARDS>::new());
    let barrier = Arc::new(Barrier::new(concurrency));
    let mut tasks = Vec::with_capacity(concurrency);

    for stu_id in ids.iter().cloned() {
        let lock = lock.clone();
        let barrier = barrier.clone();
        tasks.push(tokio::spawn(async move {
            barrier.wait().await;
            let _guard = lock.lock(&stu_id).await;
            let hold = hold_duration(&mut rand::thread_rng());
            tokio::time::sleep(hold).await;
        }));
    }

    let wall = Instant::now();
    for task in tasks {
        task.await.expect("task panicked");
    }
    wall.elapsed()
}

async fn bench_new_seg_lock(
    concurrency: usize,
    ids: &[String],
) -> Duration {
    let lock = Arc::new(NewSegLock::new());
    let barrier = Arc::new(Barrier::new(concurrency));
    let mut tasks = Vec::with_capacity(concurrency);

    for stu_id in ids.iter().cloned() {
        let lock = lock.clone();
        let barrier = barrier.clone();
        tasks.push(tokio::spawn(async move {
            barrier.wait().await;
            let _guard = lock.lock(&stu_id).await;
            let hold = hold_duration(&mut rand::thread_rng());
            tokio::time::sleep(hold).await;
        }));
    }

    let wall = Instant::now();
    for task in tasks {
        task.await.expect("task panicked");
    }
    wall.elapsed()
}

fn fmt_ms(d: Duration) -> String {
    format!("{:.1}", d.as_secs_f64() * 1000.0)
}

fn mean(samples: &[Duration]) -> Duration {
    let total: Duration = samples.iter().sum();
    total / samples.len() as u32
}

fn stddev_ms(samples: &[Duration], avg: Duration) -> f64 {
    if samples.len() < 2 {
        return 0.0;
    }
    let avg_ms = avg.as_secs_f64() * 1000.0;
    let var = samples
        .iter()
        .map(|d| {
            let x = d.as_secs_f64() * 1000.0 - avg_ms;
            x * x
        })
        .sum::<f64>()
        / (samples.len() as f64 - 1.0);
    var.sqrt()
}

#[tokio::main]
async fn main() {
    println!("SegLock<{SEG_SHARDS}> vs NewSegLock");
    println!(
        "hold=[{HOLD_MS_MIN},{HOLD_MS_MAX}]ms  rounds={ROUNDS}  ids=unique stu_id per task"
    );
    println!(
        "stu_id = <year 2023-2026><college 01-14><major 01-09><class 01-12><seq 01-30>"
    );
    println!();
    println!(
        "{:<6} {:>14} {:>14} {:>14} {:>10}",
        "conc",
        "SegLock(ms)",
        "NewSegLock(ms)",
        "delta(ms)",
        "speedup"
    );
    println!("{}", "-".repeat(64));

    for &conc in CONCURRENCIES {
        let mut seg_samples = Vec::with_capacity(ROUNDS);
        let mut new_samples = Vec::with_capacity(ROUNDS);

        for _ in 0..ROUNDS {
            // 同一轮用同一批学号，保证对比公平
            let ids = unique_stu_ids(conc);
            seg_samples.push(bench_seg_lock(conc, &ids).await);
            new_samples.push(bench_new_seg_lock(conc, &ids).await);
        }

        let seg_avg = mean(&seg_samples);
        let new_avg = mean(&new_samples);
        let (delta_sign, delta) = if seg_avg >= new_avg {
            ("-", seg_avg - new_avg)
        } else {
            ("+", new_avg - seg_avg)
        };
        let speedup = seg_avg.as_secs_f64() / new_avg.as_secs_f64();

        println!(
            "{:<6} {:>8}±{:<4.0} {:>8}±{:<4.0} {:>6}{:<7} {:>10.2}x",
            conc,
            fmt_ms(seg_avg),
            stddev_ms(&seg_samples, seg_avg),
            fmt_ms(new_avg),
            stddev_ms(&new_samples, new_avg),
            delta_sign,
            fmt_ms(delta),
            speedup,
        );
    }
}
