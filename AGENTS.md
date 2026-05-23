# AGENTS.md — AI Agent Entry Document

## Project Identity

- **Name**: analysis_fund
- **Description**: Rust 基金数据分析 CLI 工具
- **Tech Stack**: Rust 2021 Edition, Tokio, Reqwest, Clap, Serde, tracing, anyhow/thiserror, chrono
- **Directory**: See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for layout

## Hard Constraints

### Dependency Direction

- `CLI → Application → Domain ← Models`
- `Application → Presentation`；`Application → API/Infra`（经 `Session`）
- `API/Infra → Models`
- **禁止反向依赖**：Domain/Models 不依赖 API 层；Config 不依赖 CLI 层
- `services/` 为兼容 re-export，新代码请用 `domain` / `application`

### Prohibited Libraries

- 不引入 `print` / `println!` 替代日志（开发调试除外，提交前必须移除）
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

## Coding Conventions

### Error Handling

- **库层**（models, api）：使用 `thiserror` 定义自定义错误类型，返回 `Result<T, E>`
- **应用层**（main, services）：使用 `anyhow::Result` 作为顶层错误传播
- **可恢复错误**：`Result<T, E>`，调用方决定如何处理
- **不可恢复错误**：仅 `panic!` / `unwrap` 用于编程错误（如逻辑不变量违反），不得用于可预期的运行时错误
- **错误包装**：底层错误用 `#[from]` 或 `.context()` 添加上下文，避免裸 `?` 丢失信息

### Logging

- 使用 `tracing` 框架，禁止 `println!` / `print` 作为日志输出
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
- 配置文件格式：TOML
- 配置文件位置：项目根目录 `config/default.toml`
- 敏感配置（密钥、Token）仅通过环境变量注入，不得写入配置文件或代码

### Dependency Version Policy

- 依赖版本使用 `^` 语义（Cargo 默认）
- dev-dependencies 仅用于测试、基准，不得泄漏到正式构建

## Testing Strategy

### Framework

- 内置 `#[test]` + `#[cfg(test)]` 模块
- 集成测试：`assert_cmd` + `predicates`（CLI 测试）

### Test Layers

- **单元测试**：与源码同文件 `#[cfg(test)] mod tests`
- **集成测试**：`tests/` 目录，测试模块间交互与 CLI 行为
- **端到端测试**：待后续扩展，依赖外部 API 时需 mock

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
