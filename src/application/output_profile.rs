//! 结构化输出粒度（Agent token 优化）。

/// JSON 输出 profile：`summary` 最省 token，`full` 含时间序列。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputProfile {
    /// 仅标量快照，省略 series 与重仓明细。
    Summary,
    /// 默认：省略 series，保留业务字段。
    #[default]
    Standard,
    /// 完整数据（含 series）。
    Full,
}

impl OutputProfile {
    pub fn parse(s: &str) -> anyhow::Result<Self> {
        match s {
            "summary" => Ok(Self::Summary),
            "standard" => Ok(Self::Standard),
            "full" => Ok(Self::Full),
            other => anyhow::bail!("无效 profile：{other}，可选 summary/standard/full"),
        }
    }

    pub fn compact_series(self) -> bool {
        !matches!(self, Self::Full)
    }

    pub fn json_compact(self) -> bool {
        matches!(self, Self::Summary | Self::Standard)
    }

    pub fn summary_mode(self) -> bool {
        matches!(self, Self::Summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_parse_and_flags() {
        assert!(OutputProfile::Summary.compact_series());
        assert!(OutputProfile::Summary.summary_mode());
        assert!(!OutputProfile::Full.compact_series());
        assert!(OutputProfile::parse("standard").is_ok());
        assert!(OutputProfile::parse("invalid").is_err());
    }
}
