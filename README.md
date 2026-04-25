# analysis_fund

基于 Rust 的基金数据分析工具，面向个人投资者，提供基金数据获取、净值分析、收益计算与可视化功能。

## 功能

- 基金数据获取（净值、持仓、分红等）
- 收益分析（总收益、年化收益、最大回撤、波动率）
- CLI 命令行交互
- 结构化日志与可配置的数据源

## 文档索引

| 文档 | 说明 |
|------|------|
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

## 环境变量

复制 `.env.example` 为 `.env` 并填入实际值：

```bash
cp .env.example .env
```

## 许可证

MIT License — 详见 [LICENSE](LICENSE)。
