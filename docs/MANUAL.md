# analysis_fund 使用手册

面向终端用户的命令说明。数据来源主要为东方财富开放式基金接口；输出仅供个人研究参考，不构成投资建议。

## 运行方式

在项目根目录执行（需已安装 Rust 工具链）：

```bash
cargo run -- <子命令与参数>
```

编译安装后可省略 `cargo run --`：

```bash
cargo build --release
./target/release/analysis_fund <子命令与参数>
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
| `--offline` | 仅使用本地已缓存的净值数据。**不可**与 `fetch`、`info`、`rank` 等需联网子命令共用；`analyze` / `compare` / `export` 在已有缓存时可用。 |
| `--watchlist-file <PATH>` | 自选文件路径，默认 `config/watchlist.toml`。 |

## 子命令总览

| 子命令 | 需要联网 | 自选 `--watchlist` | 说明 |
|--------|----------|-------------------|------|
| `fetch` | 是 | 支持 | 拉取净值历史并打印 |
| `analyze` | 否（`--offline` 时仅缓存） | 支持 | 收益、风险、经理与费率等分析 |
| `compare` | 否（`--offline` 时仅缓存） | 支持 | 多基金对比 |
| `export` | 否（`--offline` 时仅缓存） | 支持（须 `--output-dir`） | 导出净值 CSV/JSON |
| `info` | 是 | 支持 | 基金概况与招募说明书摘要类字段 |
| `rank` | 是 | 不适用 | 按天天基金官网排行接口拉取某类型全市场前 N 名 |

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
cargo run -- analyze --code "华夏成长混合" --days 30
cargo run -- analyze --watchlist --days 60
```

| 参数 | 说明 |
|------|------|
| `-c` / `--code` | 与 `--watchlist` 二选一（自选模式下可不写） |
| `--watchlist` | 对自选文件中每只基金各跑一次分析 |
| `-d` / `--days` | 分析窗口（**日历天**），默认 `30` |

`--offline` 时从本地净值缓存读取；若缓存不足会报错，需先在线执行过一次相关基金的抓取或分析以写入缓存。

---

## `compare` — 对比多只基金

```bash
cargo run -- compare --codes 000001,000003 --days 90
cargo run -- compare --codes "基金甲","基金乙" --days 30
cargo run -- compare --watchlist --days 60
```

| 参数 | 说明 |
|------|------|
| `--codes` | 逗号分隔的代码或名称，至少 2 只；与 `--watchlist` 二选一 |
| `--watchlist` | 使用自选列表中全部基金参与对比（有效样本仍须 ≥2） |
| `-d` / `--days` | 分析窗口（日历天），默认 `30` |

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
