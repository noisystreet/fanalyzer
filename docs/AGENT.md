# Agent 集成指南

面向大模型 Agent、MCP 工具与自动化脚本的 **结构化 CLI** 说明。

> 数据边界与免责声明见 [DISCLAIMER.md](DISCLAIMER.md)。输出仅供研究参考，不构成投资建议。

## 快速开始

```bash
# 单基金分析（stdout 仅 JSON；`CODE` 位置参数或 `--code`）
cargo run -- json analyze 110011 --days 90

# 紧凑单行 JSON（管道友好）
cargo run -- json --compact analyze 110011 --days 90

# 省略时间序列曲线（省 token）
cargo run -- json --compact-series analyze 110011 --days 90
```

## 流约定

| 流 | 内容 |
|----|------|
| **stdout** | 成功或失败均为 JSON 信封 |
| **stderr** | `tracing` 日志（`RUST_LOG=warn` 可减少噪音） |
| **退出码** | `0` 成功；非 `0` 失败（失败时 stdout 仍有 JSON） |

信封 `meta` 含 `generated_at`、`duration_ms`（命令耗时毫秒，便于 Agent 追踪）。

## 信封格式

Schema 索引见 [schemas/index.json](../schemas/index.json)（由 `fanalyzer schema export` 自动生成）。

- **Agent 工具入参**：`schemas/tools.v1.agent.json`
- **内联 outputSchema**：`schemas/tools.v1.agent.embedded.json`
- **CLI 完整工具入参**：`schemas/tools.v1.json`

### 成功

```json
{
  "v": 1,
  "command": "analyze",
  "ok": true,
  "meta": {
    "offline": false,
    "generated_at": "2026-05-23T12:00:00+08:00",
    "days": 90,
    "requested": 1,
    "succeeded": 1
  },
  "warnings": [],
  "data": { }
}
```

### 失败

```json
{
  "v": 1,
  "command": "compare",
  "ok": false,
  "meta": {
    "offline": false,
    "generated_at": "2026-05-23T12:00:00+08:00"
  },
  "warnings": [],
  "error": {
    "code": "INSUFFICIENT_SAMPLES",
    "message": "有效样本不足（需要≥2）；请检查离线缓存或数据源"
  }
}
```

### 字段说明

| 字段 | 说明 |
|------|------|
| `v` | 信封版本，当前为 `1` |
| `command` | 业务子命令名（非 `json` 包装层） |
| `ok` | 是否成功 |
| `meta` | 请求上下文（离线、时间戳、窗口天数等，因命令而异） |
| `warnings` | 非致命警告（如部分标的跳过、深度分析截断） |
| `data` | 成功时的 payload（失败时省略） |
| `error` | 失败时的 `{ code, message }`（成功时省略） |

## 错误码

| code | 典型场景 |
|------|----------|
| `INSUFFICIENT_SAMPLES` | 对比/筛选有效样本不足 |
| `INSUFFICIENT_DATA` | 分析/导出无有效数据 |
| `OFFLINE_UNSUPPORTED` | 离线模式下调用需联网命令 |
| `COMMAND_FAILED` | 其他运行时错误 |

## 批量命令与 partial success

多标的命令（`analyze`、`compare`、`fetch` 等）的 `data` 使用：

```json
{
  "items": [ /* 成功条目 */ ],
  "errors": [
    {
      "code": "000001",
      "message": "分析数据不足",
      "error_code": "INSUFFICIENT_DATA"
    }
  ]
}
```

- 至少 1 条 `items` 时 `ok: true`，失败条目记录在 `errors`
- 全部失败时 `ok: false`，见 `error` 字段

## 各命令 `data` 形状

| command | data 类型 |
|---------|-----------|
| `analyze` | `{ items: FundAnalysisReport[], errors? }` |
| `compare` | `{ items: FundAnalysis[], errors? }` |
| `portfolio` | `PortfolioReport` 对象 |
| `screen` | `ScreenPayload`（含 `passed[]`） |
| `rank` | `RankPayload`（含 `rows[]`） |
| `fetch` / `export` | `{ items: FetchPayload/ExportPayload[], errors? }` |
| `info` / `sectors` / `holdings` / `brief` | `{ items: [...], errors? }` |

模型字段详见 [DATA_MODEL.md](DATA_MODEL.md)。

## `json` 子命令

```bash
fanalyzer json [--compact] [--compact-series] <子命令> [参数...]
```

| 参数 | 说明 |
|------|------|
| `json` / `structured` | 进入结构化输出模式 |
| `--compact` | 单行 minified JSON |
| `--compact-series` | 省略 `series` 时间序列 |
| `--profile` | 输出粒度：`summary` / `standard` / `full`（推荐 Agent 用 `summary` 或 `standard`） |
| `--offline` | 全局：仅本地缓存（部分嵌套命令不可用） |

嵌套子命令：`fetch`、`analyze`、`compare`、`portfolio`、`export`、`info`、`sectors`、`holdings`、`rank`、`brief`、`screen`、`watchlist`、`portfolio-config`。

## MCP Server（推荐）

无需 Agent 拼 shell，直接通过 stdio MCP 调用：

```bash
cargo run -- mcp serve --profile standard --tools minimal
```

| 参数 | 说明 |
|------|------|
| `--profile` | 输出粒度：`summary` / `standard` / `full` |
| `--tools` | 暴露工具集：`minimal`（6 个核心）/ `standard` / `full`（默认） |
| `--offline` | 进程级离线模式 |
| `--watchlist-file` / `--portfolio-file` | 自选与组合 TOML 路径 |

**Resources**（`resources/list`、`resources/read`）：`fanalyzer://schemas/index`、`fanalyzer://watchlist`、`fanalyzer://portfolio`、`fanalyzer://config`（配置摘要，不含 proxy）。

完整 MCP 配置（Cursor、Trae、Claude Code、Windsurf 等）见 [README.md](../README.md#mcp-与-agent-集成) 的 **「MCP 与 Agent 集成」** 章节。

通用 JSON 结构：

```json
{
  "mcpServers": {
    "fanalyzer": {
      "command": "/path/to/fanalyzer/target/debug/fanalyzer",
      "args": ["mcp", "serve", "--profile", "summary", "--tools", "minimal"],
      "cwd": "/path/to/fanalyzer",
      "env": { "RUST_LOG": "warn" }
    }
  }
}
```

| 字段 | 说明 |
|------|------|
| `command` | 二进制绝对路径；Trae/Cursor 可用 `${workspaceFolder}/target/debug/fanalyzer` |
| `args` | 推荐 `--profile summary`（最省 token） |
| `cwd` | 可选；未设置时若找不到 `config/default.toml`，会尝试相对可执行文件目录解析（见下 `--config`） |
| `env` | 可选；`RUST_LOG=warn` 减少 stderr |

**配置文件解析**（与 MCP `cwd` 解耦）：

```bash
# 显式指定（或环境变量 FANALYZER_CONFIG）
fanalyzer --config /path/to/config/default.toml mcp serve

# 自动查找顺序：--config → ./config/default.toml → 可执行文件旁 ../config/default.toml
```

MCP 工具列表来自 `schemas/tools.v1.agent.json`（已剥离 `compact` 等内部参数）。复合工具：

| 工具 | 说明 |
|------|------|
| `fanalyzer_research_fund` | info + analyze + sectors + holdings |
| `fanalyzer_compare_watchlist` | 对比自选全部基金 |
| `fanalyzer_watchlist_*` | 自选增删查 |

## Agent 调用建议

1. **始终解析 stdout JSON**（CLI）或 tool result text（MCP），勿依赖 stderr
2. **检查 `warnings`**：部分成功时 Agent 应告知用户哪些标的被跳过
3. **大上下文**用 `--profile summary` 或 MCP `--profile summary`
4. **对比前先 `analyze` 或 `fetch`** 写入缓存，再 `--offline` 复用
5. Tool schema：Agent 入参见 `schemas/tools.v1.agent.json`；内联 outputSchema 见 `tools.v1.agent.embedded.json`
6. 失败时查看 `error.hint` 与 `error.retryable` 决定是否重试

### 错误信封扩展字段

```json
"error": {
  "code": "INSUFFICIENT_DATA",
  "message": "...",
  "retryable": true,
  "hint": "先运行 fetch 或 analyze 写入缓存后再试"
}
```

## Schema 生成与校验

```bash
# 导出/更新 schemas/（无需联网）
cargo run -- schema export --output-dir schemas
# 或
python3 scripts/generate_schemas.py

# CI / 本地校验是否与代码同步
python3 scripts/generate_schemas.py --check
```

## 示例：Shell 中提取 items

```bash
cargo run -- json --compact analyze 110011 --days 90 2>/dev/null \
  | jq -r '.data.items[0].snapshot.sharpe_ratio'
```

## 示例：失败处理

```bash
if ! out=$(cargo run -- json compare --codes 110011 2>/dev/null); then
  code=$(echo "$out" | jq -r '.error.code')
  echo "failed: $code"
fi
```
