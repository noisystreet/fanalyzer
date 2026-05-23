//! F10 脚本片段 `var apidata={ content:"..."};` 中的 HTML 正文提取。

/// 取出 `content:"..."` 内嵌 HTML（至 `",arryear:` 为止）。
pub fn extract_apidata_content(body: &str) -> Result<&str, String> {
    let key = "content:\"";
    let start = body
        .trim()
        .find(key)
        .ok_or_else(|| "F10 apidata: missing content:\"".to_string())?
        + key.len();
    let tail = &body[start..];
    let end = tail
        .find("\",arryear:")
        .ok_or_else(|| "F10 apidata: missing \",arryear:\"".to_string())?;
    Ok(&tail[..end])
}

#[cfg(test)]
mod tests {
    use super::extract_apidata_content;

    #[test]
    fn extract_content_ok() {
        let js = r#"var apidata={ content:"<p>hi</p>",arryear:[2026]};"#;
        assert_eq!(extract_apidata_content(js).unwrap(), "<p>hi</p>");
    }
}
