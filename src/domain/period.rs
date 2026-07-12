//! 分析窗口预设（与 `rank --sort` / 官网区间列对齐）。

use anyhow::{Context, bail};
use chrono::{Datelike, NaiveDate};

/// 将 `--period` 解析为日历天数；未指定时返回 `days`。
pub fn resolve_analysis_days(
    period: Option<&str>,
    days: u32,
    today: NaiveDate,
) -> anyhow::Result<u32> {
    let Some(raw) = period else {
        return Ok(days);
    };
    let p = raw.trim().to_ascii_lowercase();
    if p.is_empty() {
        return Ok(days);
    }
    match p.as_str() {
        "7d" | "1w" | "week" | "zzf" => Ok(7),
        "30d" | "1m" | "month" | "1yzf" => Ok(30),
        "90d" | "3m" | "3yzf" => Ok(90),
        "180d" | "6m" | "6yzf" => Ok(180),
        "365d" | "1y" | "1n" | "1nzf" | "2nzf" | "3nzf" => Ok(365),
        "ytd" | "jnzf" => Ok(ytd_calendar_days(today)),
        "730d" | "2y" => Ok(730),
        "1095d" | "3y" => Ok(1095),
        other if other.chars().all(|c| c.is_ascii_digit()) => other
            .parse::<u32>()
            .with_context(|| format!("无法解析 period `{raw}`")),
        other => {
            bail!("未知 period `{other}`；可用 7d/1m/3m/6m/1y/ytd 或 rank 的 sc（如 1nzf、zzf）")
        }
    }
}

/// 排行 `sc` 参数对应的近似分析窗口（日历天），用于 `screen` 与 deep analyze 对齐。
pub fn days_for_rank_sort(sort: &str, today: NaiveDate) -> u32 {
    resolve_analysis_days(Some(sort.trim()), 365, today).unwrap_or(365)
}

fn ytd_calendar_days(today: NaiveDate) -> u32 {
    let jan1 = chrono::NaiveDate::from_ymd_opt(today.year(), 1, 1).unwrap();
    (today - jan1).num_days().max(1) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    const SAMPLE_TODAY: NaiveDate = NaiveDate::from_ymd_opt(2026, 5, 23).unwrap();

    #[test]
    fn resolve_period_aliases() {
        assert_eq!(
            resolve_analysis_days(Some("1m"), 30, SAMPLE_TODAY).unwrap(),
            30
        );
        assert_eq!(
            resolve_analysis_days(Some("1nzf"), 30, SAMPLE_TODAY).unwrap(),
            365
        );
        assert_eq!(resolve_analysis_days(None, 90, SAMPLE_TODAY).unwrap(), 90);
    }

    #[test]
    fn ytd_uses_injected_today() {
        assert_eq!(
            resolve_analysis_days(Some("ytd"), 30, SAMPLE_TODAY).unwrap(),
            142
        );
    }

    #[test]
    fn rank_sort_days() {
        assert_eq!(days_for_rank_sort("zzf", SAMPLE_TODAY), 7);
        assert_eq!(days_for_rank_sort("3yzf", SAMPLE_TODAY), 90);
    }
}
