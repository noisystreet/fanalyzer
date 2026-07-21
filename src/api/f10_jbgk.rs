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
    /// 管理费率（年化百分点，如 1.20）
    pub management_fee: Option<f64>,
    /// 托管费率（年化百分点）
    pub custody_fee: Option<f64>,
    /// 申购费率（百分点；优先天天基金优惠档）
    pub purchase_fee: Option<f64>,
    /// 最高赎回费率（百分点；阶梯费率取最高档，通常对应短持有期）
    pub redemption_fee: Option<f64>,
    /// 申购/交易状态文案（如「限大额」「开放申购」）
    pub subscribe_status: String,
    /// 赎回状态文案（如「开放赎回」）
    pub redeem_status: String,
}

pub fn parse_fund_detail(html: &str) -> FundDetailInfo {
    let purchase_raw = extract_td_after_th(html, "最高申购费率");
    let (subscribe_status, redeem_status) = parse_trade_status(html);
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
        management_fee: extract_td_after_th(html, "管理费率")
            .as_deref()
            .and_then(first_pct),
        custody_fee: extract_td_after_th(html, "托管费率")
            .as_deref()
            .and_then(first_pct),
        purchase_fee: purchase_raw.as_deref().and_then(parse_purchase_fee_pct),
        redemption_fee: extract_td_after_th(html, "最高赎回费率")
            .as_deref()
            .and_then(first_pct),
        subscribe_status,
        redeem_status,
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

/// 提取文案中首个 `数字%` 百分点。
fn first_pct(text: &str) -> Option<f64> {
    all_pcts(text).into_iter().next()
}

fn all_pcts(text: &str) -> Vec<f64> {
    let mut out = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_digit() || (chars[i] == '.' && i + 1 < chars.len()) {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            if i < chars.len() && chars[i] == '%' {
                let num: String = chars[start..i].iter().collect();
                if let Ok(v) = num.parse::<f64>() {
                    out.push(v);
                }
            }
        } else {
            i += 1;
        }
    }
    out
}

/// 申购费率：有「优惠费率」时取优惠档，否则取首个百分比。
fn parse_purchase_fee_pct(text: &str) -> Option<f64> {
    let pcts = all_pcts(text);
    if pcts.is_empty() {
        return None;
    }
    if text.contains("优惠") && pcts.len() >= 2 {
        return Some(pcts[pcts.len() - 1]);
    }
    Some(pcts[0])
}

fn parse_trade_status(html: &str) -> (String, String) {
    let Some(idx) = html.find("交易状态：") else {
        return (String::new(), String::new());
    };
    let window = &html[idx..idx.saturating_add(500).min(html.len())];
    let cleaned = clean_html(window);
    let subscribe = cleaned
        .strip_prefix("交易状态：")
        .unwrap_or(&cleaned)
        .split(['（', '('])
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    let redeem = ["开放赎回", "暂停赎回", "限制赎回", "封闭期"]
        .iter()
        .find(|s| cleaned.contains(*s))
        .map(|s| (*s).to_string())
        .unwrap_or_default();
    (subscribe, redeem)
}

#[cfg(test)]
mod tests {
    use super::{first_pct, parse_fund_detail, parse_purchase_fee_pct};

    const SAMPLE_JBGK: &str = r#"
        <label>
            净资产规模：<span>
                26.44亿元
                （截止至：2026-03-31）</span></label>
        <p class="row">
            <label>
                交易状态：<span>限大额 </span>
                <span>（<span>单日累计购买上限5.00万元</span>）</span>
                <span>开放赎回</span>
            </label>
        </p>
        <table class="info w790">
            <tr><th>基金全称</th><td style='width:300px;'>华夏成长证券投资基金</td>
                <th>基金简称</th><td>华夏成长混合</td></tr>
            <tr><th>净资产规模</th><td>26.44亿元（截止至：2026年03月31日）
                <th>份额规模</th><td>25.9234亿份</td></tr>
            <tr><th>管理费率</th><td>1.20%（每年）</td><th>托管费率</th><td>0.20%（每年）</td></tr>
            <tr><th>最高申购费率</th><td><span style="text-decoration:line-through;color:#666666">1.50%（前端）</span><br><span>天天基金优惠费率：<span style="Color:#ff0000">0.15%（前端）</span></span></td><th>最高赎回费率</th><td>1.50%（前端）</td></tr>
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

    #[test]
    fn parse_fund_detail_reads_fees_and_trade_status() {
        let detail = parse_fund_detail(SAMPLE_JBGK);
        assert_eq!(detail.management_fee, Some(1.20));
        assert_eq!(detail.custody_fee, Some(0.20));
        assert_eq!(detail.purchase_fee, Some(0.15));
        assert_eq!(detail.redemption_fee, Some(1.50));
        assert_eq!(detail.subscribe_status, "限大额");
        assert_eq!(detail.redeem_status, "开放赎回");
    }

    #[test]
    fn purchase_fee_prefers_discount_tier() {
        assert_eq!(
            parse_purchase_fee_pct("1.50%（前端）天天基金优惠费率：0.15%（前端）"),
            Some(0.15)
        );
        assert_eq!(first_pct("1.20%（每年）"), Some(1.20));
    }
}
