//! SVG 折线图渲染（纯 SSR，无外部依赖）。

use crate::models::SeriesPoint;

#[derive(Debug, Clone, Copy)]
pub enum ChartValueKind {
    /// 小数比率 → 百分比显示（如 0.12 → 12%）
    Percent,
    /// 原始数值（如夏普、Beta、归一化净值）
    Number,
}

/// 将时间序列渲染为内联 SVG 折线图。
pub fn svg_line_chart(
    points: &[SeriesPoint],
    width: u32,
    height: u32,
    title: &str,
    kind: ChartValueKind,
) -> String {
    if points.len() < 2 {
        return format!(r#"<p class="muted">"{title}" 数据点不足，无法绘图。</p>"#);
    }

    let pad_l = 48u32;
    let pad_r = 12u32;
    let pad_t = 28u32;
    let pad_b = 32u32;
    let plot_w = width.saturating_sub(pad_l + pad_r).max(1);
    let plot_h = height.saturating_sub(pad_t + pad_b).max(1);

    let values: Vec<f64> = points.iter().map(|p| p.value).collect();
    let mut y_min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let mut y_max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    if (y_max - y_min).abs() < 1e-12 {
        y_min -= 0.01;
        y_max += 0.01;
    }
    let y_span = y_max - y_min;

    let n = points.len();
    let coords: Vec<(f64, f64)> = values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let x = pad_l as f64 + (i as f64 / (n - 1) as f64) * plot_w as f64;
            let y = pad_t as f64 + (1.0 - (v - y_min) / y_span) * plot_h as f64;
            (x, y)
        })
        .collect();

    let polyline: String = coords
        .iter()
        .map(|(x, y)| format!("{x:.1},{y:.1}"))
        .collect::<Vec<_>>()
        .join(" ");

    let y0_label = format_value(y_max, kind);
    let y1_label = format_value(y_min, kind);
    let x0 = points
        .first()
        .map(|p| p.date.format("%m-%d").to_string())
        .unwrap_or_default();
    let x1 = points
        .last()
        .map(|p| p.date.format("%m-%d").to_string())
        .unwrap_or_default();

    format!(
        r#"<svg class="chart-svg" viewBox="0 0 {width} {height}" role="img" aria-label="{title}">
<title>{title}</title>
<text class="chart-title" x="{pad_l}" y="18">{title}</text>
<line class="chart-axis" x1="{pad_l}" y1="{pad_t}" x2="{pad_l}" y2="{y_bottom}"/>
<line class="chart-axis" x1="{pad_l}" y1="{y_bottom}" x2="{x_right}" y2="{y_bottom}"/>
<text class="chart-tick" x="4" y="{y_top}">{y0_label}</text>
<text class="chart-tick" x="4" y="{y_bottom}">{y1_label}</text>
<text class="chart-tick" x="{pad_l}" y="{height}">{x0}</text>
<text class="chart-tick" x="{x_right}" y="{height}" text-anchor="end">{x1}</text>
<polyline class="chart-line" fill="none" points="{polyline}"/>
</svg>"#,
        width = width,
        height = height,
        title = escape_xml(title),
        pad_l = pad_l,
        pad_t = pad_t,
        y_bottom = pad_t + plot_h,
        x_right = pad_l + plot_w,
        y_top = pad_t + 4,
        y0_label = escape_xml(&y0_label),
        y1_label = escape_xml(&y1_label),
        x0 = escape_xml(&x0),
        x1 = escape_xml(&x1),
        polyline = polyline,
    )
}

fn format_value(v: f64, kind: ChartValueKind) -> String {
    match kind {
        ChartValueKind::Percent => format!("{:.1}%", v * 100.0),
        ChartValueKind::Number => format!("{:.2}", v),
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn sample_points() -> Vec<SeriesPoint> {
        (0..5)
            .map(|i| SeriesPoint {
                date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap() + chrono::Duration::days(i),
                value: 1.0 + i as f64 * 0.01,
            })
            .collect()
    }

    #[test]
    fn svg_contains_polyline() {
        let svg = svg_line_chart(&sample_points(), 400, 160, "测试", ChartValueKind::Number);
        assert!(svg.contains("<polyline"));
        assert!(svg.contains("chart-line"));
    }

    #[test]
    fn svg_insufficient_points() {
        let svg = svg_line_chart(
            &[SeriesPoint {
                date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                value: 1.0,
            }],
            400,
            160,
            "空",
            ChartValueKind::Number,
        );
        assert!(svg.contains("数据点不足"));
    }
}
