# Project Rules for analysis_fund

## Always Apply

- Follow conventions in [AGENTS.md](../../AGENTS.md)
- Error handling: `thiserror` for library code, `anyhow` for application code
- Logging: use `tracing` crate, never `println!`
- Run verification after changes: `cargo fmt -- --check && cargo clippy -- -D warnings && cargo test`

## Rust Source Files

- Unit tests in `#[cfg(test)] mod tests` within the same file
- No `unwrap()` on user-facing paths; use proper error propagation
- Maximum line width: 100 characters (cargo fmt 默认值)

## New Dependencies

- Must justify in PR description
- Check license compatibility (no GPL)
- Prefer well-maintained crates with >1000 GitHub stars
