# Fanalyzer

[![CI](https://github.com/noisystreet/fanalyzer/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/noisystreet/fanalyzer/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/github/license/noisystreet/fanalyzer)](LICENSE)

Fund analysis CLI & Web UI — 个人基金研究与选基工具（Rust）。

仓库：<https://github.com/noisystreet/fanalyzer>

> **免责声明**：本工具数据来自第三方公开渠道，输出仅供个人研究参考，**不构成投资建议**。使用前请阅读 [docs/DISCLAIMER.md](docs/DISCLAIMER.md)。

## 功能

- 基金数据获取（净值、排行、行业配置、重仓股等）
- 收益分析（总收益、年化收益、最大回撤、波动率）
- CLI 命令行交互
- **MCP Server**（Cursor、Trae 等 Agent 直连）
- 结构化 JSON 输出（`json` 子命令，供自动化脚本）
- 结构化日志与可配置的数据源

## 文档索引

| 文档 | 说明 |
|------|------|
| [docs/MANUAL.md](docs/MANUAL.md) | **CLI 使用手册**（子命令、自选、离线、排行等） |
| [docs/AGENT.md](docs/AGENT.md) | **Agent / MCP 集成**（结构化 JSON、Schema、Cursor 配置） |
| [docs/DISCLAIMER.md](docs/DISCLAIMER.md) | **免责声明与数据使用说明**（发布前必读） |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | 架构总览 |
| [docs/DATA_MODEL.md](docs/DATA_MODEL.md) | 数据模型 |
| [AGENTS.md](AGENTS.md) | AI Agent 入口文档 |
| [CONTRIBUTING.md](CONTRIBUTING.md) | 贡献指南 |
| [SECURITY.md](SECURITY.md) | 安全政策 |
| [CHANGELOG.md](CHANGELOG.md) | 变更日志 |

## 快速开始

```bash
git clone https://github.com/noisystreet/fanalyzer.git
cd fanalyzer
```

```bash
# 构建（默认仅 CLI，编译更快）
cargo build

# Web UI 需显式启用 feature
cargo build --features web

# 运行
cargo run -- fetch --code 000001
cargo run -- analyze --code 000001 --days 90

# 测试
cargo test

# 代码检查
cargo fmt -- --check && cargo clippy -- -D warnings
```

日常 CLI / MCP 开发无需 `--features web`；仅 `serve` 或改 Web 页面时再启用。

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

页面：`/` 首页、`/analyze` 单基金分析、`/compare` 多基金对比、`/info` 基金概况、`/brief` 选基简报、`/disclaimer` 免责声明；与 CLI 共用 application 层与本地缓存。

```bash
cargo run --features web -- serve --host 0.0.0.0 --port 8080
```

## Agent 与 MCP 集成

在 Cursor、Trae 等支持 MCP 的客户端中，可通过 **stdio MCP** 直接调用基金分析工具，无需 Agent 拼 shell。完整说明见 **[docs/AGENT.md](docs/AGENT.md)**；工具 Schema 见 `schemas/tools.v1.agent.json`。

### 构建

```bash
cargo build
# 推荐 release：cargo build --release
```

记下二进制绝对路径，例如：

- 开发：`/path/to/fanalyzer/target/debug/fanalyzer`
- 发布：`/path/to/fanalyzer/target/release/fanalyzer`

修改代码或拉取更新后需 **重新 `cargo build`**，并在客户端中 **刷新或重启 MCP**。

### 通用 MCP 配置

各客户端 JSON 结构基本一致（`mcpServers` 键名可能略有差异）。将 `/path/to/fanalyzer` 替换为**本机仓库绝对路径**：

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
| `command` | `cargo build` 产物的**绝对路径**（release 可改为 `target/release/fanalyzer`） |
| `args` | 固定 `mcp serve`；`--profile`、`--tools` 见下表 |
| `cwd` | 建议设为项目根；也可用 `--config` / `FANALYZER_CONFIG` 指定配置，减少对 `cwd` 的依赖 |
| `env` | 可选；`RUST_LOG=warn` 减少 stderr 日志 |

**配置文件**（`config/default.toml`、缓存目录等）解析顺序：`--config` / `FANALYZER_CONFIG` → 当前目录 `config/default.toml` → 可执行文件相对路径 `../../config/default.toml`（适配 `target/debug/fanalyzer`）。

```bash
fanalyzer --config /path/to/config/default.toml mcp serve --profile summary
```

**`--profile` 选项**：

| 值 | 说明 |
|----|------|
| `summary` | 最省 token（默认推荐 Agent 对话） |
| `standard` | 省略时间序列，保留完整业务字段 |
| `full` | 含 `series` 时间序列 |

**`--tools` 选项**（控制 `tools/list` 暴露范围；Windsurf 等客户端有工具数量上限时推荐 `minimal`）：

| 值 | 说明 |
|----|------|
| `minimal` | 6 个核心工具：analyze、compare、research_fund、compare_watchlist、watchlist_list、portfolio |
| `standard` | minimal + export、brief、portfolio_config、自选增删 |
| `full` | 全部 Agent 工具（默认） |

**Resources**：MCP `resources/list` 提供 `fanalyzer://schemas/index`、`fanalyzer://watchlist`、`fanalyzer://portfolio`、`fanalyzer://config`（只读上下文，减少 Agent 猜路径）。

### Cursor

**项目级**（推荐）：在仓库根目录自行创建 `.cursor/mcp.json`，内容见上方 [通用 MCP 配置](#通用-mcp-配置)。

**全局**：Settings → MCP → Add server，填入相同 `command` / `args` / `cwd`。

保存后在 MCP 面板 **Refresh** 或重启 Cursor；应能看到 `fanalyzer_analyze`、`fanalyzer_research_fund` 等工具。

### Trae

**项目级**（推荐）：在仓库根目录自行创建 `.trae/mcp.json`：

```json
{
  "mcpServers": {
    "fanalyzer": {
      "command": "${workspaceFolder}/target/debug/fanalyzer",
      "args": ["mcp", "serve", "--profile", "summary", "--tools", "minimal"],
      "env": { "RUST_LOG": "warn" }
    }
  }
}
```

Trae 支持 `${workspaceFolder}` 变量，会展开为当前项目根路径；MCP 进程工作目录通常即为项目根，`config/watchlist.toml` 等相对路径可正常读取。若不用变量，也可改用通用配置中的绝对路径 + `cwd`。

也可在 **Settings → MCP** 中手动添加，或从其他 IDE 的 JSON 粘贴。修改后 **Reload Window**（`Ctrl/Cmd+Shift+P` → Developer: Reload Window）。

在 Builder / Agent 对话中直接描述基金分析需求即可。

### Claude Code

**项目级**（推荐团队共享）：仓库根目录 `.claude/mcp.json`，格式与通用配置相同。

**CLI 添加**（写入用户级配置）：

```bash
claude mcp add fanalyzer \
  /path/to/fanalyzer/target/debug/fanalyzer \
  --args mcp serve --profile summary \
  --cwd /path/to/fanalyzer
```

（具体子命令以本机 `claude mcp --help` 为准。）在 Claude Code 会话中直接提问即可调用工具。

### Windsurf（Cascade）

**仅全局配置**（不支持项目级 MCP 文件）：

`~/.codeium/windsurf/mcp_config.json`

也可在 Cascade 面板 → **MCP** 图标 → 编辑配置。JSON 格式与通用配置相同。修改后 **重启 Windsurf**。

> Windsurf 对所有 MCP 工具有数量上限（约 100 个）；若与其他 Server 同开导致工具被截断，请为 fanalyzer 加上 `--tools minimal` 或 `--tools standard`。

### Continue（VS Code / JetBrains）

编辑 Continue 配置（**Continue: Open Config**）中的 `mcpServers`，例如：

```yaml
mcpServers:
  - name: fanalyzer
    command: /path/to/fanalyzer/target/debug/fanalyzer
    args:
      - mcp
      - serve
      - --profile
      - summary
    cwd: /path/to/fanalyzer
    env:
      RUST_LOG: warn
```

保存后重载窗口；在 Continue Chat 中提问即可。

### 其他 Agent / 自动化

| 方式 | 适用场景 |
|------|----------|
| **MCP stdio** | 任何支持 `command` + `args` 的 MCP 客户端，配置同通用 JSON |
| **CLI JSON** | 不支持 MCP 的 Agent、n8n、GitHub Actions、自研脚本 |
| **Function Calling** | 导入 `schemas/tools.v1.agent.json` 或 `tools.v1.agent.embedded.json`，自行 subprocess 调 `fanalyzer json ...` |

CLI 示例：

```bash
/path/to/fanalyzer/target/debug/fanalyzer json --profile summary analyze 110011 --days 90 2>/dev/null | jq .
```

### 在 Agent 中使用（通用）

在 **Agent / 对话模式** 下直接提问，例如：

- 「用 fanalyzer 分析基金 110011，最近 90 天，总结夏普比率和最大回撤。」
- 「用 fanalyzer_research_fund 研究 110011。」
- 「列出当前自选基金，并把 110011 加进去。」

Agent 会通过 MCP 调用工具；返回内容为 JSON 信封（`ok` / `data` / `error`），失败时可查看 `error.hint` 与 `error.retryable`。

### 命令行自测

确认 MCP 正常（应输出 `true`，而非 `NO_OUTPUT`）：

```bash
cd /path/to/fanalyzer
cargo build

printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"fanalyzer_analyze","arguments":{"code":"110011","days":90}}}' \
| ./target/debug/fanalyzer mcp serve --profile summary 2>/dev/null \
| jq -r '.result.content[0].text | fromjson | .ok'
```

列出已注册工具：

```bash
printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1"}}}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' \
| ./target/debug/fanalyzer mcp serve 2>/dev/null | tail -1 | jq '.result.tools[].name'
```

### 常用 MCP 工具

| 工具 | 说明 |
|------|------|
| `fanalyzer_analyze` | 单基金分析 |
| `fanalyzer_compare` | 多基金对比 |
| `fanalyzer_research_fund` | 复合：info + analyze + sectors + holdings |
| `fanalyzer_compare_watchlist` | 对比自选列表全部基金 |
| `fanalyzer_watchlist_list` / `_add` / `_remove` | 自选增删查 |
| `fanalyzer_screen` / `fanalyzer_rank` | 筛选 / 排行 |

### 不用 MCP 的替代方式

任意能执行 shell 的环境可使用结构化 CLI：

```bash
cargo run -- json --profile summary analyze 110011 --days 90 2>/dev/null | jq .
```

## 环境变量

复制 `.env.example` 为 `.env` 并填入实际值：

```bash
cp .env.example .env
```

## 免责声明与数据合规

完整条款见 **[docs/DISCLAIMER.md](docs/DISCLAIMER.md)**，核心要点：

- 数据主要来自东方财富 / 天天基金等公开渠道；**与官方无关联、无授权**
- 输出仅供**个人非商业研究**，**不构成**投资建议
- 须遵守法律法规及第三方服务条款，**合理控制访问频率**
- 数据与指标可能存在延迟、误差或解析失败，**请以基金官方公告为准**

## 许可证

MIT License — 详见 [LICENSE](LICENSE)。
