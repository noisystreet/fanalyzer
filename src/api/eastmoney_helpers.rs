//! 从 `pingzhongdata.js` 中提取数据的辅助函数。

use super::eastmoney_types::PeerRankSnapshot;
use chrono::{TimeZone, Utc};

/// 从 JS 内容中提取指定变量的值（变量值以分号结束；兼容 `var x =` / `var x=`）。
pub(crate) fn extract_js_variable(js_content: &str, var_name: &str) -> Option<String> {
    let patterns = [format!("var {var_name} ="), format!("var {var_name}=")];
    let (start, pattern_len) = patterns.iter().find_map(|pattern| {
        js_content
            .find(pattern.as_str())
            .map(|idx| (idx, pattern.len()))
    })?;
    let remaining = &js_content[start + pattern_len..];

    // 找到变量值的结束位置（分号）
    let end = remaining.find(';').unwrap_or(remaining.len());
    let value = remaining[..end].trim();

    Some(value.to_string())
}

/// 从 JS 内容中提取 `var name="value"` 格式的字符串值。
pub(crate) fn extract_js_string_value(js_content: &str, var_name: &str) -> Option<String> {
    // 去除可能的 UTF-8 BOM
    let content = js_content.strip_prefix('\u{feff}').unwrap_or(js_content);

    let pattern = format!("var {}=\"", var_name);
    let start = content.find(&pattern)?;
    let start = start + pattern.len();
    let remaining = &content[start..];

    // 找到引号结束位置
    let end = remaining.find('"')?;
    Some(remaining[..end].to_string())
}

/// 解析 "14年又138天" 格式的工作时间，返回天数。
pub(crate) fn parse_work_time(work_time: &str) -> i32 {
    let mut days = 0i32;

    // 提取年数
    if let Some(year_idx) = work_time.find("年")
        && let Ok(years) = work_time[..year_idx].trim().parse::<i32>()
    {
        days += years * 365;
    }

    // 提取天数
    if let Some(day_start) = work_time.find("又")
        && let Some(day_end) = work_time.find("天")
    {
        let day_str = &work_time[day_start + 3..day_end]; // "又" 是3字节UTF-8
        if let Ok(d) = day_str.trim().parse::<i32>() {
            days += d;
        }
    }

    days
}

/// 解析 pingzhongdata 中近 3 月同类排名最新一点。
pub(crate) fn parse_peer_rank_snapshot(js_content: &str) -> PeerRankSnapshot {
    let mut out = PeerRankSnapshot::default();

    if let Some(raw) = extract_js_variable(js_content, "Data_rateInSimilarType")
        && let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(&raw)
        && let Some(last) = arr.last()
    {
        out.rank = json_u32(last.get("y"));
        out.peer_count = json_u32(last.get("sc"));
        if let Some(ms) = last.get("x").and_then(|v| v.as_i64()) {
            out.as_of = Some(ms_to_date(ms));
        }
    }

    if let Some(raw) = extract_js_variable(js_content, "Data_rateInSimilarPersent")
        && let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(&raw)
        && let Some(last) = arr.last()
        && let Some(pair) = last.as_array()
    {
        out.percentile = pair.get(1).and_then(|v| v.as_f64());
        if out.as_of.is_none()
            && let Some(ms) = pair.first().and_then(|v| v.as_i64())
        {
            out.as_of = Some(ms_to_date(ms));
        }
    }

    out
}

fn json_u32(v: Option<&serde_json::Value>) -> Option<u32> {
    let v = v?;
    if let Some(n) = v.as_u64() {
        return u32::try_from(n).ok();
    }
    if let Some(n) = v.as_f64() {
        return Some(n.round() as u32);
    }
    v.as_str()?.parse().ok()
}

fn ms_to_date(ms: i64) -> String {
    Utc.timestamp_millis_opt(ms)
        .single()
        .map(|dt| dt.date_naive().to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_work_time_years_only() {
        assert_eq!(parse_work_time("5年"), 5 * 365);
    }

    #[test]
    fn test_parse_work_time_years_and_days() {
        assert_eq!(parse_work_time("3年又120天"), 3 * 365 + 120);
    }

    #[test]
    fn test_parse_work_time_empty() {
        assert_eq!(parse_work_time(""), 0);
    }

    #[test]
    fn test_extract_js_string_value_basic() {
        let js = "var fS_name=\"测试基金\";";
        assert_eq!(
            extract_js_string_value(js, "fS_name"),
            Some("测试基金".into())
        );
    }

    #[test]
    fn test_extract_js_string_value_with_bom() {
        let js = "\u{feff}var fS_name=\"测试基金\";";
        assert_eq!(
            extract_js_string_value(js, "fS_name"),
            Some("测试基金".into())
        );
    }

    #[test]
    fn test_extract_js_string_value_not_found() {
        let js = "var other=\"value\";";
        assert_eq!(extract_js_string_value(js, "fS_name"), None);
    }

    #[test]
    fn test_extract_js_variable() {
        let js = "var Data_currentFundManager = [{\"name\":\"张三\"}];";
        let result = extract_js_variable(js, "Data_currentFundManager");
        assert!(result.is_some());
        assert!(result.unwrap().contains("张三"));
    }

    #[test]
    fn parse_peer_rank_takes_latest_point() {
        let js = r#"
var Data_rateInSimilarType = [{"x":1357228800000,"y":27,"sc":"59"},{"x":1784476800000,"y":75,"sc":"92"}];
var Data_rateInSimilarPersent = [[1357228800000,54.24],[1784476800000,18.48]];
"#;
        let snap = parse_peer_rank_snapshot(js);
        assert_eq!(snap.rank, Some(75));
        assert_eq!(snap.peer_count, Some(92));
        assert_eq!(snap.percentile, Some(18.48));
        assert_eq!(snap.as_of.as_deref(), Some("2026-07-19"));
    }

    #[test]
    fn parse_peer_rank_accepts_var_without_space_before_eq() {
        let js = r#"
var Data_rateInSimilarType =[{"x":1,"y":2,"sc":"3"}];
var Data_rateInSimilarPersent=[[1,88.5]];
"#;
        let snap = parse_peer_rank_snapshot(js);
        assert_eq!(snap.rank, Some(2));
        assert_eq!(snap.peer_count, Some(3));
        assert_eq!(snap.percentile, Some(88.5));
    }

    #[test]
    fn parse_peer_rank_missing_is_default() {
        let snap = parse_peer_rank_snapshot("var fS_name=\"x\";");
        assert!(snap.rank.is_none());
        assert!(snap.peer_count.is_none());
        assert!(snap.percentile.is_none());
    }
}
