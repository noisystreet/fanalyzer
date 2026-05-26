# Contributing to Fanalyzer

Thank you for your interest in contributing! Please follow the guidelines below.

## How to Contribute

1. Fork the repository
2. Create a feature branch (`feat/your-feature`)
3. Make your changes with tests
4. Ensure all checks pass: `cargo fmt -- --check && cargo clippy -- -D warnings && cargo test`
5. Submit a Pull Request

## Code Style

- Run `cargo fmt` before committing
- Address all `cargo clippy` warnings
- Follow the error handling and logging conventions in AGENTS.md

## Commit Messages

Format: `type(scope): description`

Types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `ci`

Example: `feat(api): add fund search endpoint`

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Security

Do not report security vulnerabilities in public issues. See [SECURITY.md](SECURITY.md).
