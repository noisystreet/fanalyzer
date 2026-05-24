# analysis_fund

基于 Rust 的基金数据分析工具，面向个人投资者，提供基金数据获取、净值分析、收益计算与可视化功能。

## 功能

- 基金数据获取（净值、排行、行业配置、重仓股等）
- 收益分析（总收益、年化收益、最大回撤、波动率）
- CLI 命令行交互
- 结构化日志与可配置的数据源

## 文档索引

| 文档 | 说明 |
|------|------|
| [docs/MANUAL.md](docs/MANUAL.md) | **CLI 使用手册**（子命令、自选、离线、排行等） |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | 架构总览 |
| [docs/DATA_MODEL.md](docs/DATA_MODEL.md) | 数据模型 |
| [AGENTS.md](AGENTS.md) | AI Agent 入口文档 |
| [CONTRIBUTING.md](CONTRIBUTING.md) | 贡献指南 |
| [SECURITY.md](SECURITY.md) | 安全政策 |
| [CHANGELOG.md](CHANGELOG.md) | 变更日志 |

## 快速开始

```bash
# 构建
cargo build

# 运行
cargo run -- fetch --code 000001
cargo run -- analyze --code 000001 --days 90

# 测试
cargo test

# 代码检查
cargo fmt -- --check && cargo clippy -- -D warnings
```

## 使用指南

更完整的参数说明、全局选项与注意事项见 **[docs/MANUAL.md](docs/MANUAL.md)**。

### 全局与自选（摘要）

- **`--offline`**：仅用本地净值缓存；不能与 `fetch`、`info`、`rank`、`brief`、`screen` 等需联网子命令同时使用。
- **`--watchlist-file`**：自选列表路径，默认 `config/watchlist.toml`；配合各子命令的 **`--watchlist`** 批量处理。
- 自选文件格式：TOML 中 `funds = ["000001", "基金名称"]`。

### 1. 获取基金净值数据

```bash
# 获取最近 30 条净值记录
cargo run -- fetch --code 000001

# 获取指定数量的记录
cargo run -- fetch --code 000001 --limit 100
```

### 2. 基金分析

分析基金的多维度指标，包括收益、风险、经理信息等：

```bash
# 分析最近 90 天的数据（或使用与 rank 对齐的窗口）
cargo run -- analyze --code 000001 --days 90
cargo run -- analyze --code 000001 --period 1y
```

**输出指标说明：**

| 指标 | 说明 | 参考意义 |
|------|------|----------|
| 总收益率 | 分析周期内的累计收益 | 绝对收益表现 |
| 年化收益率 | 收益按年标准化 | 便于不同周期比较 |
| 波动率 | 收益率的标准差（年化） | 风险水平，越低越稳定 |
| 最大回撤 | 峰值到谷底的最大跌幅 | 极端风险，越小越好 |
| 夏普比率 | (收益-无风险利率)/波动率 | 风险调整后收益，>1 较好 |
| 索提诺比率 | 仅计下行波动的风险调整收益 | 比夏普更关注亏损风险 |
| 卡玛比率 | 年化收益/最大回撤 | 收益与极端回撤的平衡 |
| 阿尔法 (Alpha) | 超越契约基准指数的超额收益 | 越高越好 |
| 贝塔 (Beta) | 相对于市场的波动程度 | 系统风险，1 为市场水平 |
| 基金经理 | 当前基金经理姓名 | 稳定性参考 |
| 经理任期 | 经理管理该基金的时间 | 越长经验越丰富 |
| 经理任职回报 | 经理任期内的累计收益 | 经理能力体现 |
| 管理费率 | 每年收取的管理费用比例 | 持有成本，越低越好 |
| 托管费率 | 每年收取的托管费用比例 | 持有成本，越低越好 |

### 3. 基金对比

同时对比多只基金的表现：

```bash
# 对比两只基金，按夏普排序并导出
cargo run -- compare --codes 000001,000003 --period 1y --sort sharpe --output cmp.csv
```

### 4. 数据导出

```bash
# 导出为 CSV
cargo run -- export --code 000001 --days 90 --output fund_data.csv --format csv

# 导出为 JSON
cargo run -- export --code 000001 --days 90 --output fund_data.json --format json
```

### 5. 类型排行（全市场 Top N）

按天天基金官网开放式基金排行拉取某类型前 N 名（需联网）：

```bash
cargo run -- rank --kind gp --top 100
cargo run -- rank --kind 混合 --top 20 --sort 1nzf
```

`--kind` 支持 `gp|hh|zq|zs|qdii|fof` 及中文别名；`--top` 默认 100、上限 500；**`--sort` 即官网 `sc`**（默认 `1n`，与网页完全一致时可试 `1nzf`）。**`rzdf` / `zzf` / `1yzf` / `3nzf` / `jnzf` 等对照表**见 [docs/MANUAL.md](docs/MANUAL.md)。

### 6. 行业配置（板块分析）

按季报披露的 **证监会行业分类** 占比查看基金在行业上的暴露（命令 `sectors`，详见 [docs/MANUAL.md](docs/MANUAL.md)）：

```bash
cargo run -- sectors --code 000001
```

### 7. 重仓股（股票投资明细）

```bash
cargo run -- holdings --code 000001 --top 10
```

详见 [docs/MANUAL.md](docs/MANUAL.md) 中 `holdings` 子命令。

### 8. 选基工作流（`brief` + `screen`）

**`brief`**：单只或自选综合简报（分析 + 行业 + 重仓），可写 Markdown：

```bash
cargo run -- brief --code 000001 --days 90 --output brief.md
cargo run -- brief --watchlist
```

**`screen`**：排行预筛 + deep 分析 + 规则过滤 + 对比/导出：

```bash
cargo run -- screen --kind gp --sort 1nzf --min-rank-return 10 --max-drawdown 25 --sort-by sharpe --output screen.csv
```

参数与示例见 [docs/MANUAL.md](docs/MANUAL.md) 中「选基工作流」章节。

### 9. 查看基金详细信息

获取基金的基本概况、投资目标、投资范围、基金经理等详细信息：

```bash
# 查看基金详细信息
cargo run -- info --code 000001

# 支持中文名称
cargo run -- info --code "华夏成长混合"
```

**输出内容包括：**

| 信息项 | 说明 |
|--------|------|
| 基金全称 | 基金的完整名称 |
| 基金简称 | 基金的简称 |
| 基金代码 | 基金的唯一标识代码 |
| 基金类型 | 如混合型-偏股、债券型等 |
| 成立日期 | 基金成立的时间 |
| 资产规模 | 基金管理的资产规模 |
| 管理公司 | 基金管理公司名称 |
| 业绩比较基准 | 用于衡量基金表现的参考标准 |
| 基金经理 | 当前管理该基金的经理 |
| 经理任期 | 经理管理该基金的时间长度 |
| 经理任职回报 | 经理任期内的累计收益率 |
| 管理费率 | 年度管理费用比例 |
| 托管费率 | 年度托管费用比例 |
| 投资目标 | 基金的投资目标和策略方向 |
| 投资范围 | 基金可投资的资产类别 |

### 10. 使用中文基金名称

支持通过中文名称搜索基金：

```bash
# 使用中文名称进行分析
cargo run -- analyze --code "华夏成长混合" --days 90

# 系统会自动搜索并匹配基金代码
```

### 11. Web 界面（Leptos SSR，可选）

编译时需启用 `web` feature：

```bash
cargo run --features web -- serve
# 浏览器打开 http://127.0.0.1:3000
```

页面：`/` 首页、`/analyze` 单基金分析、`/compare` 多基金对比、`/info` 基金概况、`/brief` 选基简报；与 CLI 共用 application 层与本地缓存。

```bash
cargo run --features web -- serve --host 0.0.0.0 --port 8080
```

## 环境变量

复制 `.env.example` 为 `.env` 并填入实际值：

```bash
cp .env.example .env
```

## 许可证

MIT License — 详见 [LICENSE](LICENSE)。
