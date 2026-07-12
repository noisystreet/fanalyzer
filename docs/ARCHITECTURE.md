# Architecture Overview

## Project Identity

**Fanalyzer** — Rust 基金分析 CLI 工具，面向个人投资者，提供基金数据获取、分析与报告生成。

## Tech Stack

- Language: Rust 2021 Edition (MSRV 1.85)
- Async Runtime: Tokio
- HTTP Client: Reqwest
- CLI: Clap
- Serialization: Serde + Serde JSON + TOML
- Logging: tracing + tracing-subscriber
- Error Handling: anyhow (app) + thiserror (library)
- DateTime: chrono

## Layer Diagram

```
┌─────────────────────────────────────────┐
│  cli/          Clap 定义 + dispatch      │  CLI 入口（无业务逻辑）
├─────────────────────────────────────────┤
│  application/  用例编排（Use Case）       │  应用层
├─────────────────────────────────────────┤
│  domain/       纯计算与规则（无 IO）        │  领域层
├─────────────────────────────────────────┤
│  presentation/ 终端表格 / 导出 / Markdown  │  呈现层（CLI 输出）
├────────────┬────────────────────────────┤
│  models/   │  config/  watchlist.rs      │  领域模型与配置
├────────────┴────────────────────────────┤
│  api/  cache.rs  nav_cache.rs           │  基础设施（HTTP、缓存）
└─────────────────────────────────────────┘
```

## Dependency Direction

- `cli` → `application` → `domain` ← `models`
- `application` → `presentation`（CLI 终端输出）
- `application` → `api` / `cache` / `nav_cache`（通过 `Session`）
- `api` → `models`
- **禁止反向依赖**：`domain` / `models` 不依赖 `api`、`cli`

`services/` 为兼容 re-export，新代码请使用 `domain` / `application`。

## Directory Layout

```
src/
├── main.rs                 # 入口
├── lib.rs
├── cli/                    # Clap + dispatch（薄）
│   ├── mod.rs              # Cli / Commands 定义
│   ├── dispatch.rs
│   ├── dispatch_query.rs
│   └── dispatch_workflow.rs
├── application/            # 用例
│   ├── context.rs          # Session / CommandContext
│   ├── fund_service.rs     # 解析、净值、分析编排
│   ├── mappers.rs          # API DTO → models
│   ├── analyze.rs / compare.rs / screen.rs / brief.rs
│   ├── export.rs / queries.rs
│   └── mod.rs
├── domain/                 # 纯逻辑
│   ├── analyzer.rs         # FundAnalyzer
│   ├── benchmark.rs        # 契约基准 → 指数
│   ├── period.rs / sort.rs / screen_filter.rs / rank_kind.rs
│   └── mod.rs
├── presentation/           # 终端输出
│   ├── mod.rs              # 表格、报告、净值导出
│   └── comparison.rs       # 对比表 + CSV/JSON
├── api/                    # 东方财富 HTTP 与解析
│   ├── eastmoney.rs        # EastMoneyClient
│   ├── eastmoney_error.rs  # EastMoneyError + into_anyhow
│   └── …                   # f10、holdings、industry、ranking 等
├── models/
├── config/
├── cache.rs                # 名称↔代码映射缓存
├── nav_cache.rs            # 净值 JSON 缓存
└── watchlist.rs            # 自选列表读写
```

## Application Context

- **`Session`**：`EastMoneyClient` + 名称缓存 + 净值缓存（数据访问门面，`FundRepository` 类型别名）
- **`CommandContext`**：`Session` + `offline` + 自选文件路径

CLI 子命令经 `dispatch` 构造 `CommandContext`，调用 `application::*` 用例，由 `presentation` 渲染。

## 已实现能力

### CLI 子命令

| 类别 | 子命令 |
|------|--------|
| 数据 | `fetch`, `export` |
| 分析 | `analyze`, `compare`, `portfolio` |
| 查询 | `info`, `rank`, `sectors`, `holdings` |
| 工作流 | `brief`, `screen` |

分析口径：优先 `acc_nav`；Alpha/Beta 按 F10 契约基准推断指数；支持 `--period` 与 Sortino/Calmar。

## Testing Strategy

- **domain/**：单元测试（analyzer、period、sort、screen_filter）
- **application/**：golden 信封测试（`analyze` / `compare` / `portfolio` / `brief`），共享 `test_support` + `MockFundDataSource`
- **api/**：HTML/JS 解析 fixture 位于 `tests/fixtures/api/`，单元测试从文件加载以防回归
- **tests/schema_contract_test.rs**：离线 CLI 输出用 `jsonschema` 校验 `schemas/responses/*.success.json` 与 `envelope.failure.json`
- **tests/mcp_schema_contract_test.rs**：MCP `tools/call` 工具结果校验同一套 response schema（离线 `--config` + 缓存 fixture）
- **tests/integration_test.rs**：CLI/MCP 行为与 `--config` 路径
- CI 默认构建跑 clippy / test
- 外部 HTTP：`FundDataSource` trait + 离线缓存 fixture，核心路径 CI 无需联网

## Evolution Roadmap

1. **存储统一** — `storage/` 模块合并 name_cache + nav_cache，可选 TTL
2. **API trait** — `FundDataSource` async trait（✅ 已实现），便于 mock 与第二数据源
3. **配置化筛选** — `screen` 规则 TOML 模板
4. **组合分析** — 相关性、重仓重叠（v0.2 ✅ portfolio 子命令）

实现要点：

- `domain/rolling.rs`：归一化净值、回撤、60 日滚动夏普/波动/Beta
- `models/series.rs`：`FundAnalysisReport`、`PortfolioTimeSeries`
- CLI：`analyze --output report.json` 导出完整时间序列

## Open Decisions

- [ ] SQLite vs 纯文件缓存（自选/筛选结果持久化）
- [ ] 终端 charting 库
- [ ] 配置热更新
