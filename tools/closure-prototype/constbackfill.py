#!/usr/bin/env python3
"""PROTOTYPE — throwaway. CONST-BACKFILL CENSUS: classify every const name the
engine packets could NOT resolve against the const rosetta
(`manifest.json` → consts.unresolved_all, produced by enginepackets.py).

Each name gets exactly one class (first match wins, in this order):

  enum-variant     already a variant of a ported Rust enum (or an associated
                   const on a ported type) in crates/ — the packet resolution /
                   rosetta harvester needs a fix, NOT a port. Evidence: Rust path.
  ported-elsewhere exists as a Rust `const`/`static` in crates/ but wasn't
                   harvested by typemap.py (non-doc-commented, inside an impl,
                   private, …) — harvester fix. Evidence: Rust path.
  oracle-const     a real object-like `#define` / C enum constant in
                   oracle/codemp with NO Rust port yet — THE BACKFILL WORK LIST,
                   sorted by owning oracle file so one header = one backfill
                   unit. Evidence: oracle cite (path:line) + the #define text.
  macro-or-other   function-like macros, conditional-compilation names, goto
                   labels / locals that matched the screaming-snake regex, and
                   names with no definition anywhere. Evidence: why.

Census only — ports nothing, edits nothing outside out/engine/.

Output: out/engine/const-backfill.json
Usage:  .venv/bin/python constbackfill.py
"""
import json
import re
from collections import Counter, defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
MANIFEST = HERE / "out" / "engine" / "packets" / "manifest.json"
OUT = HERE / "out" / "engine" / "const-backfill.json"

CRATES = REPO / "crates"
ORACLE = REPO / "oracle" / "codemp"

# ---------------------------------------------------------------- Rust index
RS_CONST = re.compile(r"\bconst\s+([A-Z][A-Z0-9_]{2,})\s*:")
RS_STATIC = re.compile(r"\bstatic\s+(?:mut\s+)?([A-Z][A-Z0-9_]{2,})\s*:")
RS_ENUM_OPEN = re.compile(r"\benum\s+([A-Za-z_]\w*)")
# a variant line inside an enum body: `NAME,` / `NAME = expr,` / `NAME(…)`
RS_VARIANT = re.compile(r"^\s*([A-Z][A-Z0-9_]{2,})\s*(?:=[^,]*)?,?\s*(?://.*)?$")


def index_rust():
    """Scan crates/*.rs once. Returns (variants, consts): name -> first
    'path:line' evidence. Variant detection is a light brace-tracked state
    machine over enum bodies (good enough for house-style one-item-per-file)."""
    variants, consts = {}, {}
    for rs in sorted(CRATES.rglob("*.rs")):
        try:
            lines = rs.read_text().splitlines()
        except UnicodeDecodeError:
            continue
        rel = str(rs.relative_to(REPO))
        in_enum, depth = False, 0
        for i, ln in enumerate(lines, 1):
            m = RS_CONST.search(ln)
            if m:
                consts.setdefault(m.group(1), f"{rel}:{i}")
            m = RS_STATIC.search(ln)
            if m:
                consts.setdefault(m.group(1), f"{rel}:{i}")
            if not in_enum and RS_ENUM_OPEN.search(ln) \
                    and not ln.lstrip().startswith("//"):
                in_enum, depth = True, 0
            if in_enum:
                if depth > 0:
                    m = RS_VARIANT.match(ln)
                    if m and m.group(1) not in ("_",):
                        variants.setdefault(m.group(1), f"{rel}:{i}")
                depth += ln.count("{") - ln.count("}")
                if depth <= 0 and "{" in "".join(lines[max(0, i - 10):i]):
                    if "}" in ln and depth <= 0:
                        in_enum = False
        # (an unclosed tracker just runs to EOF — harmless for a census)
    return variants, consts


# -------------------------------------------------------------- oracle index
# object-like define: `#define NAME value` (NO `(` immediately after the name);
# function-like define: `#define NAME(args)`.
ORC_DEF_OBJ = re.compile(r"^\s*#\s*define\s+([A-Z][A-Z0-9_]{2,})(?![A-Za-z0-9_(])(.*)$")
ORC_DEF_FN = re.compile(r"^\s*#\s*define\s+([A-Z][A-Z0-9_]{2,})\(")
ORC_ENUM_CONST = re.compile(r"^\s*([A-Z][A-Z0-9_]{2,})\s*(?:=[^,}]*)?\s*[,}]")
ORC_IFDEF = re.compile(r"^\s*#\s*(?:ifdef|ifndef|elif|if)\b.*?\b([A-Z][A-Z0-9_]{2,})\b")


def index_oracle():
    """Scan oracle/codemp once. Returns (obj_defines, fn_defines, enum_consts,
    cond_names): name -> ('path:line', text) / set. Headers AND .cpp/.c are
    indexed (a .cpp object-like #define is still a real constant; it just
    shards with that file)."""
    obj, fnm, enums, conds = {}, {}, {}, set()
    for src in sorted(ORACLE.rglob("*")):
        if src.suffix not in (".h", ".cpp", ".c"):
            continue
        try:
            lines = src.read_text(errors="replace").splitlines()
        except OSError:
            continue
        rel = str(src.relative_to(REPO))
        in_enum = False
        for i, ln in enumerate(lines, 1):
            m = ORC_DEF_FN.match(ln)
            if m:
                fnm.setdefault(m.group(1), (f"{rel}:{i}", ln.strip()[:120]))
                continue
            m = ORC_DEF_OBJ.match(ln)
            if m:
                obj.setdefault(m.group(1), (f"{rel}:{i}", ln.strip()[:120]))
                continue
            m = ORC_IFDEF.match(ln)
            if m:
                conds.add(m.group(1))
            s = ln.strip()
            if re.match(r"^(typedef\s+)?enum\b", s) and ";" not in s:
                in_enum = True
            if in_enum:
                m = ORC_ENUM_CONST.match(ln)
                if m and not re.match(r"^(typedef|enum)$", m.group(1)):
                    enums.setdefault(m.group(1), (f"{rel}:{i}", s[:120]))
                if "}" in ln:
                    in_enum = False
    return obj, fnm, enums, conds


def main():
    man = json.loads(MANIFEST.read_text())
    names = man["consts"]["unresolved_all"]
    print(f"[constbackfill] {len(names)} unresolved consts from manifest")

    variants, rust_consts = index_rust()
    print(f"[constbackfill] rust index: {len(variants)} enum variants, "
          f"{len(rust_consts)} const/static defs")
    obj, fnm, enums, conds = index_oracle()
    print(f"[constbackfill] oracle index: {len(obj)} object defines, "
          f"{len(fnm)} fn-like defines, {len(enums)} enum consts")

    entries = []
    for n in names:
        if n in variants:
            entries.append({"name": n, "class": "enum-variant",
                            "evidence": variants[n]})
        elif n in rust_consts:
            entries.append({"name": n, "class": "ported-elsewhere",
                            "evidence": rust_consts[n]})
        elif n in obj:
            cite, text = obj[n]
            entries.append({"name": n, "class": "oracle-const",
                            "evidence": f"{cite} — {text}"})
        elif n in enums:
            cite, text = enums[n]
            entries.append({"name": n, "class": "oracle-const",
                            "evidence": f"{cite} — enum constant: {text}"})
        elif n in fnm:
            cite, text = fnm[n]
            entries.append({"name": n, "class": "macro-or-other",
                            "evidence": f"function-like macro {cite} — {text}"})
        elif n in conds:
            entries.append({"name": n, "class": "macro-or-other",
                            "evidence": "conditional-compilation name "
                                        "(#ifdef/#if guard operand only)"})
        else:
            entries.append({"name": n, "class": "macro-or-other",
                            "evidence": "no definition found in crates/ or "
                                        "oracle/codemp (local / label / regex "
                                        "false positive)"})

    counts = Counter(e["class"] for e in entries)
    # backfill work list: oracle-const grouped by owning oracle file, sorted by
    # per-file count desc — one file = one backfill unit for sharding.
    by_header = defaultdict(list)
    for e in entries:
        if e["class"] == "oracle-const":
            hdr = e["evidence"].split(":", 1)[0]
            by_header[hdr].append(e["name"])
    backfill = [{"header": h, "count": len(v), "consts": sorted(v)}
                for h, v in sorted(by_header.items(),
                                   key=lambda kv: (-len(kv[1]), kv[0]))]

    out = {
        "generated_by": "tools/closure-prototype/constbackfill.py",
        "input": "out/engine/packets/manifest.json consts.unresolved_all",
        "total_unresolved": len(names),
        "class_counts": dict(counts.most_common()),
        "backfill_units": len(backfill),
        "backfill_by_header": backfill,
        "entries": sorted(entries, key=lambda e: (e["class"], e["name"])),
    }
    OUT.write_text(json.dumps(out, indent=1))
    print(f"[constbackfill] classes: {dict(counts.most_common())}")
    print(f"[constbackfill] backfill units (headers): {len(backfill)}; top:")
    for b in backfill[:10]:
        print(f"    {b['count']:4d}  {b['header']}")
    print(f"[constbackfill] wrote {OUT}")


if __name__ == "__main__":
    main()
