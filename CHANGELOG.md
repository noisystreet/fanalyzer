# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- 项目重命名为 **Fanalyzer**（crate / CLI 二进制：`fanalyzer`）；本地缓存目录改为 `fanalyzer`

### Added
- Initial project scaffold
- CLI with `fetch` and `analyze` subcommands
- Fund data models (Fund, FundNav, FundAnalysis)
- API client skeleton
- Configuration management via TOML
- Structured logging with tracing
