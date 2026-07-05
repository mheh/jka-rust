#!/usr/bin/env python3
"""PROTOTYPE — throwaway. Pass-3 prep: exhaustive missing-symbol manifest.

One libclang unity parse of the jampgame oracle (mp-game module — bg_*.c
lives in the same codemp/game/ dir, so one parse covers both tiers) collects
every macro constant / enum constant / file-scope global referenced from a
game or bg .c file, then cross-checks each name against what already exists
(and is actually *reachable*, not just present-somewhere) in the worktree's
crates/mp/{game,bg,qshared}/src.

"Reachable" matters: several names are declared `pub` in one file (e.g.
qshared) yet still fail to resolve (`cannot find value`) at a bare-name use
site elsewhere because nothing there imports them (no explicit `use`, no
matching `Module::*`/`EnumName::*` glob). Grep-level, not syntax-perfect —
matches the existing prototype tooling's house style (see pass2lib.py).

Usage:
  .venv/bin/python symsweep.py [--out DIR]

Companion to fnsweep.py (function manifest) and pass2lib.py (worktree Rust fn
scanner) — this is the const/enum/global analogue for the same pass-3 prep.
"""
import argparse
import json
import re
import sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import closure as C
import fnsweep as FS
from clang.cindex import CursorKind

WT = C.REPO / ".claude" / "worktrees" / "agent-a43cc53200d2fdf54"
RUST_ROOTS = {
    "game": WT / "crates" / "mp" / "game" / "src",
    "bg": WT / "crates" / "mp" / "bg" / "src",
    "qshared": WT / "crates" / "mp" / "qshared" / "src",
}

# ------------------------------------------------------------ oracle side

def is_file_scope_var(cur):
    return (cur.kind == CursorKind.VAR_DECL and cur.semantic_parent is not None
            and cur.semantic_parent.kind == CursorKind.TRANSLATION_UNIT)


def in_root(cur):
    f = cur.location.file
    return f is None or f.name.startswith(str(C.SRC_ROOT))


_file_cache: dict[str, list[str]] = {}


def read_lines(path):
    if path not in _file_cache:
        try:
            _file_cache[path] = Path(path).read_text(errors="replace").splitlines()
        except OSError:
            _file_cache[path] = []
    return _file_cache[path]


def take_snippet(cur, max_lines=3):
    f = cur.extent.start.file
    if f is None:
        return ""
    lines = read_lines(f.name)
    a = cur.extent.start.line
    b = min(cur.extent.end.line, a + max_lines - 1)
    return "\n".join(lines[a - 1:b]).strip("\n")


FUNCLIKE_MACRO_RE_CACHE: dict[str, re.Pattern] = {}


def is_functionlike_macro(defn):
    """clang.cindex in this venv has no is_macro_functionlike() binding —
    match preprocessor semantics directly: function-like requires '(' with
    no space right after the macro name in the #define line."""
    lines = read_lines(defn.location.file.name)
    ln = defn.location.line - 1
    if not (0 <= ln < len(lines)):
        return False
    pat = FUNCLIKE_MACRO_RE_CACHE.get(defn.spelling)
    if pat is None:
        pat = re.compile(r'#\s*define\s+' + re.escape(defn.spelling) + r'\(')
        FUNCLIKE_MACRO_RE_CACHE[defn.spelling] = pat
    return bool(pat.match(lines[ln]))


def collect(tu):
    """Iterative, pruned DFS: only descend into subtrees rooted in the
    oracle tree (skips system-header bodies entirely — the bulk of the AST).
    Collects macro instantiations, enum-constant refs, and file-scope-var
    refs; sites are counted only when they occur in a `codemp/game/*.c` file
    (covers both jampgame and bg_* — same dir per closure.py's module map)."""
    macros: dict[str, dict] = {}
    enums: dict[str, dict] = {}
    gvars: dict[str, dict] = {}

    stack = [tu.cursor]
    while stack:
        cur = stack.pop()
        k = cur.kind
        if k == CursorKind.MACRO_INSTANTIATION:
            defn = cur.get_definition()
            if defn is not None:
                dloc = defn.location.file
                if dloc is not None and dloc.name.startswith(str(C.SRC_ROOT)) \
                        and not is_functionlike_macro(defn):
                    name = cur.spelling
                    e = macros.setdefault(name, {"defn": defn, "sites": set()})
                    if FS.is_game_c(cur):
                        e["sites"].add(FS.basename(cur))
        elif k == CursorKind.DECL_REF_EXPR and cur.referenced is not None:
            ref = cur.referenced
            if ref.kind == CursorKind.ENUM_CONSTANT_DECL:
                rloc = ref.location.file
                if rloc is not None and rloc.name.startswith(str(C.SRC_ROOT)):
                    name = ref.spelling
                    e = enums.setdefault(name, {"decl": ref, "sites": set()})
                    if FS.is_game_c(cur):
                        e["sites"].add(FS.basename(cur))
            elif is_file_scope_var(ref):
                rloc = ref.location.file
                if rloc is not None and rloc.name.startswith(str(C.SRC_ROOT)):
                    name = ref.spelling
                    e = gvars.setdefault(name, {"decl": ref, "sites": set()})
                    if FS.is_game_c(cur):
                        e["sites"].add(FS.basename(cur))
        if cur.kind == CursorKind.TRANSLATION_UNIT or in_root(cur):
            for c in cur.get_children():
                stack.append(c)
    return macros, enums, gvars


# -------------------------------------------------------------- worktree side
DECL_RE = re.compile(
    r'(?m)^[ \t]*(pub(?:\([^)]*\))?)?\s*(const|static)\s+(?:mut\s+)?'
    r'([A-Za-z_]\w*)\s*:')
FN_RE = re.compile(
    r'(?m)^[ \t]*(pub(?:\([^)]*\))?)?\s*(?:extern\s+"C"\s+)?fn\s+([A-Za-z_]\w*)\s*\(')
ENUM_RE = re.compile(r'(pub(?:\([^)]*\))?)?\s*enum\s+([A-Za-z_]\w*)')
USE_LINE_RE = re.compile(r'(?m)^[ \t]*use\s+.*;')


def split_top_level(s):
    """Split on top-level commas only (depth tracked over ([{<>}])."""
    depth = 0
    items, cur = [], []
    for ch in s:
        if ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
        if ch == "," and depth == 0:
            items.append("".join(cur))
            cur = []
        else:
            cur.append(ch)
    if cur:
        items.append("".join(cur))
    return items


VARIANT_NAME_RE = re.compile(
    r'(?:#\[[^\]]*\]\s*|//[^\n]*\n\s*)*([A-Za-z_]\w*)')

_LINE_COMMENT_RE = re.compile(r'//[^\n]*')
_BLOCK_COMMENT_RE = re.compile(r'/\*.*?\*/', re.S)


def blank_comments(text):
    """Replace comment bodies with same-length whitespace (newlines kept), so
    structural scans (brace matching, top-level comma splitting) never see
    brackets/commas inside comments. Offset-preserving: positions in the
    blanked text match the original. (Real bug: an unbalanced `)` in a Raven
    `//#` doc tag — a `:)` smiley in anim_number.rs — drove split_top_level's
    depth negative and silently dropped the last ~500 animNumber_t variants.)"""
    def _blank(m):
        return "".join(c if c == "\n" else " " for c in m.group(0))
    return _LINE_COMMENT_RE.sub(_blank, _BLOCK_COMMENT_RE.sub(_blank, text))


def find_enum_variants(text):
    text = blank_comments(text)
    out = []  # (enum_name, enum_pub, variant_name)
    for m in ENUM_RE.finditer(text):
        vis, ename = m.group(1), m.group(2)
        brace = text.find('{', m.end())
        if brace == -1 or brace - m.end() > 40:  # not immediately the body
            continue
        depth, i = 0, brace
        while i < len(text):
            if text[i] == '{':
                depth += 1
            elif text[i] == '}':
                depth -= 1
                if depth == 0:
                    break
            i += 1
        body = text[brace + 1:i]
        for item in split_top_level(body):
            item = item.strip()
            if not item:
                continue
            vm = VARIANT_NAME_RE.match(item)
            if vm:
                out.append((ename, bool(vis), vm.group(1)))
    return out


def scan_worktree_decls():
    """name -> list of {kind, pub, path (repo-rel), stem, enum_name?}."""
    decls = defaultdict(list)
    file_texts = {}
    for tier, root in RUST_ROOTS.items():
        for rs in sorted(root.rglob("*.rs")):
            text = rs.read_text(errors="replace")
            rel = str(rs.relative_to(C.REPO))
            file_texts[rel] = text
            stem = str(rs.relative_to(root)).removesuffix(".rs")
            for m in DECL_RE.finditer(text):
                vis, kw, name = m.group(1), m.group(2), m.group(3)
                decls[name].append({"kind": kw, "pub": bool(vis),
                                    "path": rel, "stem": stem})
            for m in FN_RE.finditer(text):
                vis, name = m.group(1), m.group(2)
                decls[name].append({"kind": "fn", "pub": bool(vis),
                                    "path": rel, "stem": stem})
            for ename, epub, vname in find_enum_variants(text):
                decls[vname].append({"kind": "enum-variant", "pub": epub,
                                     "path": rel, "stem": stem,
                                     "enum_name": ename})
    return decls, file_texts


def is_reachable(name, decl, usage_file, text):
    """Grep-level: is `name` (declared at `decl`) brought into scope in
    `usage_file` — an explicit `use` mentioning it, or a glob (`Stem::*` /
    `EnumName::*`) over its declaring module/enum?"""
    if usage_file == decl["path"]:
        return True
    for m in USE_LINE_RE.finditer(text):
        line = m.group(0)
        if re.search(r'\b' + re.escape(name) + r'\b', line):
            return True
        if decl["stem"].split("/")[-1] + "::*" in line.replace(" ", ""):
            return True
        ename = decl.get("enum_name")
        if ename and (ename + "::*") in line.replace(" ", ""):
            return True
    return False


BARE_USE_RE_CACHE: dict[str, re.Pattern] = {}


def bare_occurrences(name, text):
    """Line-stripped-of-`use`-statements text, minus `::name` qualified refs
    and the `use` lines themselves; word-boundary hits remaining are bare
    uses that need `name` in scope."""
    pat = BARE_USE_RE_CACHE.get(name)
    if pat is None:
        pat = re.compile(r'(?<!::)\b' + re.escape(name) + r'\b')
        BARE_USE_RE_CACHE[name] = pat
    hits = []
    for lineno, line in enumerate(text.splitlines(), 1):
        stripped = line.split("//", 1)[0]
        if re.match(r'\s*use\s', stripped):
            continue
        if pat.search(stripped):
            hits.append(lineno)
    return hits


def classify(name, decls_for_name, file_texts):
    """-> ('missing', None) | ('private_needs_export', reason) | ('ok', None)"""
    if not decls_for_name:
        return "missing", None
    pub_decls = [d for d in decls_for_name if d["pub"]]
    if not pub_decls:
        return "private_needs_export", (
            f"declared only privately in {len(decls_for_name)} place(s), "
            f"none `pub`")
    # at least one pub decl exists — check every file with a bare reference
    # (excluding the declaring files) actually has it in scope.
    decl_paths = {d["path"] for d in decls_for_name}
    broken_files = []
    for path, text in file_texts.items():
        if path in decl_paths:
            continue
        if name not in text:
            continue
        if not bare_occurrences(name, text):
            continue
        if not any(is_reachable(name, d, path, text) for d in pub_decls):
            broken_files.append(path)
    if broken_files:
        return "private_needs_export", (
            f"pub in {pub_decls[0]['path']} but not glob/use-imported at "
            f"{len(broken_files)} bare-use site(s): "
            f"{', '.join(broken_files[:3])}"
            + (f" +{len(broken_files) - 3} more" if len(broken_files) > 3 else ""))
    return "ok", None


# ------------------------------------------------------------------- main
def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--out", default=str(Path(__file__).resolve().parent / "out"))
    args = ap.parse_args()

    print("[symsweep] parsing mp-game unity TU (jampgame + bg_*.c) ...",
          file=sys.stderr)
    tu = C.parse_tu("mp-game", None, unity=True)
    print("[symsweep] collecting macro/enum/global references ...",
          file=sys.stderr)
    macros, enums, gvars = collect(tu)
    print(f"[symsweep] {len(macros)} macros, {len(enums)} enum-consts, "
          f"{len(gvars)} file-scope vars (pre-filter)", file=sys.stderr)

    print("[symsweep] scanning worktree crates/mp/{game,bg,qshared}/src ...",
          file=sys.stderr)
    decls, file_texts = scan_worktree_decls()

    all_identifiers = []  # (name, kind, defn_or_decl_cursor, sites)
    for name, e in macros.items():
        if e["sites"]:
            all_identifiers.append((name, "macro", e["defn"], e["sites"]))
    for name, e in enums.items():
        if e["sites"]:
            all_identifiers.append((name, "enum-const", e["decl"], e["sites"]))
    for name, e in gvars.items():
        if e["sites"]:
            all_identifiers.append((name, "global-var", e["decl"], e["sites"]))

    missing = []
    private_needs_export = []
    by_kind_total = defaultdict(int)
    by_kind_missing = defaultdict(int)
    by_kind_private = defaultdict(int)
    ok_count = 0

    for name, kind, cur, sites in all_identifiers:
        by_kind_total[kind] += 1
        status, reason = classify(name, decls.get(name, []), file_texts)
        if status == "missing":
            by_kind_missing[kind] += 1
            missing.append({
                "name": name,
                "kind": kind,
                "source": C.loc(cur),
                "snippet": take_snippet(cur),
                "referenced_by_files": sorted(sites),
            })
        elif status == "private_needs_export":
            by_kind_private[kind] += 1
            ds = decls.get(name, [])
            rust_path = (ds[0]["path"] if ds else None)
            private_needs_export.append({
                "name": name,
                "kind": kind,
                "rust_path": rust_path,
                "all_rust_paths": sorted({d["path"] for d in ds}),
                "reason": reason,
                "referenced_by_files": sorted(sites),
            })
        else:
            ok_count += 1

    missing.sort(key=lambda m: -len(m["referenced_by_files"]))
    private_needs_export.sort(key=lambda m: -len(m["referenced_by_files"]))

    stats = {
        "total_identifiers_considered": len(all_identifiers),
        "by_kind_total": dict(by_kind_total),
        "missing_count": len(missing),
        "missing_by_kind": dict(by_kind_missing),
        "private_needs_export_count": len(private_needs_export),
        "private_needs_export_by_kind": dict(by_kind_private),
        "already_ok_count": ok_count,
    }

    out = {"missing": missing, "private_needs_export": private_needs_export,
           "stats": stats}

    outdir = Path(args.out) / "pass3"
    outdir.mkdir(parents=True, exist_ok=True)
    opath = outdir / "missing-symbols.json"
    opath.write_text(json.dumps(out, indent=1))

    print(f"[symsweep] wrote {opath}")
    print(f"[symsweep] stats: {json.dumps(stats, indent=1)}")

    # ---- sanity check
    sanity_names = ["CONTENTS_LAVA", "MASK_SHOT", "SFL2_TRANSITION_DAMAGE",
                     "saberMoveData", "MAT_GLASS", "EF_TALK", "MAX_PS_EVENTS",
                     "VH_FIGHTER", "DAMAGE_NO_KNOCKBACK",
                     "G2TRFLAG_DOGHOULTRACE"]
    missing_names = {m["name"] for m in missing}
    private_names = {m["name"] for m in private_needs_export}
    print("\n[symsweep] sanity check:")
    all_ok = True
    for n in sanity_names:
        where = ("missing" if n in missing_names else
                 "private_needs_export" if n in private_names else
                 "NOT FOUND IN EITHER — FAIL")
        if where == "NOT FOUND IN EITHER — FAIL":
            all_ok = False
        print(f"  {n:28} -> {where}")
    print(f"\n[symsweep] sanity check {'PASSED' if all_ok else 'FAILED'}")


if __name__ == "__main__":
    main()
