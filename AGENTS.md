# AGENTS.md — AI Agent Entry Document

## Project Identity

- **Name**: Fanalyzer (`fanalyzer`)
- **Description**: Rust 基金数据分析 CLI 工具
- **Tech Stack**: Rust 2021 Edition, Tokio, Reqwest, Clap, Serde, tracing, anyhow/thiserror, chrono
- **Directory**: See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for layout

## Hard Constraints

### Dependency Direction

- `CLI → Application → Domain ← Models`
- `Application → Presentation`；`Application → API/Infra`（经 `Session`）
- `API/Infra → Models`
- **禁止反向依赖**：
  - `domain` / `models` 不得依赖 `api`、`cli`、`application`、`presentation`
  - `presentation` 不得依赖 `application`（呈现层只接收已算好的数据）
  - `config` 不得依赖 `cli` 层
- `services/` 仅为兼容 re-export，**新代码必须使用** `domain` / `application`

### Layer Responsibilities（分层职责）

| 层 | 目录 | 允许 | 禁止 |
|----|------|------|------|
| 入口 | `cli/` | Clap 定义、`dispatch` 映射参数 → Request | 业务逻辑、HTTP、分析公式、`println!` |
| 用例 | `application/` | 编排 IO；构造/传递 `CommandContext` / `Session` | 解析 Clap；终端表格渲染 |
| 领域 | `domain/` | 纯函数、规则、指标计算 | 任何 IO（HTTP、文件、缓存、`tracing` 以外的副作用） |
| 呈现 | `presentation/` | 表格、报告、CSV/JSON/Markdown 输出 | 拉取数据、修改缓存 |
| 基础设施 | `api/`、`cache/`、`nav_cache/` | HTTP、持久化缓存 | 业务编排、CLI 参数 |

新增代码必须先确定所属层；若单文件/function 职责跨层，**拆分**而非 `#allow` 或继续堆叠。

### Explicit State（显式状态）

运行时可变状态**仅**允许从入口注入，经参数向下传递：

| 状态 | 载体 | 说明 |
|------|------|------|
| HTTP 客户端 | `Session.client` | 单次 `cli::run` 生命周期内创建 |
| 名称↔代码映射 | `Session.name_cache` | 经 `Session` 读写，禁止模块级缓存单例 |
| 净值 JSON 缓存 | `Session.nav_store` | 同上 |
| 离线/联网 | `CommandContext.offline` | 不得在 `domain` 分支判断 |
| 自选文件路径 | `CommandContext.watchlist_path` | 不得在深层硬编码路径 |

**禁止**：`static mut`、全局 `lazy_static` 业务状态、在 `domain` 内读缓存或 client、函数签名超过 7 个参数（改用 `XxxRequest` 结构体）。

数据流固定为：

```text
Cli::parse → CommandContext → application::run_*(ctx, Request) → domain（纯计算）→ presentation（输出）
```

### Prohibited Libraries

- 不引入 `print` / `println!` 替代日志（`presentation` 层用户可见输出除外；开发调试用的 `print` 提交前必须移除）
- 不引入未在 Cargo.toml 中声明的依赖，新增依赖需在 PR 中说明理由
- 不引入 GPL 许可证的库

### Security Red Lines

- 不得在代码或提交中包含 API Key、Token、证书等敏感信息
- 不得绕过权限检查逻辑
- 敏感配置仅通过环境变量或 `.env` 注入

### Code Annotations

- Agent 生成的代码**无需**特定标注

### Test Requirements

- 新功能必须同时编写测试（单元测试 + 必要的集成测试）
- **`domain/` 变更必须有同文件单元测试**（无 IO，应易测）
- **新 CLI 子命令至少补充** `tests/integration_test.rs` 中 `--help`  smoke 测试
- 测试分层遵循本文「测试策略」章节
- 提交前 `cargo test` 必须通过

## Verification Commands

Agent 完成修改后**必须**运行以下命令：

```bash
cargo fmt -- --check
python3 scripts/check_code_metrics.py
cargo clippy --all-targets -- -D warnings -W clippy::cognitive_complexity -W clippy::too_many_lines
cargo test
```

阈值见仓库根目录 **`.clippy.toml`**：**认知复杂度**（`cognitive-complexity-threshold`，对应 `-W clippy::cognitive_complexity`）与 **单函数行数**（`too-many-lines-threshold`，对应 `-W clippy::too_many_lines`）。单文件物理行数上限由 **`scripts/check_code_metrics.py`** 扫描 **`src/`**、**`tests/`**。

超阈值时：**拆函数/拆模块**，禁止仅在局部 `#allow(clippy::...)` 除非有注释说明理由。

## Coding Conventions

### Adding a CLI Command（新子命令 checklist）

1. 在 `cli/mod.rs` 的 `Commands` 增加 Clap 字段
2. 在 `cli/dispatch_query.rs` 或 `cli/dispatch_workflow.rs` 映射为 `application::*Request`
3. 在 `application/` 实现用例（接收 `&CommandContext<'_>`，不接收 `Cli`）
4. 计算逻辑放 `domain/`；输出放 `presentation/`
5. 更新 `docs/MANUAL.md`；补充集成测试

### Error Handling

- **库层**（`models`、`api`）：使用 `thiserror` 定义自定义错误类型，返回 `Result<T, E>`
- **应用层**（`application`、`cli` 入口）：使用 `anyhow::Result` 作为顶层错误传播
- **可恢复错误**：`Result<T, E>`，调用方决定如何处理
- **不可恢复错误**：仅 `panic!` / `unwrap` 用于编程错误（如逻辑不变量违反），**不得**用于可预期的运行时错误（网络失败、用户输入错误等）
- **错误包装**：底层错误用 `#[from]` 或 `.context()` 添加上下文，避免裸 `?` 丢失信息

### Logging

- 使用 `tracing` 框架；业务路径禁止用 `println!` / `print` 作日志（用户可见表格/报告由 `presentation` 负责）
- 日志级别约定：
  - `ERROR`：需人工介入的严重错误
  - `WARN`：可自动恢复的异常（如配置缺失使用默认值）
  - `INFO`：关键业务事件（如启动、数据获取、分析完成）
  - `DEBUG`：开发调试信息（如 HTTP 请求详情）
  - `TRACE`：详细追踪（如函数进出）
- 结构化字段：使用 `tracing::info!(key = value, "message")` 格式
- 开发环境输出 stderr，生产环境可配置 JSON 格式

### Configuration Management

- 优先级（从高到低）：命令行参数 → 环境变量 → 配置文件 → 代码内默认值
- 同一默认值**只在一处**定义（优先 `domain` 常量或 Request 构造处），避免 CLI / application / domain 各写一份
- 配置文件格式：TOML
- 配置文件位置：项目根目录 `config/default.toml`
- 敏感配置（密钥、Token）仅通过环境变量注入，不得写入配置文件或代码

### Anti-Patterns（禁止）

- 在 `domain` 调用 `EastMoneyClient` 或读写 `nav_cache`
- 在 `cli` 或 `handlers` 式大模块堆叠多种用例
- 用 `HashMap<String, f64>` 传递有明确含义的指标（应使用 `FundAnalysis` 等类型）
- 在调用链中途隐式依赖全局配置或环境变量（应从 `CommandContext` / `AppConfig` 显式传入）
- 为通过 clippy 行数门控而复制粘贴；应提取到 `domain` 或 `application` 子模块

### Dependency Version Policy

- 依赖版本使用 `^` 语义（Cargo 默认）
- dev-dependencies 仅用于测试、基准，不得泄漏到正式构建

## Testing Strategy

### Framework

- 内置 `#[test]` + `#[cfg(test)]` 模块
- 集成测试：`assert_cmd` + `predicates`（CLI 测试）

### Test Layers

- **单元测试**：与源码同文件 `#[cfg(test)] mod tests`；**`domain/` 优先覆盖**
- **集成测试**：`tests/` 目录，测试模块间交互与 CLI 行为
- **端到端测试**：依赖外部 API 时需 mock 或跳过；优先测 `domain` + `application` 纯逻辑

### Coverage

- 暂不强设覆盖率门控，鼓励新功能附带测试
- 工具：`cargo-llvm-cov`（按需启用）

## Documentation & Language Conventions

- 代码注释语言：中文
- 提交信息格式：`type(scope): description`（英文）
- 文档语言：中文为主

## Collaboration Entry

- Security Policy: [SECURITY.md](SECURITY.md)
- PR Template: [.github/pull_request_template.md](.github/pull_request_template.md)
