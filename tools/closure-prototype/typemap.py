#!/usr/bin/env python3
"""PROTOTYPE — throwaway. Type rosetta for the engine port: every ported Rust
type, keyed by its Raven name, with its Rust path/crate/name — harvested from
the house-style doc comments (`Raven \`X\`` + `Source:`/`Type definition
source:` cites) that every ported item carries. Handed to porting agents as a
packet reference so type imports resolve mechanically and nothing gets
mis-stubbed.

Output: out/engine/type-rosetta.tsv  (raven_name, rust_name, kind, crate,
path, oracle_cite) sorted by raven_name.

Usage: .venv/bin/python typemap.py [--crates DIR] [--out DIR]
"""
import argparse
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]

# `const`/`static` harvested alongside the type kinds: engine packets resolve
# array-size / count identifiers (MAX_*, and any screaming-snake const) against
# the rosetta so porters import them instead of re-defining a magic number (the
# #1 jampgame pass-3 porter failure class). Identity-named fallback — no
# doc-comment gate — so non-doc-commented consts (the census's 38
# "ported-elsewhere": ACTION_*, S_COLOR_WHITE, …) resolve too.
ITEM_RE = re.compile(
    r'^\s*pub\s+(struct|enum|union|type|trait|const|static)\s+(?:mut\s+)?'
    r'([A-Za-z_][A-Za-z0-9_]*)')
# ENUM VARIANTS of ported enums (kind "variant", rust = `EnumName::VARIANT`):
# Raven spells its C enum constants bare (`FS_SEEK_SET`), so the packet CONSTS
# resolution needs the bare name → qualified Rust form. Screaming-snake only —
# CamelCase variants can't be named by a C source slice and would only add
# collision noise.
VARIANT_RE = re.compile(r"^\s*([A-Z][A-Z0-9_]{2,})\s*(?:=[^,]*)?,?\s*(?://.*)?$")
# Non-pub (or pub(crate)) const/static fallback — screaming-snake only. Several
# census "ported-elsewhere" names are PRIVATE consts (mp_game's ACTION_* flags,
# botlib.h:66-89); a rosetta row still resolves the name for porters (the row's
# path says where the value lives — visibility/tier is the porter's/finisher's
# call, better than an unresolved escalation). Gated to [A-Z_]+ so local helper
# consts inside fn bodies don't flood the rosetta.
PRIV_CONST_RE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?(const|static)\s+(?:mut\s+)?"
    r"([A-Z][A-Z0-9_]{2,})\s*:")
RAVEN_RE = re.compile(r'Raven(?:\'s)?\s+`([A-Za-z_][A-Za-z0-9_]*)`')
CITE_RE = re.compile(r'(?:Source|source):\s*`?(oracle/[^\s`]+)`?')


def crate_of(path: Path, cache={}):
    d = path.parent
    while d != REPO and d.parent != d:
        c = d / "Cargo.toml"
        if c.exists():
            if c not in cache:
                m = re.search(r'^name\s*=\s*"([^"]+)"', c.read_text(), re.M)
                cache[c] = m.group(1) if m else d.name
            return cache[c]
        d = d.parent
    return "?"


def scan(crates_dir):
    rows = []
    for rs in sorted(crates_dir.rglob("*.rs")):
        try:
            lines = rs.read_text().splitlines()
        except UnicodeDecodeError:
            continue
        # file-level //! header (one-type-per-file convention) as fallback
        header = "\n".join(l for l in lines[:30] if l.strip().startswith("//!"))
        f_raven, f_cite = RAVEN_RE.search(header), CITE_RE.search(header)
        doc = []  # trailing comment block (/// or //) above the current line
        # enum-body tracker: (enum rust name, opening-brace seen, brace depth)
        in_enum, enum_seen, enum_depth = None, False, 0
        for ln in lines:
            s = ln.strip()
            # ---- inside a pub enum body: harvest screaming-snake variants
            if in_enum is not None and not s.startswith("//"):
                if enum_seen and enum_depth > 0:
                    vm = VARIANT_RE.match(ln)
                    if vm:
                        rows.append({
                            "raven": vm.group(1),
                            "rust": f"{in_enum}::{vm.group(1)}",
                            "kind": "variant",
                            "crate": crate_of(rs),
                            "path": str(rs.relative_to(REPO)),
                            "cite": "",
                        })
                enum_depth += ln.count("{") - ln.count("}")
                if enum_depth > 0:
                    enum_seen = True
                elif enum_seen:
                    in_enum = None
            if s.startswith("///") or s.startswith("//") or s.startswith("#["):
                doc.append(s)
                continue
            m = ITEM_RE.match(ln)
            if m:
                block = "\n".join(doc)
                raven = RAVEN_RE.search(block) or f_raven
                cite = CITE_RE.search(block) or f_cite
                kind, rust = m.group(1), m.group(2)
                raven_name = raven.group(1) if raven else rust
                # Register every Raven spelling an engine signature could use for
                # this type. Raven's `typedef struct X_s {...} X_t;` gives two
                # names; a signature may spell either the typedef (`X_t`) or the
                # struct tag (`struct X_s *`). The house port declares ONE Rust
                # item plus a `pub type` counterpart, so harvest both sides:
                #  - the doc's `Raven `...`` name (primary, existing behavior),
                #  - the Rust item's own name (usually the `_t` typedef), and
                #  - a bare `pub type X = ...` alias's own name.
                # Extra rows are harmless: load_rosetta() dedups by Raven name,
                # preferring the identity-named (rust == raven) row.
                names = []
                if raven or cite:
                    names.append(raven_name)
                if kind in ("struct", "enum", "union") and rust != raven_name:
                    names.append(rust)
                if kind == "type":
                    names.append(rust)
                # A const/static's Rust name IS the import spelling and is
                # usually identical to the Raven `#define`/enum name
                # (MAX_QPATH etc.); ALWAYS register it (identity fallback, no
                # doc-comment gate — the census's "ported-elsewhere" fix).
                if kind in ("const", "static") and rust not in names:
                    names.append(rust)
                # a pub enum: harvest its screaming-snake variants (kind
                # "variant") — start the body tracker on this very line.
                if kind == "enum":
                    in_enum = rust
                    enum_depth = ln.count("{") - ln.count("}")
                    enum_seen = enum_depth > 0
                for nm in dict.fromkeys(names):
                    rows.append({
                        "raven": nm,
                        "rust": rust,
                        "kind": kind,
                        "crate": crate_of(rs),
                        "path": str(rs.relative_to(REPO)),
                        "cite": cite.group(1) if cite else "",
                    })
            else:
                # non-pub const/static fallback (identity row only) — picks up
                # the doc-block Source cite when present.
                pm = PRIV_CONST_RE.match(ln)
                if pm:
                    cite = CITE_RE.search("\n".join(doc)) or f_cite
                    rows.append({
                        "raven": pm.group(2),
                        "rust": pm.group(2),
                        "kind": pm.group(1),
                        "crate": crate_of(rs),
                        "path": str(rs.relative_to(REPO)),
                        "cite": cite.group(1) if cite else "",
                    })
            if s:
                doc = []
    return rows


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--crates", default=str(REPO / "crates"))
    ap.add_argument("--out", default=str(Path(__file__).resolve().parent / "out" / "engine"))
    args = ap.parse_args()
    rows = scan(Path(args.crates))
    # On raven-name collisions prefer the identity-named port (rust == raven) —
    # doc-comment mentions of OTHER types (e.g. `Raven \`qboolean\`` inside
    # gtimer_t's block) otherwise shadow the real port. Then prefer cited rows.
    rows.sort(key=lambda r: (r["raven"].lower(), r["rust"] != r["raven"],
                             r["cite"] == "", r["path"]))
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    tsv = ["raven_name\trust_name\tkind\tcrate\tpath\toracle_cite"]
    for r in rows:
        tsv.append("\t".join([r["raven"], r["rust"], r["kind"], r["crate"],
                              r["path"], r["cite"]]))
    (out / "type-rosetta.tsv").write_text("\n".join(tsv) + "\n")
    names = {r["raven"] for r in rows}
    print(f"[typemap] {len(rows)} items, {len(names)} distinct raven names "
          f"-> {out / 'type-rosetta.tsv'}")


if __name__ == "__main__":
    main()
