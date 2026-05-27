# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Agent 结构化 CLI：`fanalyzer json` 子命令、schema export、`--profile summary/standard/full`
- MCP stdio 服务器：`fanalyzer mcp serve`（复合工具 `research_fund`、`compare_watchlist` 等）
- Agent 专用 schema：`schemas/tools.v1.agent.json`、`.embedded.json`
- 全局 `--config` / `FANALYZER_CONFIG`；可执行文件相对路径自动查找 `config/default.toml`
- 信封 `meta.duration_ms` 字段
- `FundDataSource` trait + `MockFundDataSource`（单元/集成测试，CI 无需联网）

### Fixed
- MCP `tools/call` 返回 `NO_OUTPUT`（JSON 捕获上下文不一致）

### Changed
- 项目重命名为 **Fanalyzer**（crate / CLI 二进制：`fanalyzer`）；本地缓存目录改为 `fanalyzer`
- `docs/AGENT.md` / README MCP 章节与 Trae、Cursor 示例对齐

### Added (initial)
- Initial project scaffold
- CLI with `fetch` and `analyze` subcommands
- Fund data models (Fund, FundNav, FundAnalysis)
- API client skeleton
- Configuration management via TOML
- Structured logging with tracing
