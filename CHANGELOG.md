# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- 基金概况 `info` 命令：展示基金全称、成立日期、资产规模、投资目标、投资范围、业绩比较基准等详细信息
- `eastmoney_types` 模块：提取 IndexData、FundManagerInfo、FundFeeInfo、FundProfile 等 5 个数据模型
- `eastmoney_helpers` 模块：提取 JS 变量提取、工作时间解析等辅助函数，含 6 个单元测试
- `screen_filter` 模块：新增 6 个筛选器边界测试
- `returns` 模块：新增 5 个测试（负相关、相关矩阵、nav_price、不等长对齐）
- Agent 结构化 CLI：`fanalyzer json` 子命令、schema export、`--profile summary/standard/full`
- MCP stdio 服务器：`fanalyzer mcp serve`（复合工具 `research_fund`、`compare_watchlist` 等）
- Agent 专用 schema：`schemas/tools.v1.agent.json`、`.embedded.json`
- 全局 `--config` / `FANALYZER_CONFIG`；可执行文件相对路径自动查找 `config/default.toml`
- 信封 `meta.duration_ms` 字段
- `FundDataSource` trait + `MockFundDataSource`（单元/集成测试，CI 无需联网）

### Fixed
- MCP `tools/call` 返回 `NO_OUTPUT`（JSON 捕获上下文不一致）
- MCP 声明 `outputSchema` 但未返回 `structuredContent`，导致客户端校验失败；现与 `content[0].text` 对齐，且 `ok: false` 时统一设置 `isError`
- MCP / 捕获路径失败信封统一为 `envelope.failure.json`（含 `warnings`、`error.hint` / `retryable`）

### Changed
- 项目重命名为 **Fanalyzer**（crate / CLI 二进制：`fanalyzer`）；本地缓存目录改为 `fanalyzer`
- `docs/AGENT.md` / README MCP 章节与 Trae、Cursor 示例对齐
- 重构 `eastmoney.rs`：拆分为 `eastmoney_types` 和 `eastmoney_helpers`，文件体积减少约 100 行
- MCP 失败码补充 `INVALID_ARGS` / `UNKNOWN_TOOL` / `INVALID_OUTPUT` 等与文档对齐

### Added (initial)
- Initial project scaffold
- CLI with `fetch` and `analyze` subcommands
- Fund data models (Fund, FundNav, FundAnalysis)
- API client skeleton
- Configuration management via TOML
- Structured logging with tracing
