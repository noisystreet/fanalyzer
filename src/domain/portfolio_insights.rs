//! 组合分析结果规则解读（纯函数，无 IO）。

use crate::models::{
    CorrelationMatrix, InsightLevel, OverlapPair, PortfolioInsight, PortfolioInterpretation,
    PortfolioMember, PortfolioSummary,
};

const HIGH_CORRELATION: f64 = 0.85;
const ELEVATED_CORRELATION: f64 = 0.70;
const HIGH_OVERLAP_PCT: f64 = 15.0;
const ELEVATED_OVERLAP_PCT: f64 = 8.0;
const CONCENTRATED_WEIGHT: f64 = 0.40;
const DRAWDOWN_CAUTION: f64 = 0.20;
const SHARPE_GOOD: f64 = 1.0;
const SHARPE_WEAK: f64 = 0.5;

/// 根据组合报告生成规则化解读文本。
pub fn interpret_portfolio(
    summary: &PortfolioSummary,
    correlation: &CorrelationMatrix,
    overlaps: &[OverlapPair],
) -> PortfolioInterpretation {
    let mut insights = Vec::new();
    insights.extend(assess_overall_risk(summary));
    insights.extend(assess_concentration(&summary.members));
    insights.extend(assess_contributions(&summary.members));
    insights.extend(assess_correlations(correlation));
    insights.extend(assess_overlaps(overlaps));
    insights.extend(assess_data_quality(summary));

    let headline = build_headline(summary, &insights);
    PortfolioInterpretation { headline, insights }
}

fn build_headline(summary: &PortfolioSummary, insights: &[PortfolioInsight]) -> String {
    let cautions = insights
        .iter()
        .filter(|i| i.level == InsightLevel::Caution)
        .count();
    let positives = insights
        .iter()
        .filter(|i| i.level == InsightLevel::Positive)
        .count();

    if cautions >= 3 {
        format!(
            "组合「{}」在 {} 天窗口内风险与分散度需重点关注（{} 条警示）",
            summary.name, summary.period_days, cautions
        )
    } else if positives >= 2 && cautions == 0 {
        format!(
            "组合「{}」在 {} 天窗口内风险收益与分散度整体尚可",
            summary.name, summary.period_days
        )
    } else {
        format!(
            "组合「{}」在 {} 天窗口内表现中性，请结合下方分项解读审慎判断",
            summary.name, summary.period_days
        )
    }
}

fn assess_overall_risk(summary: &PortfolioSummary) -> Vec<PortfolioInsight> {
    let mut out = Vec::new();
    if summary.max_drawdown >= DRAWDOWN_CAUTION {
        out.push(insight(
            InsightLevel::Caution,
            "risk",
            format!(
                "最大回撤 {:.1}%，波动较大；请确认是否在可承受范围内",
                summary.max_drawdown * 100.0
            ),
        ));
    } else if summary.max_drawdown > 0.0 {
        out.push(insight(
            InsightLevel::Info,
            "risk",
            format!(
                "最大回撤 {:.1}%，处于相对温和区间",
                summary.max_drawdown * 100.0
            ),
        ));
    }

    if summary.sharpe_ratio >= SHARPE_GOOD {
        out.push(insight(
            InsightLevel::Positive,
            "return",
            format!("夏普比率 {:.2}，风险调整后收益较好", summary.sharpe_ratio),
        ));
    } else if summary.sharpe_ratio < SHARPE_WEAK && summary.sharpe_ratio.is_finite() {
        out.push(insight(
            InsightLevel::Caution,
            "return",
            format!(
                "夏普比率 {:.2} 偏低，承担波动带来的补偿不足",
                summary.sharpe_ratio
            ),
        ));
    }

    out.push(insight(
        InsightLevel::Info,
        "return",
        format!(
            "区间总收益 {:.1}%，年化 {:.1}%",
            summary.total_return * 100.0,
            summary.annualized_return * 100.0
        ),
    ));
    out
}

fn assess_concentration(members: &[PortfolioMember]) -> Vec<PortfolioInsight> {
    let mut out = Vec::new();
    for m in members {
        if m.weight >= CONCENTRATED_WEIGHT {
            out.push(insight(
                InsightLevel::Caution,
                "concentration",
                format!(
                    "{}（{}）权重 {:.0}%，单只占比偏高，组合表现受其主导",
                    m.name,
                    m.code,
                    m.weight * 100.0
                ),
            ));
        }
    }
    if out.is_empty() && members.len() >= 2 {
        let max_w = members.iter().map(|m| m.weight).fold(0.0_f64, f64::max);
        out.push(insight(
            InsightLevel::Info,
            "concentration",
            format!("最高单只权重 {:.0}%，集中度适中", max_w * 100.0),
        ));
    }
    out
}

fn assess_contributions(members: &[PortfolioMember]) -> Vec<PortfolioInsight> {
    let mut out = Vec::new();
    if members.is_empty() {
        return out;
    }
    let mut sorted = members.to_vec();
    sorted.sort_by(|a, b| {
        b.return_contribution
            .partial_cmp(&a.return_contribution)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    if let Some(top) = sorted.first() {
        out.push(insight(
            InsightLevel::Info,
            "contribution",
            format!(
                "收益贡献最高：{}（{}），约 {:.2} 个百分点",
                top.name,
                top.code,
                top.return_contribution * 100.0
            ),
        ));
    }
    let draggers: Vec<_> = members
        .iter()
        .filter(|m| m.return_contribution < 0.0)
        .collect();
    if !draggers.is_empty() {
        let names: Vec<String> = draggers
            .iter()
            .map(|m| format!("{}({})", m.name, m.code))
            .collect();
        out.push(insight(
            InsightLevel::Caution,
            "contribution",
            format!("以下成分拖累组合收益：{}", names.join("、")),
        ));
    }
    out
}

fn assess_correlations(matrix: &CorrelationMatrix) -> Vec<PortfolioInsight> {
    let mut out = Vec::new();
    let n = matrix.labels.len();
    let mut high_pairs = Vec::new();
    let mut elevated = 0usize;

    for i in 0..n {
        for j in (i + 1)..n {
            let c = matrix.values[i][j];
            if c >= HIGH_CORRELATION {
                high_pairs.push(format!(
                    "{}↔{}({:.2})",
                    matrix.labels[i], matrix.labels[j], c
                ));
            } else if c >= ELEVATED_CORRELATION {
                elevated += 1;
            }
        }
    }

    if !high_pairs.is_empty() {
        out.push(insight(
            InsightLevel::Caution,
            "diversification",
            format!(
                "以下基金日收益高度相关（≥{:.2}），分散效果有限：{}",
                HIGH_CORRELATION,
                high_pairs.join("；")
            ),
        ));
    } else if elevated > 0 {
        out.push(insight(
            InsightLevel::Info,
            "diversification",
            format!(
                "有 {} 对基金中等相关（{:.2}～{:.2}），属常见水平",
                elevated, ELEVATED_CORRELATION, HIGH_CORRELATION
            ),
        ));
    } else if n >= 2 {
        out.push(insight(
            InsightLevel::Positive,
            "diversification",
            "成分基金日收益相关性整体较低，分散度较好".into(),
        ));
    }
    out
}

fn assess_overlaps(overlaps: &[OverlapPair]) -> Vec<PortfolioInsight> {
    let mut out = Vec::new();
    if overlaps.is_empty() {
        out.push(insight(
            InsightLevel::Info,
            "overlap",
            "未计算重仓重叠（离线模式或未拉取持仓）".into(),
        ));
        return out;
    }

    let mut high = Vec::new();
    for p in overlaps {
        if p.overlap_pct >= HIGH_OVERLAP_PCT {
            high.push(format!(
                "{}↔{}({:.1}%)",
                p.fund_a_code, p.fund_b_code, p.overlap_pct
            ));
        } else if p.overlap_pct >= ELEVATED_OVERLAP_PCT {
            out.push(insight(
                InsightLevel::Info,
                "overlap",
                format!(
                    "{} 与 {} 重仓重叠 {:.1}%（{} 只共同持仓），存在一定同质性",
                    p.fund_a_code, p.fund_b_code, p.overlap_pct, p.shared_count
                ),
            ));
        }
    }

    if !high.is_empty() {
        out.push(insight(
            InsightLevel::Caution,
            "overlap",
            format!(
                "以下基金对前十大重仓高度重叠（≥{:.0}%）：{}",
                HIGH_OVERLAP_PCT,
                high.join("；")
            ),
        ));
    } else {
        out.push(insight(
            InsightLevel::Positive,
            "overlap",
            "前十大重仓加权重叠较低，底层持仓重复度可控".into(),
        ));
    }
    out
}

fn assess_data_quality(summary: &PortfolioSummary) -> Vec<PortfolioInsight> {
    let mut out = Vec::new();
    if summary.period_days > 0 {
        let ratio = summary.aligned_days as f64 / summary.period_days as f64;
        if ratio < 0.5 {
            out.push(insight(
                InsightLevel::Caution,
                "data",
                format!(
                    "各成分净值日期交集仅 {} 天（窗口 {} 天），样本偏少，结论需谨慎",
                    summary.aligned_days, summary.period_days
                ),
            ));
        }
    }
    out
}

fn insight(level: InsightLevel, category: &str, message: String) -> PortfolioInsight {
    PortfolioInsight {
        level,
        category: category.to_string(),
        message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CorrelationMatrix, PortfolioSummary};

    fn sample_summary() -> PortfolioSummary {
        PortfolioSummary {
            name: "test".into(),
            period_days: 90,
            aligned_days: 60,
            total_return: 0.08,
            annualized_return: 0.12,
            volatility: 0.15,
            max_drawdown: 0.10,
            sharpe_ratio: 1.2,
            members: vec![
                crate::models::PortfolioMember {
                    code: "000001".into(),
                    name: "A".into(),
                    weight: 0.6,
                    total_return: 0.10,
                    volatility: 0.12,
                    max_drawdown: 0.08,
                    sharpe_ratio: 1.0,
                    return_contribution: 0.06,
                },
                crate::models::PortfolioMember {
                    code: "110011".into(),
                    name: "B".into(),
                    weight: 0.4,
                    total_return: -0.02,
                    volatility: 0.14,
                    max_drawdown: 0.12,
                    sharpe_ratio: 0.3,
                    return_contribution: -0.008,
                },
            ],
        }
    }

    #[test]
    fn interpret_flags_concentration_and_draggers() {
        let corr = CorrelationMatrix {
            labels: vec!["000001".into(), "110011".into()],
            values: vec![vec![1.0, 0.5], vec![0.5, 1.0]],
        };
        let interp = interpret_portfolio(&sample_summary(), &corr, &[]);
        assert!(interp
            .insights
            .iter()
            .any(|i| i.category == "concentration"));
        assert!(interp
            .insights
            .iter()
            .any(|i| i.category == "contribution" && i.level == InsightLevel::Caution));
    }

    #[test]
    fn interpret_high_correlation_caution() {
        let summary = sample_summary();
        let corr = CorrelationMatrix {
            labels: vec!["000001".into(), "110011".into()],
            values: vec![vec![1.0, 0.9], vec![0.9, 1.0]],
        };
        let interp = interpret_portfolio(&summary, &corr, &[]);
        assert!(interp
            .insights
            .iter()
            .any(|i| i.category == "diversification" && i.level == InsightLevel::Caution));
    }
}
