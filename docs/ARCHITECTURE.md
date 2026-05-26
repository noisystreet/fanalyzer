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
│  cli/          Clap 定义 + dispatch      │  入口层（无业务逻辑）
├─────────────────────────────────────────┤
│  application/  用例编排（Use Case）       │  应用层
├─────────────────────────────────────────┤
│  domain/       纯计算与规则（无 IO）        │  领域层
├─────────────────────────────────────────┤
│  presentation/ 终端表格 / 导出 / Markdown  │  呈现层
├────────────┬────────────────────────────┤
│  models/   │  config/  watchlist/        │  领域模型与配置
├────────────┴────────────────────────────┤
│  api/  cache/  nav_cache/               │  基础设施（HTTP、缓存）
└─────────────────────────────────────────┘
```

## Dependency Direction

- `cli` → `application` → `domain` ← `models`
- `application` → `presentation`（输出）
- `application` → `api` / `cache` / `nav_cache`（通过 `Session`）
- `api` → `models`
- **禁止反向依赖**：`domain` / `models` 不依赖 `api` 或 `cli`

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
├── models/
├── config/
├── cache/                  # 名称↔代码映射缓存
└── nav_cache/              # 净值 JSON 缓存
```

## Application Context

- **`Session`**：`EastMoneyClient` + 名称缓存 + 净值缓存（数据访问门面，`FundRepository` 类型别名）
- **`CommandContext`**：`Session` + `offline` + 自选文件路径

CLI 子命令经 `dispatch` 构造 `CommandContext`，调用 `application::*` 用例，由 `presentation` 渲染。

## 已实现 CLI 能力

| 类别 | 子命令 |
|------|--------|
| 数据 | `fetch`, `export` |
| 分析 | `analyze`, `compare` |
| 查询 | `info`, `rank`, `sectors`, `holdings` |
| 工作流 | `brief`, `screen` |

分析口径：优先 `acc_nav`；Alpha/Beta 按 F10 契约基准推断指数；支持 `--period` 与 Sortino/Calmar。

## Testing Strategy

- **domain/**：单元测试（analyzer、period、sort、screen_filter）
- **presentation/comparison.rs**：导出 CSV 头测试
- **tests/**：CLI `--help` 集成测试
- 外部 HTTP：后续可通过 `Session` trait 化 + mock 扩展

## Evolution Roadmap

1. **存储统一** — `storage/` 模块合并 name_cache + nav_cache，可选 TTL
2. **API trait** — `FundDataSource` async trait，便于 mock 与第二数据源
3. **配置化筛选** — `screen` 规则 TOML 模板
4. **组合分析** — 相关性、重仓重叠（v0.5）

## Open Decisions

- [ ] SQLite vs 纯文件缓存（自选/筛选结果持久化）
- [ ] 终端 charting 库
- [ ] 配置热更新
