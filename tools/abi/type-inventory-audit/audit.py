#!/usr/bin/env python3
"""Audit ABI boundary type references against Rust and Raven source definitions."""

from __future__ import annotations

import argparse
import csv
import re
from collections import Counter, defaultdict
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable


TOOL_DIR = Path(__file__).resolve().parent
REPO_ROOT = TOOL_DIR.parents[2]

DEFAULT_BOUNDARY_ROOT = REPO_ROOT / "src" / "boundary"
DEFAULT_RUST_DEF_ROOTS = [
    REPO_ROOT / "src",
    REPO_ROOT / "oracle" / "src",
]
DEFAULT_ORACLE_ROOTS = [
    REPO_ROOT / "oracle" / "oracle" / "code",
    REPO_ROOT / "oracle" / "oracle" / "codemp",
]

RUST_SOURCE_EXTENSIONS = {".rs"}
ORACLE_SOURCE_EXTENSIONS = {
    ".c",
    ".cc",
    ".cpp",
    ".cxx",
    ".h",
    ".hpp",
    ".hh",
    ".inl",
}

PRIMITIVE_OR_WRAPPER_TYPES = {
    "Box",
    "CString",
    "Option",
    "PhantomData",
    "Result",
    "String",
    "Vec",
    "bool",
    "c_char",
    "c_double",
    "c_float",
    "c_int",
    "c_long",
    "c_short",
    "c_uchar",
    "c_uint",
    "c_ulong",
    "c_ushort",
    "c_void",
    "const",
    "core",
    "crate",
    "f32",
    "f64",
    "ffi",
    "i128",
    "i16",
    "i32",
    "i64",
    "i8",
    "isize",
    "mut",
    "self",
    "std",
    "str",
    "super",
    "u128",
    "u16",
    "u32",
    "u64",
    "u8",
    "usize",
}

RUST_DEF_RE = re.compile(
    r"\b(?:pub\s+)?(?:type|struct|enum|union|trait)\s+([A-Za-z_][A-Za-z0-9_]*)\b"
)
RUST_PUB_USE_AS_RE = re.compile(r"\bpub\s+use\b[^;]*\bas\s+([A-Za-z_][A-Za-z0-9_]*)\b")
RUST_TYPE_ALIAS_RE = re.compile(r"\btype\s+(Args|Output)\s*=\s*([^;]+);")
RUST_STRUCT_FIELD_RE = re.compile(r"\bpub\s+[A-Za-z_][A-Za-z0-9_]*\s*:\s*([^,]+),")
RUST_IDENT_RE = re.compile(r"\b[A-Za-z_][A-Za-z0-9_]*\b")
FIXME_CREATE_TYPE_RE = re.compile(r"FIXME:\s*create type(?:\s+for)?\s+`?([A-Za-z_][A-Za-z0-9_]*)`?", re.I)


@dataclass
class TypeRecord:
    name: str
    references: list[str] = field(default_factory=list)
    fixmes: list[str] = field(default_factory=list)
    rust_definitions: list[str] = field(default_factory=list)
    oracle_candidates: list[str] = field(default_factory=list)

    @property
    def status(self) -> str:
        if self.rust_definitions and self.fixmes:
            return "defined_rust_type_with_fixme"
        if self.rust_definitions:
            return "defined_rust_type"
        if self.fixmes:
            return "explicit_fixme_missing_rust_type"
        return "missing_rust_type"


def relative(path: Path, line: int | None = None) -> str:
    try:
        value = str(path.relative_to(REPO_ROOT))
    except ValueError:
        value = str(path)
    if line is not None:
        return f"{value}:{line}"
    return value


def iter_files(roots: Iterable[Path], extensions: set[str]) -> Iterable[Path]:
    for root in roots:
        if not root.exists():
            continue
        for path in root.rglob("*"):
            if path.is_file() and path.suffix.lower() in extensions:
                yield path


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="ignore")


def line_number_at(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def trim(value: str, width: int = 180) -> str:
    value = " ".join(value.strip().split())
    if len(value) <= width:
        return value
    return value[: width - 3] + "..."


def collect_rust_definitions(roots: list[Path]) -> dict[str, list[str]]:
    definitions: dict[str, list[str]] = defaultdict(list)
    for path in iter_files(roots, RUST_SOURCE_EXTENSIONS):
        text = read_text(path)
        for pattern in (RUST_DEF_RE, RUST_PUB_USE_AS_RE):
            for match in pattern.finditer(text):
                name = match.group(1)
                definitions[name].append(relative(path, line_number_at(text, match.start())))
    return definitions


def local_type_definitions(text: str) -> set[str]:
    return {match.group(1) for match in RUST_DEF_RE.finditer(text)}


def interesting_type_identifiers(type_expression: str, local_defs: set[str]) -> set[str]:
    identifiers = set()
    for name in RUST_IDENT_RE.findall(type_expression):
        if name in PRIMITIVE_OR_WRAPPER_TYPES:
            continue
        if name in local_defs:
            continue
        if name in {"Args", "Output", "Self"}:
            continue
        identifiers.add(name)
    return identifiers


def collect_boundary_references(boundary_root: Path) -> tuple[dict[str, TypeRecord], Counter[str]]:
    records: dict[str, TypeRecord] = defaultdict(lambda: TypeRecord(""))
    marker_counts: Counter[str] = Counter()

    for path in iter_files([boundary_root], RUST_SOURCE_EXTENSIONS):
        text = read_text(path)
        local_defs = local_type_definitions(text)

        for line_no, line in enumerate(text.splitlines(), start=1):
            fixme = FIXME_CREATE_TYPE_RE.search(line)
            if fixme:
                name = fixme.group(1)
                records[name].name = name
                records[name].fixmes.append(f"{relative(path, line_no)}: {trim(line)}")
                marker_counts["fixme_occurrences"] += 1
                marker_counts[f"fixme_file::{relative(path)}"] += 1

        for pattern in (RUST_TYPE_ALIAS_RE, RUST_STRUCT_FIELD_RE):
            for match in pattern.finditer(text):
                type_expression = match.group(2) if pattern is RUST_TYPE_ALIAS_RE else match.group(1)
                line_no = line_number_at(text, match.start())
                for name in interesting_type_identifiers(type_expression, local_defs):
                    records[name].name = name
                    records[name].references.append(
                        f"{relative(path, line_no)}: {trim(type_expression)}"
                    )

    return records, marker_counts


def oracle_candidate_patterns(name: str) -> list[re.Pattern[str]]:
    escaped = re.escape(name)
    return [
        re.compile(rf"\btypedef\b.*\b{escaped}\b"),
        re.compile(rf"\b(?:struct|enum|class)\s+{escaped}\b"),
        re.compile(rf"^\s*#\s*define\s+{escaped}\b"),
        re.compile(rf"\b{escaped}\b"),
    ]


def find_oracle_candidates(names: Iterable[str], roots: list[Path], limit_per_name: int) -> dict[str, list[str]]:
    remaining = {name: limit_per_name for name in names}
    candidates: dict[str, list[str]] = {name: [] for name in remaining}
    patterns = {name: oracle_candidate_patterns(name) for name in remaining}

    for path in iter_files(roots, ORACLE_SOURCE_EXTENSIONS):
        if not remaining:
            break
        try:
            lines = path.read_text(encoding="utf-8", errors="ignore").splitlines()
        except OSError:
            continue

        for line_no, line in enumerate(lines, start=1):
            for name in list(remaining):
                if remaining[name] <= 0:
                    remaining.pop(name, None)
                    continue
                matched = False
                for pattern in patterns[name]:
                    if pattern.search(line):
                        matched = True
                        break
                if not matched:
                    continue
                candidates[name].append(f"{relative(path, line_no)}: {trim(line)}")
                remaining[name] -= 1
                if remaining[name] <= 0:
                    remaining.pop(name, None)

    return candidates


def build_records(args: argparse.Namespace) -> tuple[list[TypeRecord], Counter[str]]:
    records, marker_counts = collect_boundary_references(args.boundary_root)
    definitions = collect_rust_definitions(args.rust_def_roots)

    for name, locations in definitions.items():
        if name in records:
            records[name].rust_definitions.extend(locations)

    unresolved_names = [
        name
        for name, record in records.items()
        if record.fixmes or not record.rust_definitions
    ]
    oracle_candidates = find_oracle_candidates(
        unresolved_names,
        args.oracle_roots,
        args.oracle_limit,
    )

    for name, candidates in oracle_candidates.items():
        records[name].oracle_candidates.extend(candidates)

    return sorted(records.values(), key=lambda record: record.name.lower()), marker_counts


def write_tsv(records: list[TypeRecord], output_path: Path) -> None:
    with output_path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.writer(handle, delimiter="\t")
        writer.writerow(
            [
                "type_name",
                "status",
                "reference_count",
                "fixme_count",
                "rust_definition_count",
                "references",
                "fixmes",
                "rust_definitions",
                "oracle_candidates",
            ]
        )
        for record in records:
            if record.status == "defined_rust_type" and not record.fixmes:
                continue
            writer.writerow(
                [
                    record.name,
                    record.status,
                    len(record.references),
                    len(record.fixmes),
                    len(record.rust_definitions),
                    "\n".join(record.references),
                    "\n".join(record.fixmes),
                    "\n".join(record.rust_definitions),
                    "\n".join(record.oracle_candidates),
                ]
            )


def write_summary(records: list[TypeRecord], marker_counts: Counter[str], output_path: Path, report_path: Path) -> None:
    unresolved = [record for record in records if not record.rust_definitions]
    explicit_fixmes = [record for record in records if record.fixmes]
    defined_with_fixme = [record for record in records if record.fixmes and record.rust_definitions]
    unresolved_references = [record for record in unresolved if record.references]
    fixme_files = [key for key in marker_counts if key.startswith("fixme_file::")]

    lines = [
        "# ABI Type Inventory Audit",
        "",
        "This report is generated by `tools/abi/type-inventory-audit/audit.py`.",
        "",
        "## Outputs",
        "",
        f"- TSV report: `{relative(report_path)}`",
        f"- Markdown summary: `{relative(output_path)}`",
        "",
        "## Counts",
        "",
        f"- Explicit `FIXME: create type` occurrences: {marker_counts['fixme_occurrences']}",
        f"- Files with explicit type FIXME markers: {len(fixme_files)}",
        f"- Unique names with explicit type FIXME markers: {len(explicit_fixmes)}",
        f"- Unique referenced names missing Rust definitions: {len(unresolved_references)}",
        f"- Unique unresolved names total: {len(unresolved)}",
        f"- Explicit FIXME names that now have Rust definitions: {len(defined_with_fixme)}",
        "",
        "## Unresolved Names",
        "",
    ]

    if unresolved:
        for record in unresolved:
            candidate_note = "oracle candidates found" if record.oracle_candidates else "no oracle candidates found"
            lines.append(
                f"- `{record.name}`: {len(record.references)} references, "
                f"{len(record.fixmes)} FIXME markers, {candidate_note}"
            )
    else:
        lines.append("- None.")

    lines.extend(
        [
            "",
            "## Notes",
            "",
            "- The audit is static because `src/boundary` is not currently declared from `src/lib.rs`.",
            "- Oracle candidates are text matches in Raven source, not proof of an ABI-safe Rust port.",
            "- Keep Raven comments and source locations in boundary files when resolving each type.",
            "",
        ]
    )

    output_path.write_text("\n".join(lines), encoding="utf-8")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--boundary-root", type=Path, default=DEFAULT_BOUNDARY_ROOT)
    parser.add_argument("--rust-def-root", dest="rust_def_roots", type=Path, action="append")
    parser.add_argument("--oracle-root", dest="oracle_roots", type=Path, action="append")
    parser.add_argument("--oracle-limit", type=int, default=5)
    parser.add_argument("--output-dir", type=Path, default=TOOL_DIR)
    args = parser.parse_args()

    if args.rust_def_roots is None:
        args.rust_def_roots = DEFAULT_RUST_DEF_ROOTS
    if args.oracle_roots is None:
        args.oracle_roots = DEFAULT_ORACLE_ROOTS
    return args


def main() -> int:
    args = parse_args()
    args.output_dir.mkdir(parents=True, exist_ok=True)

    records, marker_counts = build_records(args)
    report_path = args.output_dir / "type-inventory-report.tsv"
    summary_path = args.output_dir / "type-inventory-summary.md"

    write_tsv(records, report_path)
    write_summary(records, marker_counts, summary_path, report_path)

    unresolved_total = sum(1 for record in records if not record.rust_definitions)
    explicit_fixme_total = sum(1 for record in records if record.fixmes)
    print(f"Wrote {relative(report_path)}")
    print(f"Wrote {relative(summary_path)}")
    print(f"Explicit FIXME type names: {explicit_fixme_total}")
    print(f"Unresolved referenced type names: {unresolved_total}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
