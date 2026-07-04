#!/usr/bin/env python3
"""PROTOTYPE — throwaway. Pass-3 symbol PREFLIGHT: file-level resolvability sweep.

During a pass-3 porter run, freshly ported bodies name symbols (enum consts,
tables, helper fns, types) that ALREADY EXIST somewhere in the workspace
(`crates/mp/{game,bg,qshared}`) but are not reachable from the game crate's
bare-name scope — no prelude glob, a private item, or an un-re-exported
cross-crate name. Today each such name spawns one runtime "symbol fixer" agent
(20+ agents for the LS_*/Q_*/MAT_* families that all live in two enum files).

This tool closes those gaps mechanically and AGGREGATED AT FILE LEVEL — one
re-export action per DEFINING FILE, not one per symbol — so a deterministic
preflight replaces the swarm of runtime fixers.

Pipeline (reuses symsweep, no clang re-parse of the oracle):
  1. Candidate symbol set = every out-of-file identifier the pass-3 packets
     reference. This is exactly what `symsweep.py` already extracted from one
     libclang unity parse of the jampgame oracle; we consume its output
     (`out/pass3/missing-symbols.json`: name / kind / oracle source / sites)
     rather than re-parsing.
  2. Resolvability is re-tested LIVE against the target worktree's Rust crates
     (the porter is actively writing, so the cached classification is stale) by
     reusing symsweep's grep-level scanner + `classify()`. False "unresolved" is
     acceptable (apply is idempotent); false "resolved" is what we avoid.
  3. Every unresolved-but-existing symbol is grouped by its DEFINING FILE; each
     file gets ONE action: a module glob (`pub use path::*;`) for a family of
     module-level consts/fns, an enum glob (`pub use path::{E, E::*};`) for enum
     variants, a single-item re-export for a lone table/fn, or `make pub +
     re-export` when the item exists only privately. Actions match the house
     conventions in `crates/mp/game/src/prelude.rs`.
  4. Symbols that exist NOWHERE become the residue list (grouped by the oracle
     header that declares them) — the true remaining agent work.

Usage:
  .venv/bin/python preflight.py [--worktree DIR] [--in DIR] [--out DIR] [--apply]

  default        dry-run: print the plan, write out/pass3/preflight-plan.json
  --worktree DIR operate on DIR's crates/mp/* (default: the main repo).
  --apply        edit the target prelude/mod files (idempotent: skip lines
                 already present, never duplicate). NOT run by default.

Companion to symsweep.py (the const/enum/global manifest) and packets3.py
(the pass-3 packet generator).
"""
import argparse
import json
import re
import sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import closure as C
import symsweep as SS

CRATE_PKG = {"game": "crate", "bg": "mp_bg", "qshared": "mp_qshared"}
# canonical-source preference when a name is defined in several crates: a
# library crate (qshared/bg) is a better re-export source than the game crate.
CRATE_RANK = {"qshared": 0, "bg": 1, "game": 2}

CRATE_RE = re.compile(r"crates/mp/(game|bg|qshared)/src/(.+)\.rs$")

# preflight-owned block appended to prelude.rs on --apply.
PRELUDE_MARKER = "// preflight.py: file-level symbol re-exports (aggregate re-export plan)"


# ------------------------------------------------------------ path -> module

def crate_of(rel: str):
    m = CRATE_RE.search(Path(rel).as_posix())
    return m.group(1) if m else None


def module_path_of(rel: str):
    """repo-rel Rust path -> `(crate, module_path)` where module_path is the
    prelude-legal `use` prefix (`crate::foo::bar` / `mp_bg::public::baz`).
    Mirrors the one-type-per-file layout: file path == module path."""
    m = CRATE_RE.search(Path(rel).as_posix())
    if not m:
        return None, None
    crate, inner = m.group(1), m.group(2)
    # mod.rs / lib.rs collapse to their parent module.
    if inner in ("mod", "lib"):
        inner = ""
    elif inner.endswith("/mod") or inner.endswith("/lib"):
        inner = inner.rsplit("/", 1)[0]
    base = CRATE_PKG[crate]
    mod = base + ("::" + inner.replace("/", "::") if inner else "")
    return crate, mod


# ------------------------------------------------------------ decl selection

def choose_decl(decls_for_name):
    """Pick the canonical defining decl and whether any `pub` decl exists.
    Prefer a `pub` decl, then a library crate, then a stable path sort."""
    pubs = [d for d in decls_for_name if d["pub"]]
    pool = pubs if pubs else decls_for_name
    pool = sorted(pool, key=lambda d: (CRATE_RANK.get(crate_of(d["path"]), 9),
                                       d["path"]))
    return pool[0], bool(pubs)


# ------------------------------------------------------------ action shaping

def action_module_items(fa):
    """All module-level (const/static/fn) names the file contributes, whether
    already `pub` or made `pub` by this plan — a single glob covers them all."""
    items = set(fa["reexport_module_items"])
    items |= {d["name"] for d in fa["makepub_items"]
              if d["kind"] != "enum-variant"}
    return sorted(items)


def action_enums(fa):
    """All enums the file contributes variants for (pub or made-pub)."""
    enums = set(fa["reexport_enums"])
    enums |= {d["enum_name"] for d in fa["makepub_items"]
              if d["kind"] == "enum-variant" and d.get("enum_name")}
    return sorted(enums)


def action_use_lines(fa):
    """Derive the `pub use` line(s) for one file action, house-style.
    Aggregated: many module-level items collapse to one module glob; each enum
    collapses to one `{E, E::*}` glob."""
    lines = []
    mod = fa["module_path"]
    items = action_module_items(fa)
    if len(items) >= 2:
        lines.append(f"pub use {mod}::*;")
    elif len(items) == 1:
        lines.append(f"pub use {mod}::{{{items[0]}}};")
    for enum in action_enums(fa):
        lines.append(f"pub use {mod}::{{{enum}, {enum}::*}};")
    return lines


def action_kind(fa):
    prefix = "make-pub + " if fa["makepub_items"] else ""
    items, enums = action_module_items(fa), action_enums(fa)
    if enums and not items:
        return prefix + "enum-glob re-export"
    if len(items) >= 2:
        return prefix + "module-glob re-export"
    if enums:
        return prefix + "mixed re-export"
    return prefix + "single-item re-export"


# ------------------------------------------------------------ apply

def normalize(line: str):
    return re.sub(r"\s+", "", line.split("//", 1)[0])


def apply_makepub(repo_rel: str, worktree: Path, name: str, kind: str,
                  enum_name):
    """Edit the defining file to make `name` (or its owning enum) `pub`.
    Idempotent: a no-op if it is already `pub`. Returns True if changed."""
    # repo_rel is relative to C.REPO; map to the on-disk path.
    path = C.REPO / repo_rel
    if not path.exists():
        return False
    text = path.read_text(errors="replace")
    if kind == "enum-variant" and enum_name:
        target, pat = enum_name, re.compile(
            r"(?m)^([ \t]*)(enum\s+" + re.escape(enum_name) + r"\b)")
    elif kind == "fn":
        target, pat = name, re.compile(
            r'(?m)^([ \t]*)((?:extern\s+"C"\s+)?fn\s+' + re.escape(name)
            + r"\s*[(<])")
    else:  # const / static
        target, pat = name, re.compile(
            r"(?m)^([ \t]*)((?:const|static)\s+(?:mut\s+)?"
            + re.escape(name) + r"\s*:)")
    # already pub?
    if re.search(r"(?m)^[ \t]*pub(?:\([^)]*\))?\s+(?:const|static|fn|enum)\s+"
                 + re.escape(target) + r"\b", text):
        return False
    new, n = pat.subn(r"\1pub \2", text, count=1)
    if n:
        path.write_text(new)
        return True
    return False


def apply_plan(plan, worktree: Path):
    prelude = worktree / "crates" / "mp" / "game" / "src" / "prelude.rs"
    if not prelude.exists():
        print(f"[preflight] --apply: no prelude at {prelude}; skipping edits",
              file=sys.stderr)
        return
    text = prelude.read_text(errors="replace")
    present = {normalize(l) for l in text.splitlines()}
    new_lines, made_pub = [], 0

    for fa in plan["file_actions"]:
        # make-pub edits first, at the defining site.
        for it in fa["makepub_items"]:
            if apply_makepub(it["defining_path"], worktree, it["name"],
                             it["kind"], it.get("enum_name")):
                made_pub += 1
        for line in fa["use_lines"]:
            if normalize(line) in present:
                continue
            present.add(normalize(line))
            new_lines.append((line, fa["defining_file"]))

    if not new_lines and not made_pub:
        print("[preflight] --apply: nothing to add (already idempotent)")
        return

    block = [text.rstrip("\n"), "", PRELUDE_MARKER] \
        if PRELUDE_MARKER not in text else [text.rstrip("\n")]
    for line, src in new_lines:
        block.append(f"{line}  // {src}")
    prelude.write_text("\n".join(block) + "\n")
    print(f"[preflight] --apply: +{len(new_lines)} re-export line(s), "
          f"{made_pub} item(s) made pub in {prelude}")


# ------------------------------------------------------------------- main

def main():
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--worktree", default=str(C.REPO),
                    help="tree whose crates/mp/* are tested/edited (default: main repo)")
    ap.add_argument("--in", dest="indir",
                    default=str(Path(__file__).resolve().parent / "out" / "pass3"),
                    help="dir holding missing-symbols.json + manifest (out/pass3)")
    ap.add_argument("--out", default=str(Path(__file__).resolve().parent / "out"))
    ap.add_argument("--apply", action="store_true",
                    help="edit prelude/mod files (idempotent). Off by default.")
    args = ap.parse_args()

    worktree = Path(args.worktree).resolve()
    indir = Path(args.indir)
    sympath = indir / "missing-symbols.json"
    if not sympath.exists():
        print(f"[preflight] missing {sympath}; run symsweep.py first", file=sys.stderr)
        sys.exit(1)

    sym = json.loads(sympath.read_text())
    # Candidate universe = every out-of-file symbol symsweep flagged as not
    # already-ok against its scan (missing ∪ private). The porter only ADDS
    # exports, so this is a superset of the live not-ok set; live re-scan below
    # filters it to what is still unresolved.
    candidates = {}
    for e in sym["missing"] + sym["private_needs_export"]:
        candidates[e["name"]] = {
            "name": e["name"], "kind": e["kind"],
            "source": e.get("source"),
            "sites": e.get("referenced_by_files", []),
        }
    print(f"[preflight] {len(candidates)} candidate symbols from {sympath.name}",
          file=sys.stderr)

    # ---- live worktree re-scan (reuse symsweep's grep-level scanner)
    SS.RUST_ROOTS = {
        "game": worktree / "crates" / "mp" / "game" / "src",
        "bg": worktree / "crates" / "mp" / "bg" / "src",
        "qshared": worktree / "crates" / "mp" / "qshared" / "src",
    }
    missing_roots = [t for t, r in SS.RUST_ROOTS.items() if not r.exists()]
    if missing_roots:
        print(f"[preflight] WARNING: no src for {missing_roots} under {worktree}",
              file=sys.stderr)
    print(f"[preflight] scanning {worktree}/crates/mp/{{game,bg,qshared}}/src ...",
          file=sys.stderr)
    decls, file_texts = SS.scan_worktree_decls()

    # ---- classify each candidate live, shape file-level actions
    file_actions = {}       # defining_file (repo-rel) -> action dict
    residue = defaultdict(list)
    resolved_now = 0
    n_reexport = n_makepub = 0

    for name, c in sorted(candidates.items()):
        ds = decls.get(name, [])
        status, _reason = SS.classify(name, ds, file_texts)
        if status == "ok":
            resolved_now += 1
            continue
        if status == "missing" or not ds:
            header = (c["source"].rsplit(":", 1)[0] if c["source"] else "unknown")
            residue[header].append({"name": name, "kind": c["kind"],
                                    "sites": len(c["sites"])})
            continue
        # private_needs_export & defined somewhere -> actionable
        decl, has_pub = choose_decl(ds)
        defining = decl["path"]
        crate, mod = module_path_of(defining)
        if mod is None:
            residue["unmappable/" + defining].append(
                {"name": name, "kind": c["kind"], "sites": len(c["sites"])})
            continue
        fa = file_actions.get(defining)
        if fa is None:
            fa = file_actions[defining] = {
                "defining_file": defining, "crate": crate, "module_path": mod,
                "reexport_module_items": set(), "reexport_enums": set(),
                "makepub_items": [], "symbols": [],
            }
        fa["symbols"].append(name)
        if has_pub:
            n_reexport += 1
            if decl["kind"] == "enum-variant" and decl.get("enum_name"):
                fa["reexport_enums"].add(decl["enum_name"])
            else:
                fa["reexport_module_items"].add(name)
        else:
            n_makepub += 1
            fa["makepub_items"].append({
                "name": name, "kind": decl["kind"],
                "enum_name": decl.get("enum_name"),
                "defining_path": defining,
            })

    # ---- finalize actions
    actions = []
    for defining, fa in file_actions.items():
        fa["reexport_module_items"] = sorted(fa["reexport_module_items"])
        fa["reexport_enums"] = sorted(fa["reexport_enums"])
        fa["use_lines"] = action_use_lines(fa)
        fa["action_kind"] = action_kind(fa)
        fa["symbols"] = sorted(fa["symbols"])
        actions.append(fa)
    actions.sort(key=lambda a: -len(a["symbols"]))

    residue_flat = sorted(
        ((h, syms) for h, syms in residue.items()),
        key=lambda kv: -len(kv[1]))
    residue_count = sum(len(v) for v in residue.values())

    plan = {
        "worktree": str(worktree),
        "candidate_symbols": len(candidates),
        "resolved_live": resolved_now,
        "actionable_symbols": n_reexport + n_makepub,
        "reexport_symbols": n_reexport,
        "makepub_symbols": n_makepub,
        "defining_files": len(actions),
        "residue_symbols": residue_count,
        "file_actions": actions,
        "residue": {h: syms for h, syms in residue_flat},
    }

    outdir = Path(args.out) / "pass3"
    outdir.mkdir(parents=True, exist_ok=True)
    ppath = outdir / "preflight-plan.json"
    ppath.write_text(json.dumps(plan, indent=1))

    # ---- print human summary
    print(f"\n[preflight] plan -> {ppath}")
    print(f"  worktree              : {worktree}")
    print(f"  candidate symbols     : {plan['candidate_symbols']}")
    print(f"  resolved live (skip)  : {plan['resolved_live']}")
    print(f"  actionable symbols    : {plan['actionable_symbols']} "
          f"({n_reexport} re-export, {n_makepub} make-pub)")
    print(f"  defining files        : {plan['defining_files']}")
    print(f"  residue symbols       : {plan['residue_symbols']} "
          f"across {len(residue)} oracle header(s)")
    print("\n  top file-level actions (defining file -> action -> N symbols):")
    for fa in actions[:10]:
        print(f"    {fa['defining_file']}")
        for line in fa["use_lines"]:
            print(f"        {line}")
        cover = fa["symbols"]
        shown = ", ".join(cover[:6]) + (f" +{len(cover) - 6}" if len(cover) > 6 else "")
        print(f"        [{fa['action_kind']}] covers {len(cover)}: {shown}")
    print("\n  top residue headers (still unresolved anywhere):")
    for h, syms in residue_flat[:8]:
        names = ", ".join(s["name"] for s in syms[:5])
        print(f"    {len(syms):4}  {h}   e.g. {names}")

    if args.apply:
        apply_plan(plan, worktree)
    else:
        print("\n[preflight] dry-run (no files edited). Re-run with --apply to "
              "write re-exports.")


if __name__ == "__main__":
    main()
