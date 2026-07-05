#!/usr/bin/env python3
"""Bulk mechanical integration fixes, driven by cargo's JSON spans.

Modes (combine freely; --apply actually writes, default is dry-run):

  bulkfix.py --rename-entities g_weapon.rs [...]
      Scoped textual rename of the GameWorld arena access spelling
      `...world).entities` / `world.entities` -> `.g_entities`.
      Regex is anchored on the `world` receiver so unrelated `entities`
      identifiers (locals, params) are never touched.

  bulkfix.py --cast g_weapon.rs [...]
      Span-driven casts for E0308 integer-width / int-enum mismatches:
      runs `cargo check --message-format=json`, collects primary spans in
      the target files whose message is `expected \x60T\x60, found \x60U\x60` where
      T is a primitive int and U is a primitive int or a #[repr(int)] enum
      (`*_t` name), and rewrites the exact span to `(<expr>) as T`.
      Byte-precise (rustc's own span extents), applied bottom-up per file.

Always verify with cargo afterwards; the tool never touches anything
without a matching rustc diagnostic (--cast) or the anchored pattern
(--rename-entities).
"""

import json
import re
import subprocess
import sys
from pathlib import Path

WT = Path("/Users/milohehmsoth/Developer/Milo/jka-rust/.claude/worktrees/agent-a43cc53200d2fdf54")
SRC = WT / "crates/mp/game/src"

# `(*ctx.world).entities`, `world).entities`, `world.entities` — receiver-anchored.
ENTITIES_PAT = re.compile(r"(world\)?)\.entities\b")

PRIM_INTS = {"i8", "u8", "i16", "u16", "i32", "u32", "i64", "u64", "isize", "usize", "c_int"}
EXPECTED_FOUND = re.compile(r"expected `([\w:]+)`, found `([\w:]+)`")


def resolve(names):
    out = []
    for n in names:
        p = Path(n)
        if not p.is_absolute():
            p = SRC / n
        if not p.exists():
            sys.exit(f"no such file: {p}")
        out.append(p)
    return out


def rename_entities(paths, apply):
    for p in paths:
        s = p.read_text()
        hits = ENTITIES_PAT.findall(s)
        if apply and hits:
            p.write_text(ENTITIES_PAT.sub(r"\1.g_entities", s))
        print(f"rename-entities {p.name}: {len(hits)} site(s){'' if apply else ' (dry-run)'}")


def cast_targets(target_paths):
    """cargo check json -> {abs_file: [(byte_start, byte_end, cast_to)]}"""
    targets = {str(p) for p in target_paths}
    res = subprocess.run(
        ["cargo", "check", "-p", "mp_game", "--message-format=json"],
        cwd=WT, capture_output=True, text=True,
    )
    edits = {}
    for line in res.stdout.splitlines():
        try:
            m = json.loads(line)
        except json.JSONDecodeError:
            continue
        if m.get("reason") != "compiler-message":
            continue
        msg = m["message"]
        code = (msg.get("code") or {}).get("code")
        if code != "E0308":
            continue
        for sp in msg.get("spans", []):
            if not sp.get("is_primary"):
                continue
            # rustc puts the `expected T, found U` text in the primary span's
            # label, not the top-level message (which is just "mismatched types").
            mm = EXPECTED_FOUND.search(sp.get("label") or "") or EXPECTED_FOUND.search(msg.get("message", ""))
            if not mm:
                continue
            expected, found = mm.group(1).split("::")[-1], mm.group(2).split("::")[-1]
            # only primitive-int targets; sources: primitive ints or #[repr(int)] enums
            if expected not in PRIM_INTS:
                continue
            if found not in PRIM_INTS and not found.endswith("_t"):
                continue
            f = str(WT / sp["file_name"])
            if f in targets:
                edits.setdefault(f, set()).add((sp["byte_start"], sp["byte_end"], expected))
    return {f: sorted(v) for f, v in edits.items()}


def cast_fixes(paths, apply):
    edits = cast_targets(paths)
    if not edits:
        print("cast: no matching E0308 spans in target files")
        return
    for f, spans in edits.items():
        data = Path(f).read_bytes()
        n = 0
        for start, end, to in sorted(spans, reverse=True):
            expr = data[start:end]
            if b"\n" in expr:  # multi-line span: skip, needs eyes
                print(f"  SKIP multi-line span at byte {start} in {Path(f).name}")
                continue
            data = data[:start] + b"(" + expr + b") as " + to.encode() + data[end:]
            n += 1
        if apply:
            Path(f).write_bytes(data)
        print(f"cast {Path(f).name}: {n} span(s) rewritten{'' if apply else ' (dry-run)'}")


def main():
    args = sys.argv[1:]
    apply = "--apply" in args
    args = [a for a in args if a != "--apply"]
    mode = args[0] if args else ""
    files = resolve(args[1:])
    if mode == "--rename-entities":
        rename_entities(files, apply)
    elif mode == "--cast":
        cast_fixes(files, apply)
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
