#!/usr/bin/env python3
"""校验仓库内约定目录下的 Rust 源文件物理行数不超过上限。"""

from __future__ import annotations

import sys
from pathlib import Path

# 物理行数（与 Unix ``wc -l`` 一致），含空行与注释。
MAX_FILE_LINES = 800

SCAN_ROOTS = ("src", "tests")


def line_count(path: Path) -> int:
    """物理行数（Unix ``wc -l`` 语义）。"""
    try:
        data = path.read_bytes()
    except OSError as exc:
        print(f"check_code_metrics: cannot read {path}: {exc}", file=sys.stderr)
        raise
    if not data:
        return 0
    n = data.count(b"\n")
    return n if data.endswith(b"\n") else n + 1


def collect_rs_files(repo_root: Path) -> list[Path]:
    files: list[Path] = []
    for rel in SCAN_ROOTS:
        root = repo_root / rel
        if not root.is_dir():
            print(f"check_code_metrics: missing scan root {root}", file=sys.stderr)
            sys.exit(2)
        files.extend(sorted(root.rglob("*.rs")))
    return files


def main() -> int:
    repo_root = Path(__file__).resolve().parent.parent
    violations: list[tuple[Path, int]] = []
    for path in collect_rs_files(repo_root):
        n = line_count(path)
        if n > MAX_FILE_LINES:
            violations.append((path, n))

    if not violations:
        return 0

    print(
        f"check_code_metrics: 单文件不得超过 {MAX_FILE_LINES} 行 "
        f"（扫描目录：{', '.join(SCAN_ROOTS)}）",
        file=sys.stderr,
    )
    for path, n in sorted(violations, key=lambda x: (-x[1], str(x[0]))):
        try:
            rel = path.relative_to(repo_root)
        except ValueError:
            rel = path
        print(f"  {n:4}  {rel}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(main())
