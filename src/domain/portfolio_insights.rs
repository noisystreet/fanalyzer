//! 组合分析结果规则解读（纯函数，无 IO）。

use crate::insight_config::PortfolioInsightThresholds;
use crate::models::{
    CorrelationMatrix, InsightLevel, OverlapPair, PortfolioInsight, PortfolioInterpretation,
    PortfolioMember, PortfolioSummary,
};

/// 等权组合指标快照（与当前权重配置对比）。
#[derive(Debug, Clone, Copy)]
pub struct EqualWeightComparison {
    pub total_return: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
}

/// 根据组合报告生成规则化解读文本。
pub fn interpret_portfolio(
    summary: &PortfolioSummary,
    correlation: &CorrelationMatrix,
    overlaps: &[OverlapPair],
    thresholds: &PortfolioInsightThresholds,
    equal_weight: Option<EqualWeightComparison>,
) -> PortfolioInterpretation {
    let mut insights = Vec::new();
    insights.extend(assess_overall_risk(summary, thresholds));
    insights.extend(assess_concentration(&summary.members, thresholds));
    insights.extend(assess_contributions(&summary.members));
    insights.extend(assess_correlations(correlation, thresholds));
    insights.extend(assess_overlaps(overlaps, thresholds));
    insights.extend(assess_data_quality(summary, thresholds));
    if let Some(eq) = equal_weight {
        insights.extend(assess_equal_weight(summary, eq, thresholds));
    }

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

fn assess_overall_risk(
    summary: &PortfolioSummary,
    t: &PortfolioInsightThresholds,
) -> Vec<PortfolioInsight> {
    let mut out = Vec::new();
    if summary.max_drawdown >= t.drawdown_caution {
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

    if summary.sharpe_ratio >= t.sharpe_good {
        out.push(insight(
            InsightLevel::Positive,
            "return",
            format!("夏普比率 {:.2}，风险调整后收益较好", summary.sharpe_ratio),
        ));
    } else if summary.sharpe_ratio < t.sharpe_weak && summary.sharpe_ratio.is_finite() {
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

fn assess_concentration(
    members: &[PortfolioMember],
    t: &PortfolioInsightThresholds,
) -> Vec<PortfolioInsight> {
    let mut out = Vec::new();
    for m in members {
        if m.weight >= t.concentrated_weight {
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

fn assess_correlations(
    matrix: &CorrelationMatrix,
    t: &PortfolioInsightThresholds,
) -> Vec<PortfolioInsight> {
    let mut out = Vec::new();
    let n = matrix.labels.len();
    let mut high_pairs = Vec::new();
    let mut elevated = 0usize;

    for i in 0..n {
        for j in (i + 1)..n {
            let c = matrix.values[i][j];
            if c >= t.high_correlation {
                high_pairs.push(format!(
                    "{}↔{}({:.2})",
                    matrix.labels[i], matrix.labels[j], c
                ));
            } else if c >= t.elevated_correlation {
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
                t.high_correlation,
                high_pairs.join("；")
            ),
        ));
    } else if elevated > 0 {
        out.push(insight(
            InsightLevel::Info,
            "diversification",
            format!(
                "有 {} 对基金中等相关（{:.2}～{:.2}），属常见水平",
                elevated, t.elevated_correlation, t.high_correlation
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

fn assess_overlaps(
    overlaps: &[OverlapPair],
    t: &PortfolioInsightThresholds,
) -> Vec<PortfolioInsight> {
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
        if p.overlap_pct >= t.high_overlap_pct {
            high.push(format!(
                "{}↔{}({:.1}%)",
                p.fund_a_code, p.fund_b_code, p.overlap_pct
            ));
        } else if p.overlap_pct >= t.elevated_overlap_pct {
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
                t.high_overlap_pct,
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

fn assess_data_quality(
    summary: &PortfolioSummary,
    t: &PortfolioInsightThresholds,
) -> Vec<PortfolioInsight> {
    let mut out = Vec::new();
    if summary.period_days > 0 {
        let ratio = summary.aligned_days as f64 / summary.period_days as f64;
        if ratio < t.aligned_days_ratio_caution {
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

fn assess_equal_weight(
    current: &PortfolioSummary,
    equal: EqualWeightComparison,
    t: &PortfolioInsightThresholds,
) -> Vec<PortfolioInsight> {
    let mut out = Vec::new();
    let sharpe_delta = current.sharpe_ratio - equal.sharpe_ratio;
    let return_delta = current.total_return - equal.total_return;

    if sharpe_delta.abs() < t.equal_weight_sharpe_delta {
        out.push(insight(
            InsightLevel::Info,
            "equal_weight",
            format!(
                "当前权重与等权组合夏普接近（{:.2} vs {:.2}），倾斜配置未显著改变风险收益特征",
                current.sharpe_ratio, equal.sharpe_ratio
            ),
        ));
        return out;
    }

    if sharpe_delta > t.equal_weight_sharpe_delta {
        out.push(insight(
            InsightLevel::Positive,
            "equal_weight",
            format!(
                "当前权重夏普 {:.2} 高于等权 {:.2}（总收益差 {:.1} 个百分点），倾斜配置带来更好风险补偿",
                current.sharpe_ratio,
                equal.sharpe_ratio,
                return_delta * 100.0
            ),
        ));
    } else {
        out.push(insight(
            InsightLevel::Caution,
            "equal_weight",
            format!(
                "等权夏普 {:.2} 高于当前 {:.2}（总收益差 {:.1} 个百分点），可考虑是否过度集中",
                equal.sharpe_ratio,
                current.sharpe_ratio,
                -return_delta * 100.0
            ),
        ));
    }

    if equal.max_drawdown < current.max_drawdown - 0.02 {
        out.push(insight(
            InsightLevel::Info,
            "equal_weight",
            format!(
                "等权最大回撤 {:.1}% 低于当前 {:.1}%",
                equal.max_drawdown * 100.0,
                current.max_drawdown * 100.0
            ),
        ));
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
    use crate::insight_config::PortfolioInsightThresholds;
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

    fn default_thresholds() -> PortfolioInsightThresholds {
        PortfolioInsightThresholds::default()
    }

    #[test]
    fn interpret_flags_concentration_and_draggers() {
        let corr = CorrelationMatrix {
            labels: vec!["000001".into(), "110011".into()],
            values: vec![vec![1.0, 0.5], vec![0.5, 1.0]],
        };
        let interp =
            interpret_portfolio(&sample_summary(), &corr, &[], &default_thresholds(), None);
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
        let interp = interpret_portfolio(&summary, &corr, &[], &default_thresholds(), None);
        assert!(interp
            .insights
            .iter()
            .any(|i| i.category == "diversification" && i.level == InsightLevel::Caution));
    }

    #[test]
    fn interpret_equal_weight_worse_than_current() {
        let summary = sample_summary();
        let corr = CorrelationMatrix {
            labels: vec!["000001".into(), "110011".into()],
            values: vec![vec![1.0, 0.5], vec![0.5, 1.0]],
        };
        let eq = EqualWeightComparison {
            total_return: 0.05,
            sharpe_ratio: 0.8,
            max_drawdown: 0.12,
        };
        let interp = interpret_portfolio(&summary, &corr, &[], &default_thresholds(), Some(eq));
        assert!(interp
            .insights
            .iter()
            .any(|i| i.category == "equal_weight" && i.level == InsightLevel::Positive));
    }

    #[test]
    fn interpret_equal_weight_better_suggests_caution() {
        let summary = PortfolioSummary {
            sharpe_ratio: 0.6,
            total_return: 0.04,
            ..sample_summary()
        };
        let corr = CorrelationMatrix {
            labels: vec!["000001".into(), "110011".into()],
            values: vec![vec![1.0, 0.5], vec![0.5, 1.0]],
        };
        let eq = EqualWeightComparison {
            total_return: 0.08,
            sharpe_ratio: 1.1,
            max_drawdown: 0.08,
        };
        let interp = interpret_portfolio(&summary, &corr, &[], &default_thresholds(), Some(eq));
        assert!(interp
            .insights
            .iter()
            .any(|i| i.category == "equal_weight" && i.level == InsightLevel::Caution));
    }
}
