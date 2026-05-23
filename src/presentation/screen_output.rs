//! screen 命令进度与提示输出。

use crate::domain::ScreenFilters;

/// screen 命令头部上下文（供呈现层格式化）。
pub struct ScreenHeaderContext<'a> {
    pub kind: &'a str,
    pub sort: &'a str,
    pub pool_len: usize,
    pub days: u32,
    pub period_user_specified: bool,
}

pub fn print_screen_header(ctx: &ScreenHeaderContext<'_>) {
    println!(
        "候选池：{} 类型排行前 {}（sc={}），deep 分析窗口 {} 天（{}）",
        ctx.kind,
        ctx.pool_len,
        ctx.sort,
        ctx.days,
        if ctx.period_user_specified {
            "用户指定"
        } else {
            "与 sort 对齐"
        }
    );
}

pub fn print_filter_hint(f: &ScreenFilters, sort: &str) {
    let mut parts = Vec::new();
    if let Some(rr) = f.min_rank_return_pct {
        parts.push(format!("排行 {sort} 收益 ≥ {rr:.2}%"));
    }
    if let Some(dd) = f.max_drawdown_pct {
        parts.push(format!("最大回撤 ≤ {dd:.1}%"));
    }
    if let Some(s) = f.min_sharpe {
        parts.push(format!("夏普 ≥ {s:.2}"));
    }
    if let Some(fee) = f.max_mgmt_fee_pct {
        parts.push(format!("管理费 ≤ {fee:.2}%"));
    }
    if let Some(a) = f.min_alpha_pct {
        parts.push(format!("Alpha ≥ {a:.2}%"));
    }
    if let Some(v) = f.max_volatility_pct {
        parts.push(format!("波动率 ≤ {v:.1}%"));
    }
    if let Some(r) = f.min_total_return_pct {
        parts.push(format!("区间总收益 ≥ {r:.2}%"));
    }
    if parts.is_empty() {
        println!("筛选条件：（无额外约束，deep 分析后全部展示）");
    } else {
        println!("筛选条件：{}", parts.join("；"));
    }
    println!();
}

pub fn print_rank_prefilter(sort: &str, min_rr: f64, remaining: usize) {
    println!(
        "排行预筛（{} 区间收益 ≥ {:.2}%）：剩余 {} 只",
        sort, min_rr, remaining
    );
    println!();
}

pub fn print_deep_limit_hint(to_analyze: usize, total: usize) {
    println!(
        "deep 分析前 {} 只（共 {} 只候选；加 --full-scan 扫描全部）",
        to_analyze, total
    );
    println!();
}

pub fn print_insufficient_candidates(
    passed_len: usize,
    samples: &[(String, String, f64, f64, f64)],
) {
    println!(
        "筛选后有效样本 {} 只（需 ≥2 才能对比）；可放宽筛选条件或增大 --rank-top / --deep-limit",
        passed_len
    );
    for (code, name, ret, dd, sharpe) in samples {
        println!(
            "  {code} {name}  收益 {:.2}%  回撤 {:.2}%  夏普 {:.2}",
            ret * 100.0,
            dd * 100.0,
            sharpe
        );
    }
}

pub fn print_screen_passed(count: usize) {
    println!("通过筛选 {count} 只，展示前 {count} 只对比：");
    println!();
}
