//! 应用层有界并发（单任务内 `tokio::join!`，无需 spawn）。

use std::future::Future;

/// 多基金分析/对比默认并发度（兼顾吞吐与数据源礼貌）。
pub const FUND_CONCURRENCY: usize = 4;

/// 对 `items` 最多 `limit` 路并发执行 `f`，保持结果顺序与输入一致。
pub async fn map_concurrent<F, Fut, T>(items: &[String], limit: usize, mut f: F) -> Vec<T>
where
    F: FnMut(String) -> Fut,
    Fut: Future<Output = T>,
{
    if items.is_empty() {
        return Vec::new();
    }
    let limit = limit.clamp(1, 8);
    let mut out = Vec::with_capacity(items.len());
    for chunk in items.chunks(limit) {
        out.extend(join_chunk(chunk, &mut f).await);
    }
    out
}

async fn join_chunk<F, Fut, T>(chunk: &[String], f: &mut F) -> Vec<T>
where
    F: FnMut(String) -> Fut,
    Fut: Future<Output = T>,
{
    match chunk.len() {
        0 => vec![],
        1 => vec![f(chunk[0].clone()).await],
        2 => {
            let (ra, rb) = tokio::join!(f(chunk[0].clone()), f(chunk[1].clone()));
            vec![ra, rb]
        }
        3 => {
            let (ra, rb, rc) = tokio::join!(
                f(chunk[0].clone()),
                f(chunk[1].clone()),
                f(chunk[2].clone())
            );
            vec![ra, rb, rc]
        }
        _ => {
            let (ra, rb, rc, rd) = tokio::join!(
                f(chunk[0].clone()),
                f(chunk[1].clone()),
                f(chunk[2].clone()),
                f(chunk[3].clone())
            );
            vec![ra, rb, rc, rd]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Instant;

    #[tokio::test]
    async fn map_concurrent_preserves_order() {
        let items: Vec<String> = (0..5).map(|i| i.to_string()).collect();
        let out = map_concurrent(&items, 2, |s| async move { s.parse::<i32>().unwrap() }).await;
        assert_eq!(out, vec![0, 1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn map_concurrent_runs_in_parallel_within_chunk() {
        let active = Arc::new(AtomicUsize::new(0));
        let max_active = Arc::new(AtomicUsize::new(0));
        let items = vec!["a".into(), "b".into(), "c".into(), "d".into()];
        let _ = map_concurrent(&items, 4, {
            let active = active.clone();
            let max_active = max_active.clone();
            move |_s| {
                let active = active.clone();
                let max_active = max_active.clone();
                async move {
                    let now = active.fetch_add(1, Ordering::SeqCst) + 1;
                    max_active.fetch_max(now, Ordering::SeqCst);
                    tokio::time::sleep(std::time::Duration::from_millis(30)).await;
                    active.fetch_sub(1, Ordering::SeqCst);
                    now
                }
            }
        })
        .await;
        assert!(max_active.load(Ordering::SeqCst) >= 2);
    }

    #[tokio::test]
    async fn map_concurrent_faster_than_serial_for_io_bound_work() {
        let items: Vec<String> = (0..4).map(|i| i.to_string()).collect();
        let started = Instant::now();
        map_concurrent(&items, 4, |_s| async {
            tokio::time::sleep(std::time::Duration::from_millis(40)).await;
        })
        .await;
        let elapsed = started.elapsed();
        assert!(
            elapsed < std::time::Duration::from_millis(120),
            "expected parallel chunk, got {:?}",
            elapsed
        );
    }
}
