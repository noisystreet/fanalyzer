//! 基金代码：支持位置参数 `CODE` 或 `-c` / `--code`。

use clap::Args;

#[derive(Args, Debug, Clone, Default)]
pub struct FundCodeArg {
    /// 位置参数：基金代码或名称
    #[arg(value_name = "CODE")]
    pub positional: Option<String>,
    /// 基金代码或名称
    #[arg(short = 'c', long = "code", value_name = "CODE")]
    pub flag: Option<String>,
}

impl FundCodeArg {
    /// 合并位置参数与 `--code`；两者均指定且不一致时返回错误。
    pub fn resolve(&self) -> anyhow::Result<Option<String>> {
        match (&self.positional, &self.flag) {
            (Some(p), Some(f)) if p != f => {
                anyhow::bail!("位置参数 `{p}` 与 --code `{f}` 不一致，请只指定一种方式")
            }
            (Some(p), _) => Ok(Some(p.clone())),
            (None, Some(f)) => Ok(Some(f.clone())),
            (None, None) => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_prefers_positional_when_only_one() {
        let arg = FundCodeArg {
            positional: Some("110011".into()),
            flag: None,
        };
        assert_eq!(arg.resolve().unwrap(), Some("110011".into()));
    }

    #[test]
    fn resolve_uses_flag_when_positional_missing() {
        let arg = FundCodeArg {
            positional: None,
            flag: Some("110011".into()),
        };
        assert_eq!(arg.resolve().unwrap(), Some("110011".into()));
    }

    #[test]
    fn resolve_errors_on_mismatch() {
        let arg = FundCodeArg {
            positional: Some("110011".into()),
            flag: Some("000001".into()),
        };
        assert!(arg.resolve().is_err());
    }
}
