//! 天天基金开放式基金排行页 `rankhandler.aspx` 响应解析（需浏览器同源 Referer）。

/// 一行排行（字段含义与官网表格列顺序一致，百分比列为「百分点」如 9.45 表示 9.45%）。
#[derive(Debug, Clone)]
pub struct FundRankEntry {
    pub code: String,
    pub name: String,
    pub nav_date: String,
    pub unit_nav: Option<f64>,
    pub acc_nav: Option<f64>,
    /// 日增长率等（百分点）
    pub pct_day: Option<f64>,
    pub pct_week: Option<f64>,
    pub pct_month: Option<f64>,
    pub pct_3m: Option<f64>,
    pub pct_6m: Option<f64>,
    pub pct_1y: Option<f64>,
    pub pct_2y: Option<f64>,
    pub pct_3y: Option<f64>,
    pub pct_this_year: Option<f64>,
    pub pct_since_start: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct FundRankingPage {
    pub rows: Vec<FundRankEntry>,
    pub total_records: u32,
}

fn parse_pct_cell(raw: &str) -> Option<f64> {
    let t = raw.trim();
    if t.is_empty() || t == "--" {
        return None;
    }
    if let Some(v) = t.strip_suffix('%') {
        return v.trim().parse::<f64>().ok();
    }
    t.parse::<f64>().ok()
}

fn parse_nav_cell(raw: &str) -> Option<f64> {
    let t = raw.trim();
    if t.is_empty() || t == "--" {
        return None;
    }
    t.parse::<f64>().ok()
}

pub fn parse_rank_row(csv_line: &str) -> Option<FundRankEntry> {
    let cols: Vec<&str> = csv_line.split(',').collect();
    if cols.len() < 6 {
        return None;
    }
    let mut idx = 6;
    let mut next_pct = || {
        let v = cols.get(idx).map(|s| parse_pct_cell(s)).unwrap_or(None);
        idx += 1;
        v
    };

    Some(FundRankEntry {
        code: cols[0].trim().to_string(),
        name: cols[1].trim().to_string(),
        nav_date: cols[3].trim().to_string(),
        unit_nav: parse_nav_cell(cols[4]),
        acc_nav: parse_nav_cell(cols[5]),
        pct_day: next_pct(),
        pct_week: next_pct(),
        pct_month: next_pct(),
        pct_3m: next_pct(),
        pct_6m: next_pct(),
        pct_1y: next_pct(),
        pct_2y: next_pct(),
        pct_3y: next_pct(),
        pct_this_year: next_pct(),
        pct_since_start: next_pct(),
    })
}

/// 拆分 `datas:["a","b"]` 段内的 JSON 字符串（记录内含逗号，不能整段按逗号切）。
fn split_datas_records(array_inner: &str) -> Vec<String> {
    let s = array_inner.trim();
    let s = s.strip_prefix('"').unwrap_or(s);
    let s = s.strip_suffix('"').unwrap_or(s);
    if s.is_empty() {
        return Vec::new();
    }
    s.split("\",\"")
        .map(std::string::ToString::to_string)
        .collect()
}

fn extract_all_records(body: &str) -> Option<u32> {
    let key = "],allRecords:";
    let i = body.find(key)? + key.len();
    let rest = &body[i..];
    let num: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    num.parse().ok()
}

/// 解析 `var rankData = { datas:[...], allRecords: N, ...};`
pub fn parse_rankhandler_body(body: &str) -> Result<FundRankingPage, String> {
    let datas_start = body
        .find("datas:[")
        .ok_or_else(|| "rankhandler: missing datas:[".to_string())?;
    let arr_start = datas_start + "datas:[".len();
    let rel = &body[arr_start..];
    let arr_end_rel = rel
        .find("],allRecords")
        .ok_or_else(|| "rankhandler: missing ],allRecords".to_string())?;
    let array_inner = &rel[..arr_end_rel];
    let total_records = extract_all_records(body).unwrap_or(0);

    let records = split_datas_records(array_inner);
    let rows: Vec<FundRankEntry> = records.iter().filter_map(|r| parse_rank_row(r)).collect();

    Ok(FundRankingPage {
        rows,
        total_records,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sample_rankhandler() {
        let js = r#"var rankData = {datas:["001761,广发安宏回报混合A,GFAHHBHHA,2026-05-06,1.1233,1.4782,9.45,14.39,19.18,13.05,16.36,46.99,32.68,10.88,15.91,48.04,2015-12-30,1,48.036048,1.20%,0.12%,1,0.12%,1,-1.96"],allRecords:9130,pageIndex:1};"#;
        let page = parse_rankhandler_body(js).unwrap();
        assert_eq!(page.total_records, 9130);
        assert_eq!(page.rows.len(), 1);
        let r = &page.rows[0];
        assert_eq!(r.code, "001761");
        assert_eq!(r.name, "广发安宏回报混合A");
        assert!((r.pct_week.unwrap() - 14.39).abs() < 1e-6);
        assert!((r.pct_1y.unwrap() - 46.99).abs() < 1e-6);
    }
}
