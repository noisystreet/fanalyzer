# Fanalyzer 使用手册

面向终端用户的命令说明。

> **免责声明与数据使用边界**见 **[DISCLAIMER.md](DISCLAIMER.md)**。本手册所述功能输出仅供个人研究参考，**不构成投资建议**；数据主要来自东方财富 / 天天基金公开渠道，请合规、低频使用。

## 运行方式

在项目根目录执行（需已安装 Rust 工具链）：

```bash
cargo run -- <子命令与参数>
```

编译安装后可省略 `cargo run --`：

```bash
cargo build --release
./target/release/fanalyzer <子命令与参数>
```

查看全局与子命令帮助：

```bash
cargo run -- --help
cargo run -- <子命令> --help
```

## 配置与自选文件

### 应用配置 `config/default.toml`

启动时若存在该路径则读取，否则使用内置默认值。

| 段落 | 字段 | 说明 |
|------|------|------|
| `[api]` | `timeout_secs` | HTTP 超时（秒） |
| `[api]` | `user_agent` | 可选，自定义请求 UA |
| `[api]` | `proxy` | 可选，如 `http://127.0.0.1:7890` |
| `[log]` | `level` | 日志级别，如 `info`、`debug` |

`base_url` 为历史字段，当前 CLI 主要走东方财富固定接口，可保留默认。

### 自选列表 `config/watchlist.toml`

TOML 中 `funds` 为字符串数组，每项为 **6 位基金代码** 或 **基金名称/简称**（与单基金 `--code` 解析规则一致）：

```toml
funds = ["000001", "110011", "某基金简称"]
```

- 与 `--watchlist` 联用时，从该文件批量拉取标的（见下表「自选」列）。
- 默认路径为 `config/watchlist.toml`；可用全局参数 **`--watchlist-file <路径>`** 指定其他文件。

## 全局参数

| 参数 | 说明 |
|------|------|
| `--offline` | 仅使用本地已缓存的净值数据。**不可**与 `fetch`、`info`、`rank`、`brief`、`screen` 等需联网子命令共用；`analyze` / `compare` / `portfolio` / `export` 在已有缓存时可用。 |
| `--watchlist-file <PATH>` | 自选文件路径，默认 `config/watchlist.toml`。 |
| `--json`（`--structured`） | 仅向 **stdout** 输出结构化 JSON，便于 Agent / 脚本解析；人类可读表格不再打印。日志仍在 **stderr**。可与 `--output` 同时写入文件。 |
| `--json-compact` | 与 `--json` 联用：紧凑单行 JSON（管道 / jq 友好）。 |
| `--compact-series` | 与 `--json` 联用：省略 `series` 时间序列，减少 token 占用。 |

## 结构化输出（Agent 集成）

完整 Agent 集成说明见 **[AGENT.md](AGENT.md)**；JSON Schema 见 **[schemas/envelope.v1.json](../schemas/envelope.v1.json)**。

大模型 Agent 或自动化脚本可使用全局 **`--json`**（别名 **`--structured`**）获取统一 JSON 信封，便于解析与后续分析。

**约定：**

- **stdout**：成功与失败均为 JSON（默认 pretty-print；`--json-compact` 为单行）
- **stderr**：`tracing` 日志（进度、警告等）
- 退出码：`0` 表示成功；失败时 stdout 仍输出 `{ "ok": false, "error": {...} }`

**成功信封格式：**

```json
{
  "v": 1,
  "command": "analyze",
  "ok": true,
  "meta": { "offline": false, "generated_at": "...", "days": 90, "requested": 1, "succeeded": 1 },
  "warnings": [],
  "data": { "items": [], "errors": [] }
}
```

**失败信封格式：**

```json
{
  "v": 1,
  "command": "compare",
  "ok": false,
  "meta": { "offline": false, "generated_at": "..." },
  "error": { "code": "INSUFFICIENT_SAMPLES", "message": "..." }
}
```

| 字段 | 说明 |
|------|------|
| `v` | 信封版本，当前为 `1` |
| `command` | 子命令名（如 `analyze`、`portfolio`） |
| `ok` | 是否成功 |
| `meta` | 请求上下文（离线、时间戳、分析窗口等） |
| `warnings` | 非致命警告（部分标的跳过等） |
| `data` | 命令专用 payload（失败时省略） |
| `error` | 失败时 `{ code, message }`（成功时省略） |

批量命令的 `data` 含 `items[]` 与可选 `errors[]`（partial success）。

**示例子命令：**

```bash
# 单基金分析 → data.items[] 为 FundAnalysisReport
cargo run -- --json analyze 110011 --days 90

# 紧凑 + 省略曲线
cargo run -- --json --json-compact --compact-series analyze 110011 --days 90

# 组合分析 → data 为 PortfolioReport 对象
cargo run -- --json portfolio --weights 110011:0.6,005827:0.4 --days 180

# 筛选 → data 含 pool_size、passed 等
cargo run -- --json screen --kind gp --days 90 --max-drawdown 0.15

# 导出净值（须 --format json；可不指定 --output-dir，JSON 直接 stdout）
cargo run -- --json export 110011 --format json --days 365
```

**支持 `--json` 的子命令：** `fetch`、`analyze`、`compare`、`portfolio`、`export`、`info`、`sectors`、`holdings`、`rank`、`brief`、`screen`。

**注意：** `export --json` 要求 `--format json`；`serve` 为 Web 服务，不使用此模式。

## 子命令总览

| 子命令 | 需要联网 | 自选 `--watchlist` | 说明 |
|--------|----------|-------------------|------|
| `fetch` | 是 | 支持 | 拉取净值历史并打印 |
| `analyze` | 否（`--offline` 时仅缓存） | 支持 | 收益、风险、经理与费率等分析 |
| `compare` | 否（`--offline` 时仅缓存） | 支持 | 多基金对比 |
| `portfolio` | 否（`--offline` 时仅缓存；重叠需联网） | 不适用 | 组合加权分析、相关矩阵、重仓重叠 |
| `export` | 否（`--offline` 时仅缓存） | 支持（须 `--output-dir`） | 导出净值 CSV/JSON |
| `info` | 是 | 支持 | 基金概况与招募说明书摘要类字段 |
| `sectors` | 是 | 支持 | 季报「行业配置」（证监会行业分类），用于板块/行业集中度浏览 |
| `holdings` | 是 | 支持 | 季报「股票投资明细」重仓股（`FundArchivesDatas type=jjcc`） |
| `rank` | 是 | 不适用 | 按天天基金官网排行接口拉取某类型全市场前 N 名 |
| `brief` | 是 | 支持 | 单基金/自选综合简报：概况 + 分析 + 行业 + 重仓 |
| `screen` | 是 | 不适用 | 从类型排行池中按回撤/夏普/费率筛选，并对比通过者 |
| `serve` | 是 | 不适用 | 启动 Leptos SSR Web 界面（**须** `cargo run --features web -- serve`） |

---

## `fetch` — 获取净值

```bash
cargo run -- fetch --code 000001
cargo run -- fetch --code 000001 --limit 100
cargo run -- fetch --watchlist --limit 50
```

| 参数 | 说明 |
|------|------|
| `-c` / `--code` | 单只基金代码或名称 |
| `--watchlist` | 处理自选文件中的全部基金 |
| `-l` / `--limit` | 拉取条数，默认 `20` |

---

## `analyze` — 分析单只或多只（自选）

```bash
cargo run -- analyze --code 000001 --days 90
cargo run -- analyze --code 000001 --period 1y
cargo run -- analyze --code 000001 --period 1y --output ./analysis.json --format json
cargo run -- analyze --code "华夏成长混合" --period 3m
cargo run -- analyze --watchlist --days 60
```

| 参数 | 说明 |
|------|------|
| `-c` / `--code` | 与 `--watchlist` 二选一（自选模式下可不写） |
| `--watchlist` | 对自选文件中每只基金各跑一次分析 |
| `-d` / `--days` | 分析窗口（**日历天**），默认 `30`；可被 `--period` 覆盖 |
| `--period` | 预设窗口：`7d`/`1m`/`3m`/`6m`/`1y`/`ytd`，或 rank 的 `sc`（如 `1nzf`、`zzf`） |
| `-o` / `--output` | 导出 JSON 报告（含标量指标与时间序列） |
| `-f` / `--format` | 导出格式，目前支持 `json` |
| `--rolling-window` | 滚动指标窗口（交易日，10～252），默认 `60` |

**分析口径（重要）：**

- 收益、回撤、波动等优先使用 **累计净值 `acc_nav`**（若有效），以更好反映分红再投资；导出 CSV 仍保留 `nav` / `acc_nav` 两列。
- **Alpha / Beta** 在线时按 F10 **业绩比较基准** 文案推断指数（如沪深300、中证500）；无法识别时按基金类型兜底，再不行用沪深300。
- 新增 **索提诺比率**、**卡玛比率**（收益/最大回撤），与夏普一并输出。
- **时间序列（v0.3）**：默认 60 交易日滚动窗口，输出归一化净值、回撤、滚动夏普/波动率/Beta（JSON 导出与 Web 图表共用）。

`--offline` 时从本地净值缓存读取；无契约基准与经理/费率，Alpha/Beta 为 0。

---

## `compare` — 对比多只基金

```bash
cargo run -- compare --codes 000001,000003 --period 1y
cargo run -- compare --codes 000001,000003 --days 90 --sort sharpe
cargo run -- compare --watchlist --period 6m --output ./cmp.csv --format csv
cargo run -- compare --watchlist --period 3m --output ./cmp.json --format json
```

| 参数 | 说明 |
|------|------|
| `--codes` | 逗号分隔的代码或名称，至少 2 只；与 `--watchlist` 二选一 |
| `--watchlist` | 使用自选列表中全部基金参与对比（有效样本仍须 ≥2） |
| `-d` / `--days` | 分析窗口（日历天），默认 `30` |
| `--period` | 同 `analyze` |
| `--sort` | 结果排序：`sharpe`/`sortino`/`calmar`/`total-return`/`max-drawdown`/`alpha`/`volatility`（风险类指标默认升序，其余降序） |
| `-o` / `--output` | 导出对比表（CSV 或 JSON） |
| `-f` / `--format` | 导出格式，`csv`（默认）或 `json` |

对比时 **每只基金独立解析契约基准** 计算 Alpha/Beta；表格含 Sortino、Calmar 列。

---

## `portfolio` — 组合分析

按权重配置计算组合层风险收益、成分相关矩阵与重仓重叠度。

```bash
cargo run -- portfolio --portfolio-file config/portfolio.toml --period 1y
cargo run -- portfolio --days 90 --holdings-top 10
cargo run -- portfolio --period 6m --output ./portfolio.json
cargo run -- portfolio --offline --days 90   # 跳过重仓重叠，仅用净值缓存
```

### 组合配置文件 `config/portfolio.toml`

```toml
name = "demo-equal-weight"

[[holdings]]
code = "000001"
weight = 0.5

[[holdings]]
code = "110011"
weight = 0.5
```

- `code`：6 位基金代码或名称（与 `--code` 解析规则一致）
- `weight`：目标权重；合计不为 1.0 时在 weight > 0 前提下自动归一化
- 至少 2 只有效持仓

| 参数 | 说明 |
|------|------|
| `--portfolio-file` | 组合 TOML 路径，默认 `config/portfolio.toml` |
| `-d` / `--days` | 分析窗口（日历天），默认 `90` |
| `--period` | 同 `analyze` |
| `--holdings-top` | 重仓重叠分析取前 N 大重仓，默认 `10`（需联网） |
| `-o` / `--output` | 导出 JSON 报告 |
| `-f` / `--format` | 目前支持 `json` |
| `--rolling-window` | 滚动指标窗口（交易日，10～252），默认 `60` |

**输出说明：**

- **组合指标**：按成分日收益加权合成组合曲线，再算总收益、年化、波动、回撤、夏普
- **时间序列**：组合归一化净值、回撤、滚动夏普/波动率（JSON 与 Web 图表）
- **成分贡献**：静态近似 `weight × 单基总收益`
- **相关矩阵**：成分日收益 Pearson 相关（日期取交集）
- **重仓重叠**：对共同持仓取 `min(占净值%)` 之和；`--offline` 时跳过
- **分析解读**：基于规则引擎自动生成要点（风险、集中度、相关性、重叠、**等权对比**等），阈值见 `config/portfolio_insights.toml`，**不构成投资建议**

### 解读阈值 `config/portfolio_insights.toml`

可调整相关/重叠/集中度/夏普/等权对比等触发阈值；文件缺失时使用内置默认值。

---

## `export` — 导出净值序列

```bash
cargo run -- export --code 000001 --days 90 --output ./nav.csv --format csv
cargo run -- export --code 000001 --days 90 --output ./nav.json --format json
cargo run -- export --watchlist --days 90 --output-dir ./out --format csv
```

| 参数 | 说明 |
|------|------|
| `-c` / `--code` | 单基金导出时必填（与 `--watchlist` 二选一） |
| `--watchlist` | 批量导出；**必须**同时指定 `--output-dir` |
| `-d` / `--days` | 日历天窗口，默认 `30` |
| `-o` / `--output` | 单基金输出文件路径 |
| `--output-dir` | 自选模式下输出目录；文件名形如 `{代码}.csv` |
| `-f` / `--format` | `csv` 或 `json`，默认 `csv` |

---

## `info` — 基金概况

```bash
cargo run -- info --code 000001
cargo run -- info --watchlist
```

需联网；输出包含类型、规模、公司、经理、费率及投资目标/范围等（以接口返回为准）。

---

## `sectors` — 行业配置（板块分析）

依据天天基金 **F10 → 行业配置**（接口 `type=hypz`），展示基金季报披露的 **证监会行业分类** 持仓占比与市值（万元）。适用于观察基金在制造、信息技术等板块上的暴露；**不等于**申万行业或概念题材板块。

```bash
cargo run -- sectors --code 000001
cargo run -- sectors --watchlist
```

| 参数 | 说明 |
|------|------|
| `-c` / `--code` | 与 `--watchlist` 二选一 |
| `--watchlist` | 对自选列表逐只拉取行业配置 |

**说明：** 债券型、货币型等股票仓位极低的基金通常无有效行业表或为空；数据频率随季报更新。接口返回 HTML 内嵌表格，解析规则随官网改版可能需跟进。

---

## `holdings` — 重仓股（股票投资明细）

依据天天基金 **基金持仓 → 股票明细**（接口 `FundArchivesDatas.aspx`、`type=jjcc`），展示季报披露的 **前 N 条** 重仓股票：代码、名称、占净值比例、持股数（万股）、持仓市值（万元）。与网页「报告期末占基金资产净值比例排序的股票一览」一致。

```bash
cargo run -- holdings --code 000001
cargo run -- holdings --code 000001 --top 15
cargo run -- holdings --watchlist --top 10
```

| 参数 | 说明 |
|------|------|
| `-c` / `--code` | 与 `--watchlist` 二选一 |
| `--watchlist` | 对自选列表逐只查询 |
| `-t` / `--top` | 对应接口 `topline`，默认 `10`，范围 **1～50**（超出将被限制为 50） |

**说明：** 债券型、货币型基金通常无股票明细表；表中「最新价」「涨跌幅」由官网脚本异步填充，本工具不请求行情接口故留空列不展示。解析依赖 HTML 结构，改版后可能需要更新。

---

## 选基工作流：`brief` 与 `screen`

两条命令把「单只尽调」与「全市场初筛」串起来，适合日常选基：先用 `screen` 从排行池里按规则缩小范围，再对少数标的用 `brief` 出一份可读报告。

### `brief` — 单基金综合简报

一次输出 **概况**（类型、公司、规模）+ **`analyze` 指标** + **行业配置前 N 项** + **重仓股前 N 条**。可选写入 Markdown。

```bash
# 单只基金，默认近 90 天、行业前 5、重仓前 10
cargo run -- brief --code 000001

# 指定分析窗口与展示条数
cargo run -- brief --code 000001 --days 180 --industry-top 8 --holdings-top 15

# 自选逐只简报，并保存 Markdown
cargo run -- brief --watchlist --output reports/brief.md

# 单只 + 独立报告文件
cargo run -- brief --code "华夏成长" --output brief_000001.md
```

| 参数 | 说明 |
|------|------|
| `-c` / `--code` | 与 `--watchlist` 二选一 |
| `--watchlist` | 对自选列表逐只生成简报（多只时间隔打印分隔线） |
| `-d` / `--days` | 净值分析窗口（日历天），默认 `90`；可被 `--period` 覆盖 |
| `--period` | 同 `analyze` |
| `--industry-top` | 行业配置展示前 N 项，默认 `5` |
| `--holdings-top` | 重仓股条数，默认 `10`（接口上限 50） |
| `-o` / `--output` | 可选，将同一份内容写入 Markdown 文件 |

**说明：** 需联网；行业与重仓为 **季报** 口径；分析段使用累计净值与契约基准（同 `analyze`）。`--offline` 不可用。

### `screen` — 排行池规则筛选 + 对比

从某类型 **`rank` 前 N 名** 中，先用 **排行区间收益**（`--min-rank-return`，列与 `--sort` 一致）预筛，再对少量候选做 **deep 分析**（默认最多 15 只，可用 `--full-scan` 扫全池），按回撤/夏普/Alpha 等过滤，最后排序并输出对比表。

```bash
# 近 1 年排行前 30，区间收益 ≥10%，再 deep 分析并按夏普排序
cargo run -- screen --kind gp --sort 1nzf --min-rank-return 10 --max-drawdown 25 --min-sharpe 0.5

# 显式指定 deep 窗口与导出
cargo run -- screen --kind 混合 --sort 3yzf --period 3m --deep-limit 20 --sort-by calmar --output out.csv

# 扫全池（较慢）
cargo run -- screen --kind gp --rank-top 50 --full-scan --min-sharpe 0.8
```

| 参数 | 说明 |
|------|------|
| `-k` / `--kind` | 同 `rank --kind` |
| `--sort` | 同 `rank --sort`（`sc`），默认 `1n` |
| `--rank-top` | 从排行前 N 只取候选，默认 `30`（5～100） |
| `-d` / `--days` | deep 分析窗口；**省略时按 `--sort` 区间自动对齐**（如 `1nzf`→365 天） |
| `--period` | 同 `analyze`，覆盖 `-d` / 自动对齐 |
| `--min-rank-return` | 排行区间收益下限（**百分点**，与 `--sort` 列一致） |
| `--max-drawdown` | 可选，最大回撤上限（百分点） |
| `--min-sharpe` | 可选，最低夏普 |
| `--max-mgmt-fee` | 可选，管理费率上限（百分点） |
| `--min-alpha` | 可选，最低 Alpha（百分点） |
| `--max-volatility` | 可选，波动率上限（百分点） |
| `--min-total-return` | 可选，deep 分析区间总收益下限（百分点） |
| `--deep-limit` | deep 分析最多只数，默认 `15` |
| `--full-scan` | 对预筛后全部候选做 deep 分析 |
| `--sort-by` | 通过筛选后的排序键，默认 `sharpe` |
| `-l` / `--limit` | 对比展示上限，默认 `10`（2～30） |
| `-o` / `--output`、`-f` / `--format` | 同 `compare` 导出 |

**说明：** 默认 `--deep-limit 15` 避免对整池逐只拉净值；需全量 deep 分析时加 `--full-scan`。`--sort` 与 `--kind` 含义见下文 `rank` 章节。

---

## `rank` — 某类型全市场排行 Top N

数据与天天基金「开放式基金排行」一致（接口需合法 Referer，由程序内置）。用于按类型浏览「当前排序下的前 N 名」，**不是**对本工具已持仓池的排序。

```bash
# 股票型，默认前 100；默认 sc=1n（近一年维度降序，与网页完全一致时可试 --sort 1nzf）
cargo run -- rank --kind gp

# 混合型前 50，按近 1 周（sc=zzf）降序
cargo run -- rank --kind hh --top 50 --sort zzf

# 与浏览器排行页默认排序字段对齐示例（近 1 年）
cargo run -- rank --kind 混合 --top 20 --sort 1nzf

# 中文类型别名
cargo run -- rank --kind 混合 --top 100
```

| 参数 | 说明 |
|------|------|
| `-k` / `--kind` | 类型代码或中文别名，见下表 |
| `-t` / `--top` | 取前 N 名，默认 `100`，**最大 500** |
| `--sort` | 对应接口查询参数 **`sc`**（排序依据哪一列）；见下文 **`--sort`（`sc`）说明** |

### `--sort`（对接官网 `sc`）

- **`--sort` 的值会原样传给** `rankhandler.aspx` 的 **`sc`**。
- 本工具请求里 **`st` 固定为 `desc`**，即按该列 **数值从高到低** 排序后再截取 `--top` 条（与排行页点击表头后常见的「降序」一致）。
- 浏览器打开 [开放式基金排行](http://fund.eastmoney.com/data/fundranking.html) 时，页面脚本里的默认排序字段多为 **`1nzf`**（近 1 年）；CLI **默认 `--sort` 为 `1n`**（接口仍可识别时常用于近 1 年维度）。若你发现 **`rank` 结果与网页默认视图不一致**，可显式加上 **`--sort 1nzf`** 再对比。
- 其它 **`sc` 未在下面列出时**：在网页上对某一列表头排序后，用开发者工具查看本次请求的 `rankhandler.aspx?...&sc=???`，即可得到当前对应的代码。

下表按官网排行表 **「可排序列 → `sc` 取值」** 归纳（与页面 `<th col="...">` 一致；官网改版时需以实际页面为准）。

| `--sort` / `sc` | 含义 |
|-----------------|------|
| `rzdf` | 日增长率 |
| `zzf` | 近 1 周 |
| `1yzf` | 近 1 月 |
| `3yzf` | 近 3 月 |
| `6yzf` | 近 6 月 |
| `1n`、`1nzf` | 近 1 年（后者与当前页默认写法一致，二者择一以接口返回为准） |
| `2nzf` | 近 2 年 |
| `3nzf` | 近 3 年 |
| `jnzf` | 今年来 |
| `lnzf` | 成立来 |
| `qjzf` | 自定义区间收益率（依赖页面选择的起止日期；无对应筛选时行为以官网为准） |

### `--kind` 与接口类型 `ft`

| `kind`（不区分大小写的英文） | 中文别名示例 | 含义 |
|------------------------------|--------------|------|
| `gp` | 股票、股票型 | 股票型 |
| `hh` | 混合、混合型 | 混合型 |
| `zq` | 债券、债券型 | 债券型 |
| `zs` | 指数、指数型 | 指数型 |
| `qdii` | — | QDII |
| `fof` | fof型 | FOF |

终端表格中会打印近 1 周、近 1 月、近 3 月、近 6 月、近 1 年、今年来等收益率（百分点，与解析列一致）。

**说明：** `rank` 不支持 `--offline`；若接口返回异常，请检查网络与代理配置（`config/default.toml` 中的 `proxy`）。

---

## `serve` — Web 界面（Leptos SSR）

需编译时启用 feature：

```bash
cargo run --features web -- serve
cargo run --features web -- serve --host 0.0.0.0 --port 8080
```

| 参数 | 说明 |
|------|------|
| `--host` | 监听地址，默认 `127.0.0.1` |
| `-p` / `--port` | 端口，默认 `3000` |
| `--portfolio-file` | Web 组合页**首次打开**时的预填来源（默认 `config/portfolio.toml`）；分析以页面表单为准 |

浏览器访问：

- `/` — 首页
- `/analyze?code=000001&days=90` — 单基金分析（含净值/回撤/滚动指标 SVG 图表）
- `/compare?codes=000001,110011&days=90&sort=sharpe` — 多基金对比
- `/portfolio?run=1&days=90` — 组合分析（含组合净值与滚动指标图表；支持「从自选导入等权」、浏览器 localStorage 暂存草稿）
- `/info?code=000001` — 基金概况（F10）
- `/brief?code=000001&days=90&industry_top=5&holdings_top=10` — 选基综合简报

**说明：** 纯 SSR（无 WASM  hydration）；需联网。HTTP 代理可配置 `config/default.toml` 的 `[api].proxy` 或环境变量 `http_proxy` / `https_proxy`。

---

## 日志与环境变量

日志由 `tracing` 输出；级别由配置 `[log].level` 等启动逻辑决定（详见代码与 `main`）。敏感凭据勿写入仓库；若项目提供 `.env.example`，可按说明复制为 `.env` 仅在本地使用。

## 常见问题

1. **`--offline` 报错**  
   先去掉 `--offline` 在线跑一次 `analyze`/`export` 等以写入净值缓存，再离线使用。

2. **自选为空**  
   确认 `config/watchlist.toml`（或 `--watchlist-file`）存在且 `funds` 中有非空项。

3. **名称解析失败**  
   离线时名称→代码依赖此前在线缓存；可直接使用 6 位数字代码。

4. **排行与网页不一致**  
   确认 `--kind`、`--sort`（`sc`）与浏览器打开的排行页筛选、排序一致。

## 免责声明

完整条款见 **[DISCLAIMER.md](DISCLAIMER.md)**。使用本工具即表示你理解：数据版权归原提供方所有，本工具不保证数据准确完整，你须自行承担投资与合规风险。
