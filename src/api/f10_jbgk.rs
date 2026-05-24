//! 东方财富 F10「基本概况」页（jbgk）HTML 解析。

#[derive(Debug, Clone, Default)]
pub struct FundDetailInfo {
    pub full_name: Option<String>,
    pub fund_type: String,
    pub establishment_date: String,
    pub asset_size: String,
    pub company: String,
    pub investment_target: String,
    pub investment_scope: String,
    pub investment_strategy: String,
    pub benchmark: String,
}

pub fn parse_fund_detail(html: &str) -> FundDetailInfo {
    FundDetailInfo {
        full_name: extract_td_after_th(html, "基金全称"),
        fund_type: extract_td_after_th(html, "基金类型").unwrap_or_default(),
        establishment_date: extract_td_after_th(html, "成立日期/规模")
            .or_else(|| extract_td_after_th(html, "成立日期"))
            .unwrap_or_default(),
        asset_size: extract_td_after_th(html, "净资产规模")
            .or_else(|| extract_td_after_th(html, "资产规模"))
            .unwrap_or_default(),
        company: extract_td_after_th(html, "基金管理人").unwrap_or_default(),
        investment_target: extract_tditem_after_keyword(html, "投资目标").unwrap_or_default(),
        investment_scope: extract_tditem_after_keyword(html, "投资范围").unwrap_or_default(),
        investment_strategy: extract_tditem_after_keyword(html, "投资策略").unwrap_or_default(),
        benchmark: extract_td_after_th(html, "业绩比较基准").unwrap_or_default(),
    }
}

/// `<th>标签</th><td>...</td>` 纯文本（避免子串误匹配）。
fn extract_td_after_th(html: &str, label: &str) -> Option<String> {
    let th_pat = format!("<th>{label}</th>");
    let idx = html.find(&th_pat)?;
    let tail = &html[idx + th_pat.len()..];
    let td_open = tail.find("<td")?;
    let after_open = &tail[td_open..];
    let gt = after_open.find('>')?;
    let inner = &after_open[gt + 1..];
    let close = inner.find("</td>")?;
    Some(clean_html(&inner[..close]))
}

/// 关键字后紧随的首个 `<td class="tditem">...</td>` 纯文本。
fn extract_tditem_after_keyword(html: &str, anchor: &str) -> Option<String> {
    const TDITEM: &str = "<td class=\"tditem\">";
    let idx = html.find(anchor)?;
    let tail = &html[idx..];
    let open = tail.find(TDITEM)?;
    let inner = &tail[open + TDITEM.len()..];
    let close = inner.find("</td>")?;
    Some(clean_html(&inner[..close]))
}

fn clean_html(html: &str) -> String {
    let mut result = html.to_string();
    while let Some(start) = result.find('<') {
        if let Some(end) = result[start..].find('>') {
            result.replace_range(start..start + end + 1, "");
        } else {
            break;
        }
    }
    result = result
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"");
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::parse_fund_detail;

    const SAMPLE_JBGK: &str = r#"
        <label>
            净资产规模：<span>
                26.44亿元
                （截止至：2026-03-31）</span></label>
        <table class="info w790">
            <tr><th>基金全称</th><td style='width:300px;'>华夏成长证券投资基金</td>
                <th>基金简称</th><td>华夏成长混合</td></tr>
            <tr><th>净资产规模</th><td>26.44亿元（截止至：2026年03月31日）
                <th>份额规模</th><td>25.9234亿份</td></tr>
        </table>
    "#;

    #[test]
    fn parse_fund_detail_reads_net_asset_size_from_table_th() {
        let detail = parse_fund_detail(SAMPLE_JBGK);
        assert!(
            detail.asset_size.contains("26.44亿元"),
            "unexpected asset_size: {}",
            detail.asset_size
        );
        assert!(
            !detail.asset_size.contains("华夏成长"),
            "asset_size must not pick fund name td: {}",
            detail.asset_size
        );
    }
}
