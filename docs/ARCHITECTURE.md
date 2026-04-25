# Architecture Overview

## Project Identity

**analysis_fund** — Rust 基金分析 CLI 工具，面向个人投资者，提供基金数据获取、分析与报告生成。

## Tech Stack

- Language: Rust 2021 Edition (MSRV 1.85)
- Async Runtime: Tokio
- HTTP Client: Reqwest
- CLI: Clap
- Serialization: Serde + Serde JSON + TOML
- Logging: tracing + tracing-subscriber
- Error Handling: anyhow (app) + thiserror (library)
- DateTime: chrono

## Goals

- 提供准确可靠的基金数据分析
- CLI 优先，易于脚本化与自动化
- 模块化架构，便于扩展新数据源与分析算法
- 可复现的构建与测试

## Non-Goals

- 不做实时交易系统
- 不做 Web UI（后期可独立仓库）
- 不做移动端

## Layer Diagram

```
┌─────────────────────────────────────────┐
│              CLI (clap)                  │  入口层
├─────────────────────────────────────────┤
│          Application Services            │  业务逻辑层
│  (fund analysis, report generation)      │
├────────────┬────────────────────────────┤
│   Models   │         Config             │  领域层
│ (Fund,     │  (AppConfig, LogConfig)    │
│  FundNav,  │                            │
│  Analysis) │                            │
├────────────┴────────────────────────────┤
│           API / Infra Adapter            │  基础设施层
│  (HTTP client, data source adapters)     │
└─────────────────────────────────────────┘
```

## Dependency Direction

- CLI → Application Services → Models ← Config
- API/Infra → Models
- **禁止反向依赖**：Models 不依赖 API 层，Config 不依赖 CLI 层

## Directory Layout

```
analysis_fund/
├── src/
│   ├── main.rs           # CLI 入口
│   ├── api/              # HTTP 客户端与数据源适配
│   │   └── mod.rs
│   ├── config/           # 配置管理
│   │   └── mod.rs
│   └── models/           # 领域模型
│       └── mod.rs
├── tests/                # 集成测试
├── config/               # 配置文件
│   └── default.toml
├── docs/                 # 文档
├── scripts/              # 辅助脚本
├── .github/              # CI / 模板
├── Cargo.toml
└── ...
```

## Evolution Roadmap

1. **v0.1** — CLI 骨架 + 基础数据获取
2. **v0.2** — 净值分析（收益率、波动率、最大回撤）
3. **v0.3** — 多数据源支持（天天基金、东方财富等）
4. **v0.4** — 报告生成（Markdown / HTML）
5. **v0.5** — 投资组合分析

## Open Decisions

- [ ] 数据存储方案：SQLite vs 文件缓存 vs 纯内存
- [ ] 是否引入 charting 库做终端图表
- [ ] 配置热更新 vs 仅启动时加载
