# Architecture Overview

## Project Identity

**Fanalyzer** — Rust 基金分析 CLI 工具（可选 Web UI），面向个人投资者，提供基金数据获取、分析与报告生成。

## Tech Stack

- Language: Rust 2021 Edition (MSRV 1.85)
- Async Runtime: Tokio
- HTTP Client: Reqwest
- CLI: Clap
- Web (optional feature `web`): Leptos SSR + Axum + tower-http
- Serialization: Serde + Serde JSON + TOML
- Logging: tracing + tracing-subscriber
- Error Handling: anyhow (app) + thiserror (library)
- DateTime: chrono

## Layer Diagram

```
┌─────────────────────────────────────────┐
│  cli/          Clap 定义 + dispatch      │  CLI 入口（无业务逻辑）
├─────────────────────────────────────────┤
│  web/          Axum 路由 + Leptos SSR     │  Web 入口（feature `web`）
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
- `web` → `application` → `domain`（Web 不得依赖 `cli`）
- `application` → `presentation`（CLI 终端输出）
- `application` → `api` / `cache` / `nav_cache`（通过 `Session`）
- `api` → `models`
- **禁止反向依赖**：`domain` / `models` 不依赖 `api`、`cli`、`web`

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
├── web/                    # Leptos SSR + Axum（feature `web`）
│   ├── mod.rs              # 启动 wiring
│   ├── routes.rs           # HTTP 路由
│   ├── components.rs       # Leptos 页面组件
│   ├── services.rs         # 薄适配层 → application
│   └── state.rs            # AppState / CommandContext 构造
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

Web 路由经 `web/services.rs` 构造 `CommandContext`，复用同一套 `application` 用例，由 Leptos 组件渲染 HTML。

## 已实现能力

### CLI 子命令

| 类别 | 子命令 |
|------|--------|
| 数据 | `fetch`, `export` |
| 分析 | `analyze`, `compare`, `portfolio` |
| 查询 | `info`, `rank`, `sectors`, `holdings` |
| 工作流 | `brief`, `screen` |
| Web | `serve`（需 `--features web`） |

### Web 页面（feature `web`）

| 页面 | 对应 CLI 能力 |
|------|---------------|
| `/` | 首页导航 |
| `/analyze` | `analyze` |
| `/compare` | `compare` |
| `/info` | `info` |
| `/brief` | `brief` |
| `/portfolio` | `portfolio` |
| `/disclaimer` | 免责声明 |

尚未覆盖：`screen`、`rank`、`export`、`fetch`（后续按需补齐）。

Web 组合页在表单中编辑自选组合（每行「代码 权重」）；首次打开预填 `portfolio.toml` 或自选等权。CLI 仍使用 `--portfolio-file`。

分析口径：优先 `acc_nav`；Alpha/Beta 按 F10 契约基准推断指数；支持 `--period` 与 Sortino/Calmar。

## Testing Strategy

- **domain/**：单元测试（analyzer、period、sort、screen_filter）
- **application/compare.rs**：排序逻辑单元测试
- **api/eastmoney_error.rs**：`into_anyhow` 测试
- **presentation/comparison.rs**：导出 CSV 头测试
- **tests/**：CLI `--help` 集成测试（含 `serve --help`，feature `web`）
- CI 默认构建 + `--features web` 分别跑 clippy / test
- 外部 HTTP：后续可通过 `Session` trait 化 + mock 扩展

## Evolution Roadmap

1. **存储统一** — `storage/` 模块合并 name_cache + nav_cache，可选 TTL
2. **API trait** — `FundDataSource` async trait，便于 mock 与第二数据源
3. **配置化筛选** — `screen` 规则 TOML 模板
4. **组合分析** — 相关性、重仓重叠（v0.2 ✅ portfolio 子命令）
5. **Web 功能追平** — screen、rank、导出等页面
6. **滚动指标与图表** — 滚动 Sharpe/Beta、净值/回撤曲线（v0.3）

## Open Decisions

- [ ] SQLite vs 纯文件缓存（自选/筛选结果持久化）
- [ ] 终端 charting 库
- [ ] 配置热更新
