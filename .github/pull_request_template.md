## Description

<!-- 简述改动目的与影响范围 -->

## Type of Change

- [ ] feat: New feature
- [ ] fix: Bug fix
- [ ] docs: Documentation change
- [ ] refactor: Code refactoring
- [ ] test: Test addition or update
- [ ] chore: Build or tooling change
- [ ] ci: CI / GitHub automation

## Checklist

与 [`.github/workflows/ci.yml`](.github/workflows/ci.yml) 保持一致，合并前请确认本地通过：

```bash
cargo fmt -- --check
python3 scripts/check_code_metrics.py
python3 scripts/generate_schemas.py --check
cargo clippy --all-targets -- -D warnings -W clippy::cognitive_complexity -W clippy::too_many_lines
cargo clippy --all-targets --features web -- -D warnings -W clippy::too_many_lines -A clippy::cognitive_complexity
cargo test
cargo test --features web
```

- [ ] 上述命令（或等效 CI）已通过
- [ ] 新功能/修复附带测试（`domain/` 变更需同文件单测）
- [ ] 新 CLI 子命令已补充 `tests/integration_test.rs` `--help` smoke（如适用）
- [ ] 修改 JSON 信封或 MCP 工具时，已运行 `python3 scripts/generate_schemas.py` 并提交 `schemas/`
- [ ] 文档已更新（`docs/MANUAL.md` / `docs/AGENT.md` / `CHANGELOG.md`，如适用）
