//! 净值 fetch 命令终端输出。

use crate::models::FundNav;

pub fn print_fetch_result(code: &str, name: &str, nav_list: &[FundNav], total: u32) {
    println!(
        "Fetched {} records (total: {}) for fund {} ({})",
        nav_list.len(),
        total,
        code,
        name
    );
    for nav in nav_list {
        println!(
            "  {}  NAV: {:.4}  AccNAV: {:.4}  DailyReturn: {}",
            nav.date,
            nav.nav,
            nav.acc_nav,
            nav.daily_return
                .map(|r| format!("{:.2}%", r * 100.0))
                .unwrap_or_else(|| "N/A".to_string())
        );
    }
}
