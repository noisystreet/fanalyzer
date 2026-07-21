//! 东方财富 F10「基金经理」页（jjjl）HTML 解析。

/// 现任基金经理上任信息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagerAppointment {
    pub name: String,
    /// 本基金上任日期（YYYY-MM-DD）
    pub start_date: String,
}

/// 解析「现任基金经理简介」中的姓名与上任日期。
pub fn parse_current_managers(html: &str) -> Vec<ManagerAppointment> {
    let mut out = Vec::new();
    let mut rest = html;
    const NAME_MARK: &str = "姓名：</strong>";
    const DATE_MARK: &str = "上任日期：</strong>";

    while let Some(name_idx) = rest.find(NAME_MARK) {
        let after_name = &rest[name_idx + NAME_MARK.len()..];
        let Some(name) = extract_name_after_label(after_name) else {
            rest = &after_name[1.min(after_name.len())..];
            continue;
        };
        let Some(date_rel) = after_name.find(DATE_MARK) else {
            rest = after_name;
            continue;
        };
        let after_date = &after_name[date_rel + DATE_MARK.len()..];
        let start_date = after_date.chars().take(10).collect::<String>();
        if start_date.len() == 10 && start_date.as_bytes()[4] == b'-' {
            out.push(ManagerAppointment { name, start_date });
        }
        rest = after_date;
    }
    out
}

fn extract_name_after_label(after_name: &str) -> Option<String> {
    // 常见：<a ...>张坤</a> 或纯文本
    if let Some(a_idx) = after_name.find("<a") {
        let from_a = &after_name[a_idx..];
        let gt = from_a.find('>')?;
        let inner = &from_a[gt + 1..];
        let close = inner.find('<')?;
        let name = inner[..close].trim();
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }
    let end = after_name.find('<').unwrap_or(after_name.len().min(32));
    let name = after_name[..end].trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn parse_sample_jjjl_fixture() {
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/api/jjjl_sample.html");
        let html = fs::read_to_string(path).expect("jjjl fixture");
        let managers = parse_current_managers(&html);
        assert_eq!(managers.len(), 2);
        assert_eq!(managers[0].name, "张坤");
        assert_eq!(managers[0].start_date, "2012-09-28");
        assert_eq!(managers[1].name, "彭珂");
        assert_eq!(managers[1].start_date, "2026-06-27");
    }

    #[test]
    fn parse_inline_snippet() {
        let html = r#"
<div class="jl_intro"><p><strong>姓名：</strong><a href="x">甲</a></p><p><strong>上任日期：</strong>2020-01-01</p></div>
<div class="jl_intro"><p><strong>姓名：</strong><a href="y">乙</a></p><p><strong>上任日期：</strong>2021-06-15</p></div>
"#;
        let managers = parse_current_managers(html);
        assert_eq!(
            managers,
            vec![
                ManagerAppointment {
                    name: "甲".into(),
                    start_date: "2020-01-01".into(),
                },
                ManagerAppointment {
                    name: "乙".into(),
                    start_date: "2021-06-15".into(),
                },
            ]
        );
    }
}
