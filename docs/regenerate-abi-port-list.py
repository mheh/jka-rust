#!/usr/bin/env python3
"""Regenerate the ABI port worklist from the manifest and current Rust TODOs.

This script intentionally does not rewrite docs/abi-port-manifest.tsv. It reads
the manifest as the stable assignment inventory, scans src/abi for
`TODO: Port args` and `TODO: Port output`, then prints the remaining rows with
the Rust line numbers needed for worker prompts.
"""

from __future__ import annotations

import argparse
import csv
from collections import Counter, defaultdict
from pathlib import Path


TODO_MARKERS = ("TODO: Port args", "TODO: Port output")


def repo_root() -> Path:
    return Path(__file__).resolve().parents[1]


def todo_lines(root: Path) -> dict[str, list[tuple[int, str]]]:
    todos: dict[str, list[tuple[int, str]]] = defaultdict(list)
    for path in sorted((root / "src" / "abi").rglob("*.rs")):
        rel = path.relative_to(root).as_posix()
        try:
            lines = path.read_text(encoding="utf-8").splitlines()
        except UnicodeDecodeError:
            lines = path.read_text().splitlines()
        for line_no, line in enumerate(lines, start=1):
            if any(marker in line for marker in TODO_MARKERS):
                todos[rel].append((line_no, line.strip()))
    return todos


def manifest_rows(root: Path) -> list[dict[str, str]]:
    manifest = root / "docs" / "abi-port-manifest.tsv"
    with manifest.open(newline="", encoding="utf-8") as handle:
        return list(csv.DictReader(handle, delimiter="\t"))


def print_summary(rows: list[dict[str, str]], todos: dict[str, list[tuple[int, str]]]) -> None:
    marker_counts = Counter()
    file_counts = Counter()
    for row in rows:
        path = row["boundary_file"]
        if path not in todos:
            continue
        key = f"{row['surface']} / {row['file_kind']} / {row['transport']}"
        marker_counts[key] += len(todos[path])
        file_counts[key] += 1

    print("# Remaining ABI port TODOs")
    print()
    print("| Surface | Files | TODO markers |")
    print("| --- | ---: | ---: |")
    for key in sorted(marker_counts):
        print(f"| {key} | {file_counts[key]} | {marker_counts[key]} |")
    print()


def row_scope(row: dict[str, str], todos: list[tuple[int, str]]) -> str:
    todo_refs = ", ".join(f"{line_no}:{text}" for line_no, text in todos)
    return (
        f"{row['boundary_file']} todos={todo_refs}\t"
        f"surface={row['surface']}\t"
        f"file_kind={row['file_kind']}\t"
        f"direction={row['direction']}\t"
        f"transport={row['transport']}\t"
        f"call_name={row['call_name']}\t"
        f"assignment_status={row['assignment_status']}\t"
        f"enum_source={row['enum_source']}\t"
        f"arg_output_source_hints={row['arg_output_source_hints']}\t"
        f"worker_scope={row['worker_scope']}\t"
        f"notes={row['notes']}"
    )


def print_rows(
    rows: list[dict[str, str]],
    todos: dict[str, list[tuple[int, str]]],
    surface_filter: str | None,
    limit: int | None,
) -> None:
    printed = 0
    print("# Worker scope rows")
    for row in rows:
        path = row["boundary_file"]
        if path not in todos:
            continue
        if surface_filter and row["surface"] != surface_filter:
            continue
        print(row_scope(row, todos[path]))
        printed += 1
        if limit is not None and printed >= limit:
            break


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Print remaining ABI port work from docs/abi-port-manifest.tsv."
    )
    parser.add_argument(
        "--surface",
        help="Only print rows for an exact manifest surface, e.g. 'SP cgame'.",
    )
    parser.add_argument(
        "--limit",
        type=int,
        help="Limit printed worker rows after summary.",
    )
    parser.add_argument(
        "--no-summary",
        action="store_true",
        help="Only print worker scope rows.",
    )
    args = parser.parse_args()

    root = repo_root()
    rows = manifest_rows(root)
    todos = todo_lines(root)

    if not args.no_summary:
        print_summary(rows, todos)
    print_rows(rows, todos, args.surface, args.limit)


if __name__ == "__main__":
    main()
