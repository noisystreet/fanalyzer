//! 排行 `--kind` → 天天基金 `ft` 映射。

pub fn rank_ft_code(kind: &str) -> anyhow::Result<&'static str> {
    match kind.trim().to_ascii_lowercase().as_str() {
        "gp" | "股票" | "股票型" => Ok("gp"),
        "hh" | "混合" | "混合型" => Ok("hh"),
        "zq" | "债券" | "债券型" => Ok("zq"),
        "zs" | "指数" | "指数型" => Ok("zs"),
        "qdii" => Ok("qdii"),
        "fof" | "fof型" => Ok("fof"),
        _ => anyhow::bail!("`--kind` 须为 gp/hh/zq/zs/qdii/fof 或中文别名（股票/混合/债券/指数）"),
    }
}

#[cfg(test)]
mod tests {
    use super::rank_ft_code;

    #[test]
    fn rank_ft_accepts_codes_and_aliases() {
        assert_eq!(rank_ft_code("gp").unwrap(), "gp");
        assert_eq!(rank_ft_code("混合").unwrap(), "hh");
    }

    #[test]
    fn rank_ft_rejects_unknown() {
        assert!(rank_ft_code("xyz").is_err());
    }
}
