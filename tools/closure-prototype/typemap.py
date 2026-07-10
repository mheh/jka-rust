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

ITEM_RE = re.compile(
    r'^\s*pub\s+(struct|enum|union|type|trait)\s+([A-Za-z_][A-Za-z0-9_]*)')
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
        for ln in lines:
            s = ln.strip()
            if s.startswith("///") or s.startswith("//") or s.startswith("#["):
                doc.append(s)
                continue
            m = ITEM_RE.match(ln)
            if m:
                block = "\n".join(doc)
                raven = RAVEN_RE.search(block) or f_raven
                cite = CITE_RE.search(block) or f_cite
                if raven or cite:
                    rows.append({
                        "raven": raven.group(1) if raven else m.group(2),
                        "rust": m.group(2),
                        "kind": m.group(1),
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
