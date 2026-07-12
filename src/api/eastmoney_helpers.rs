//! 从 `pingzhongdata.js` 中提取数据的辅助函数。

/// 从 JS 内容中提取指定变量的值（变量值以分号或换行结束）。
pub(crate) fn extract_js_variable(js_content: &str, var_name: &str) -> Option<String> {
    let pattern = format!("var {} =", var_name);
    let start = js_content.find(&pattern)?;
    let start = start + pattern.len();
    let remaining = &js_content[start..];

    // 找到变量值的结束位置（分号或换行）
    let end = remaining.find(';').unwrap_or(remaining.len());
    let value = &remaining[..end].trim();

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
}
