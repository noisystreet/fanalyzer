#!/usr/bin/env python3
"""从 Rust 类型与 Clap 定义导出 JSON Schema，并可选校验仓库内 schemas/ 是否同步。"""

from __future__ import annotations

import argparse
import filecmp
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def export_schemas(output_dir: Path) -> None:
    root = repo_root()
    cmd = [
        "cargo",
        "run",
        "--quiet",
        "--",
        "schema",
        "export",
        "--output-dir",
        str(output_dir),
    ]
    subprocess.run(cmd, cwd=root, check=True)


def collect_json_files(directory: Path) -> set[Path]:
    return {p.relative_to(directory) for p in directory.rglob("*.json")}


def compare_trees(expected: Path, actual: Path) -> list[str]:
    errors: list[str] = []
    expected_files = collect_json_files(expected)
    actual_files = collect_json_files(actual)

    missing = sorted(expected_files - actual_files)
    extra = sorted(actual_files - expected_files)
    for rel in missing:
        errors.append(f"missing generated file: {rel}")
    for rel in extra:
        errors.append(f"unexpected generated file: {rel}")

    for rel in sorted(expected_files & actual_files):
        a = expected / rel
        b = actual / rel
        if not filecmp.cmp(a, b, shallow=False):
            errors.append(f"content differs: {rel}")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=repo_root() / "schemas",
        help="schema 输出目录（默认：仓库 schemas/）",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="导出到临时目录并与 --output-dir 比对，不一致则退出 1",
    )
    args = parser.parse_args()

    if args.check:
        with tempfile.TemporaryDirectory(prefix="fanalyzer-schemas-") as tmp:
            tmp_path = Path(tmp)
            export_schemas(tmp_path)
            errors = compare_trees(args.output_dir, tmp_path)
            if errors:
                print(
                    "generate_schemas: schemas/ 与代码不同步，请运行 "
                    "`python3 scripts/generate_schemas.py` 后提交：",
                    file=sys.stderr,
                )
                for err in errors:
                    print(f"  {err}", file=sys.stderr)
                return 1
        return 0

    args.output_dir.mkdir(parents=True, exist_ok=True)
    export_schemas(args.output_dir)
    return 0


if __name__ == "__main__":
    sys.exit(main())
