#!/usr/bin/env python3
"""PROTOTYPE — throwaway. Question: can libclang, given per-module flags,
mechanically produce (a) the transitive dependency closure of a Raven type or
function, (b) by-value vs pointer-only classification, (c) ported/unported
status vs crates/, and (d) ground-truth size/offset layout data?

Usage:
  .venv/bin/python closure.py <module> <symbol> [--tree] [--layout] [--asserts] [--file <c-file>]
  .venv/bin/python closure.py --source openjk --root <path> mp-game gclient_s --tree
  .venv/bin/python closure.py --list-modules

Modules supply the per-module compile flags (defines/includes/lang): the
`raven` profile mirrors the oracle vcproj Release configs, the `openjk`
profile mirrors JACoders/OpenJK's CMakeLists. See NOTES.md for the verdict.
"""
import argparse
import re
import subprocess
import sys
from pathlib import Path

import clang.cindex as ci
from clang.cindex import CursorKind, TypeKind

REPO = Path(__file__).resolve().parents[2]
ORACLE = REPO / "oracle" / "oracle"
CRATES = REPO / "crates"
SRC_ROOT = ORACLE  # reassigned by --root for non-oracle trees (e.g. OpenJK)

# ---------------------------------------------------------------- module DB
# Defines mirror the vcproj Release (non-FINAL_BUILD) configs; WIN32/_WINDOWS
# omitted deliberately (they gate macros/inline asm, not layouts) so the parse
# matches the 64-bit host layouts the existing Rust asserts were verified on.
RAVEN_MODULES = {
    # MP tree (codemp/) — plain C modules
    "mp-game": dict(
        lang="c", entry="codemp/game/g_local.h",
        includes=["codemp/game"],
        defines=["NDEBUG", "MISSIONPACK", "QAGAME", "_JK2"],
        srcglob=["codemp/game/*.c"]),
    # bg_* compiles into game/cgame/ui; parse it through the game TU. Alias
    # so tier-named callers (crate mp_bg) resolve without knowing that.
    "mp-bg": dict(
        lang="c", entry="codemp/game/g_local.h",
        includes=["codemp/game"],
        defines=["NDEBUG", "MISSIONPACK", "QAGAME", "_JK2"],
        srcglob=["codemp/game/bg_*.c"]),
    "mp-cgame": dict(
        lang="c", entry="codemp/cgame/cg_local.h",
        includes=["codemp/cgame", "codemp/game", "codemp/ui"],
        defines=["NDEBUG", "MISSIONPACK", "CGAME", "_JK2"],
        srcglob=["codemp/cgame/*.c", "codemp/game/bg_*.c", "codemp/ui/ui_shared.c"]),
    "mp-ui": dict(
        lang="c", entry="codemp/ui/ui_local.h",
        includes=["codemp/ui", "codemp/game"],
        defines=["NDEBUG", "MISSIONPACK", "UI_EXPORTS", "_JK2"],
        srcglob=["codemp/ui/*.c", "codemp/game/bg_*.c"]),
    "mp-engine": dict(
        lang="c++", entry="codemp/qcommon/qcommon.h",
        includes=["codemp/qcommon", "codemp/game"],
        defines=["NDEBUG", "MISSIONPACK", "_JK2"]),
    "mp-botlib": dict(
        lang="c", entry="codemp/botlib/botlib.h",
        includes=["codemp/botlib", "codemp/game"],
        defines=["NDEBUG", "MISSIONPACK", "BOTLIB", "_JK2"]),
    "mp-renderer": dict(
        lang="c++", entry="codemp/renderer/tr_local.h",
        includes=["codemp/renderer", "codemp/game", "codemp/qcommon"],
        defines=["NDEBUG", "MISSIONPACK", "_JK2"]),
    # SP tree (code/) — C++ throughout
    "sp-game": dict(
        lang="c++", entry="code/game/g_local.h",
        includes=["code/game", "code"],
        defines=["NDEBUG", "_IMMERSION"],
        srcglob=["code/game/*.cpp"]),
    "sp-bg": dict(
        lang="c++", entry="code/game/g_local.h",
        includes=["code/game", "code"],
        defines=["NDEBUG", "_IMMERSION"],
        srcglob=["code/game/bg_*.cpp"]),
    "sp-cgame": dict(
        lang="c++", entry="code/cgame/cg_local.h",
        includes=["code/cgame", "code/game", "code"],
        defines=["NDEBUG", "_IMMERSION"]),
    "sp-ui": dict(
        lang="c++", entry="code/ui/ui_local.h",
        includes=["code/ui", "code/game", "code"],
        defines=["NDEBUG", "_IMMERSION"]),
    "sp-engine": dict(
        lang="c++", entry="code/qcommon/qcommon.h",
        includes=["code/qcommon", "code/game", "code"],
        defines=["NDEBUG", "_IMMERSION"]),
    "sp-renderer": dict(
        lang="c++", entry="code/renderer/tr_local.h",
        includes=["code/renderer", "code/game", "code/qcommon", "code"],
        defines=["NDEBUG", "_IMMERSION"]),
}

# JACoders/OpenJK profile — defines/includes mirror its CMakeLists:
# MPGameDefines=_GAME, MPCGameDefines=_CGAME, MPUIDefines=UI_BUILD,
# SPGameDefines=SP_GAME; include dirs are the tree root + shared/.
OPENJK_MODULES = {
    "mp-game": dict(
        lang="c", entry="codemp/game/g_local.h",
        includes=["codemp", "shared", "codemp/game"],
        defines=["NDEBUG", "_GAME"]),
    "mp-cgame": dict(
        lang="c", entry="codemp/cgame/cg_local.h",
        includes=["codemp", "shared", "codemp/cgame", "codemp/game", "codemp/ui"],
        defines=["NDEBUG", "_CGAME"]),
    "mp-ui": dict(
        lang="c", entry="codemp/ui/ui_local.h",
        includes=["codemp", "shared", "codemp/ui", "codemp/game"],
        defines=["NDEBUG", "UI_BUILD"]),
    "sp-game": dict(
        lang="c++", entry="code/game/g_local.h",
        includes=["code", "shared", "code/game"],
        defines=["NDEBUG", "SP_GAME"]),
    "sp-cgame": dict(
        lang="c++", entry="code/cgame/cg_local.h",
        includes=["code", "shared", "code/cgame", "code/game"],
        defines=["NDEBUG"]),
}

PROFILES = {"raven": RAVEN_MODULES, "openjk": OPENJK_MODULES}
MODULES = RAVEN_MODULES  # default; swapped by --source in main()

RECORD_KINDS = {CursorKind.STRUCT_DECL, CursorKind.UNION_DECL,
                CursorKind.CLASS_DECL, CursorKind.ENUM_DECL}


_SHIM_CACHE: list | None = None


def backslash_include_shims():
    """Windows-style `#include "..\\game\\x.h"` fails on POSIX. Shadow the few
    offending files (9 in the oracle) via unsaved_files with slashes fixed —
    the on-disk tree is never touched."""
    global _SHIM_CACHE
    if _SHIM_CACHE is None:
        hits = subprocess.run(
            ["grep", "-rl", r'#include "[^"]*\\', str(SRC_ROOT),
             "--include=*.h", "--include=*.c", "--include=*.cpp"],
            capture_output=True, text=True).stdout.split()
        inc_re = re.compile(r'(#include\s*"[^"]*")')
        _SHIM_CACHE = [
            (p, inc_re.sub(lambda m: m.group(1).replace("\\", "/"),
                           Path(p).read_text(errors="replace")))
            for p in hits]
    return _SHIM_CACHE


def parse_tu(module: str, extra_file: str | None, unity: bool = False):
    cfg = MODULES[module]
    args = [f"-x{'c++' if cfg['lang'] == 'c++' else 'c'}"]
    if cfg["lang"] == "c++":
        # era-appropriate for the oracle; OpenJK is modernized C++
        args.append("-std=c++03" if SRC_ROOT == ORACLE else "-std=c++11")
    args += [f"-I{SRC_ROOT / inc}" for inc in cfg["includes"]]
    args += [f"-D{d}" for d in cfg["defines"]]
    # No platform macro is defined (WIN32/MACOS_X gate macros+inline asm, not
    # layouts), so supply the two the headers expect from the platform section.
    args += ["-DID_INLINE=inline", "-DMAC_STATIC="]
    # keep plain `//` comment blocks attached to decls (cursor.raw_comment)
    args += ["-fparse-all-comments"]
    args += ["-Wno-everything", "-ferror-limit=0"]
    if sys.platform == "darwin":
        sdk = subprocess.run(["xcrun", "--show-sdk-path"], capture_output=True,
                             text=True).stdout.strip()
        if sdk:
            args.append(f"-isysroot{sdk}")
        res = subprocess.run(["clang", "-print-resource-dir"], capture_output=True,
                             text=True).stdout.strip()
        if res:
            args.append(f"-resource-dir={res}")
    entry = SRC_ROOT / (extra_file or cfg["entry"])
    if not entry.exists():
        sys.exit(f"entry file not found: {entry}")
    src = str(entry)
    unsaved = []
    if unity:  # one TU including every module source file, so callee bodies exist
        if "srcglob" not in cfg:
            sys.exit(f"module '{module}' has no srcglob — function trees need one")
        files = sorted(p for g in cfg["srcglob"] for p in SRC_ROOT.glob(g))
        src = str(entry.parent / "__closure_unity__.c")
        unsaved.append((src, "".join(f'#include "{p}"\n' for p in files)))
    elif entry.suffix == ".h":  # synthesize a TU around the header
        src = str(entry.parent / "__closure_tu__.c")
        unsaved.append((src, f'#include "{entry.name}"\n'))
    unsaved += backslash_include_shims()
    idx = ci.Index.create()
    tu = idx.parse(src, args=args, unsaved_files=unsaved,
                   options=ci.TranslationUnit.PARSE_DETAILED_PROCESSING_RECORD)
    fatals = [d for d in tu.diagnostics if d.severity >= ci.Diagnostic.Error]
    if fatals:
        print(f"[warn] {len(fatals)} parse errors (layouts may be partial); first:",
              file=sys.stderr)
        for d in fatals[:3]:
            print(f"  {d.location.file}:{d.location.line}: {d.spelling}",
                  file=sys.stderr)
    return tu


# ------------------------------------------------------------- ported scan
def scan_ported(mode: str):
    """(status, size_asserts) from crates/**/*.rs — status: name -> ('ported',
    path) | ('todo', path); size_asserts: name -> {asserted sizes}. Prefers
    declarations in the matching mode's tree (mp/ vs sp/) over the other."""
    decl_re = re.compile(r"^\s*pub\s+(?:struct|enum|union)\s+(\w+)|^\s*pub\s+type\s+(\w+)\s*=", re.M)
    todo_re = re.compile(r"//\s*TODO:\s*Port\s+(\w+)")
    size_re = re.compile(r"size_of::<\s*(\w+)\s*>\s*\(\)\s*==\s*(\d+)")
    status: dict[str, tuple[str, str]] = {}
    # keyed by file, then type name — asserts only count when they live in the
    # same file as the declaration (house style: one type per file, colocated
    # asserts). Prevents MP asserts vouching for SP stubs and vice versa.
    size_asserts: dict[str, dict[str, set[int]]] = {}
    def rank(rel):  # own-mode > native > other-mode
        if rel.startswith(f"crates/{mode}/"):
            return 0
        if rel.startswith("crates/native/"):
            return 1
        return 2
    for rs in CRATES.rglob("*.rs"):
        if "target" in rs.parts:
            continue
        text = rs.read_text(errors="replace")
        rel = str(rs.relative_to(REPO))
        for m in decl_re.finditer(text):
            name = m.group(1) or m.group(2)
            prev = status.get(name)
            if prev is None or prev[0] != "ported" or rank(rel) < rank(prev[1]):
                status[name] = ("ported", rel)
        for m in todo_re.finditer(text):
            status.setdefault(m.group(1), ("todo", rel))
        for m in size_re.finditer(text):
            size_asserts.setdefault(rel, {}).setdefault(m.group(1), set()).add(int(m.group(2)))
    return status, size_asserts


def build_alias_map(decls):
    """tag -> typedef alias (struct playerState_s vs typedef playerState_t):
    the Rust port declares under the typedef name, so badge on either."""
    alias: dict[str, str] = {}
    for name, c in decls.items():
        if c.kind == CursorKind.TYPEDEF_DECL:
            under = c.underlying_typedef_type.get_declaration()
            if under.kind in RECORD_KINDS and under.spelling and under.spelling != name:
                alias[under.spelling] = name
    return alias


def make_badger(ported, size_asserts, alias):
    """badge(name, decl=None) -> verified ported-status string."""
    def badge(name, decl=None):
        rust_name = name if name in ported else alias.get(name, name)
        st = ported.get(rust_name)
        if st is None:
            st = ported.get(name)
        if st and st[0] == "todo":
            return f"◐ TODO marker in {st[1]}"
        if not st:
            return "☐ UNPORTED"
        # Verify, don't just name-match: a struct/union port must carry a
        # size_of assert agreeing with clang's ground truth (enums/aliases are
        # exempt — the house style doesn't assert those).
        if decl is not None and decl.kind in (CursorKind.STRUCT_DECL,
                                              CursorKind.UNION_DECL,
                                              CursorKind.CLASS_DECL):
            csize = (decl.get_definition() or decl).type.get_size()
            if csize and csize > 0:
                in_file = size_asserts.get(st[1], {})
                names = {name, rust_name, alias.get(name, name)}
                asserted = set().union(*(in_file.get(n, set()) for n in names))
                if not asserted:
                    return f"◐ declared, NO SIZE ASSERT (stub?) {st[1]}"
                if csize not in asserted:
                    return (f"✗ SIZE MISMATCH rust asserts {sorted(asserted)}, "
                            f"oracle says {csize}B  {st[1]}")
        return f"☑ {st[1]}"
    return badge


# --------------------------------------------------------------- closure
def loc(cur):
    f = cur.location.file
    if not f:
        return "?"
    p = Path(f.name)
    for base in (REPO, SRC_ROOT):
        try:
            p = p.relative_to(base)
            break
        except ValueError:
            continue
    return f"{p}:{cur.location.line}"


def named_decls(tu):
    """symbol name -> definition cursor (records, enums, typedefs, functions)."""
    out = {}
    def visit(cur):
        for c in cur.get_children():
            if c.location.file is None:
                continue
            if c.kind in RECORD_KINDS or c.kind == CursorKind.TYPEDEF_DECL:
                if c.spelling:
                    prev = out.get(c.spelling)
                    if prev is None or (c.is_definition() and not prev.is_definition()):
                        out[c.spelling] = c
            elif c.kind == CursorKind.FUNCTION_DECL:
                key = f"fn:{c.spelling}"
                prev = out.get(key)
                if prev is None or (c.is_definition() and not prev.is_definition()):
                    out[key] = c
            if c.kind in (CursorKind.NAMESPACE, CursorKind.UNEXPOSED_DECL,
                          CursorKind.LINKAGE_SPEC):
                visit(c)
    visit(tu.cursor)
    return out


def peel(t):
    """Strip arrays/typedefs (not pointers). -> (canonical type, saw_pointer)."""
    saw_ptr = False
    while True:
        k = t.kind
        if k in (TypeKind.CONSTANTARRAY, TypeKind.INCOMPLETEARRAY, TypeKind.VECTOR):
            t = t.element_type
        elif k in (TypeKind.POINTER, TypeKind.LVALUEREFERENCE, TypeKind.RVALUEREFERENCE):
            saw_ptr = True
            t = t.get_pointee()
        elif k in (TypeKind.TYPEDEF, TypeKind.ELABORATED, TypeKind.UNEXPOSED):
            c = t.get_canonical()
            if c.kind == k:
                break
            t = c
        else:
            break
    return t, saw_ptr


def short_name(decl):
    if decl.spelling and not decl.is_anonymous():
        return decl.spelling
    kind = decl.kind.name.split("_")[0].lower()
    return f"(anon {kind} {Path(loc(decl)).name})"


class Closure:
    def __init__(self):
        self.byval: dict[str, ci.Cursor] = {}   # must be layout-ported
        self.ptronly: dict[str, ci.Cursor] = {} # deferrable as opaque
        self.order: list[str] = []              # topo (deps before dependents)
        self._visiting: set[str] = set()

    def add_type(self, t, via_ptr=False):
        t, saw_ptr = peel(t)
        via_ptr = via_ptr or saw_ptr
        if t.kind == TypeKind.FUNCTIONPROTO:
            for a in t.argument_types():
                self.add_type(a, via_ptr=True)
            self.add_type(t.get_result(), via_ptr=True)
            return
        decl = t.get_declaration()
        if decl.kind not in RECORD_KINDS:
            return
        name = short_name(decl)
        if via_ptr:
            if name not in self.byval:
                self.ptronly.setdefault(name, decl)
            return
        self.ptronly.pop(name, None)
        if name in self.byval or name in self._visiting:
            return
        self._visiting.add(name)
        self.byval[name] = decl
        defn = decl.get_definition() or decl
        if decl.kind != CursorKind.ENUM_DECL:
            for f in defn.type.get_fields():
                self.add_type(f.type)
        self._visiting.discard(name)
        self.order.append(name)


def function_closure(fn_cur, clo):
    """Types + direct callees referenced by a function definition."""
    callees = {}
    clo.add_type(fn_cur.result_type, via_ptr=True)
    for a in fn_cur.get_arguments():
        clo.add_type(a.type, via_ptr=True)
    def walk(cur):
        for c in cur.get_children():
            if c.kind == CursorKind.CALL_EXPR and c.referenced is not None:
                callees[c.referenced.spelling] = c.referenced
            elif c.kind == CursorKind.VAR_DECL:
                clo.add_type(c.type)  # locals are by-value needs
            elif c.kind in (CursorKind.TYPE_REF, CursorKind.MEMBER_REF_EXPR):
                if c.referenced is not None and c.referenced.kind in RECORD_KINDS:
                    clo.add_type(c.referenced.type, via_ptr=True)
            walk(c)
    body = fn_cur.get_definition() or fn_cur
    walk(body)
    return callees


# ------------------------------------------------------------------ tree
def print_tree(root_decl, badge, display):
    """Field-labeled dependency tree, fully expanded at every occurrence.
    Safe: by-value cycles are impossible in C (cycles need pointers, and
    pointer fields stay as `*` opaque-ok leaves)."""

    def record_children(d):
        defn = d.get_definition() or d
        if d.kind == CursorKind.ENUM_DECL or not defn.is_definition():
            return []
        kids = []
        for f in defn.type.get_fields():
            t, saw_ptr = peel(f.type)
            dd = t.get_declaration()
            if dd.kind in RECORD_KINDS:
                kids.append((f, dd, saw_ptr))
        return kids

    def label(f, dd, saw_ptr):
        name = short_name(dd)
        sz = (dd.get_definition() or dd).type.get_size()
        kind = dd.kind.name.split("_")[0].lower()
        anon = dd.is_anonymous()
        ftype = name if anon else (f.type.spelling if f is not None else name)
        fname = ("<anon>" if anon or not f.spelling else f.spelling) if f is not None else None
        base = f"{fname}: {ftype}" if f is not None else display(name)
        b = badge(name, dd if not saw_ptr else None)  # no size check via fwd decls
        tail = f"{kind} {sz}B  {b}" if not saw_ptr else f"{kind} — ptr, opaque ok  {b}"
        return f"{'*' if saw_ptr else ''}{base}  [{tail}]"

    def rec(d, prefix, is_last, f=None, saw_ptr=False):
        connector = "" if f is None and prefix == "" else ("└─ " if is_last else "├─ ")
        print(prefix + connector + label(f, d, saw_ptr))
        if saw_ptr:
            return
        kids = record_children(d)
        child_prefix = prefix + ("" if connector == "" else ("   " if is_last else "│  "))
        for i, (cf, cd, cptr) in enumerate(kids):
            rec(cd, child_prefix, i == len(kids) - 1, cf, cptr)

    rec(root_decl, "", True)


def print_call_tree(root_fn, max_depth=None):
    """Recursive call tree. Unlike structs, call graphs have cycles and heavy
    fan-in, so each function expands once; repeats are marked ↺. Repeated
    calls to the same callee within one body collapse to ×N."""
    expanded: set[str] = set()

    def callees(fn):
        defn = fn.get_definition()
        if defn is None:
            return None  # no body in TU (engine import, unresolved, …)
        order, idx = [], {}
        def walk(cur):
            for c in cur.get_children():
                if c.kind == CursorKind.CALL_EXPR and c.referenced is not None \
                        and c.referenced.kind == CursorKind.FUNCTION_DECL:
                    name = c.referenced.spelling
                    if name.startswith("__builtin"):  # fortify-macro noise
                        walk(c)
                        continue
                    if name in idx:
                        order[idx[name]][1] += 1
                    else:
                        idx[name] = len(order)
                        order.append([c.referenced, 1])
                walk(c)
        walk(defn)
        return order

    def is_external(fn):
        f = (fn.get_definition() or fn).location.file
        return f is not None and not f.name.startswith(str(SRC_ROOT))

    def rec(fn, prefix, is_last, count=1, root=False, depth=0):
        name = fn.spelling
        connector = "" if root else ("└─ " if is_last else "├─ ")
        line = f"{prefix}{connector}{name}"
        if count > 1:
            line += f" ×{count}"
        if is_external(fn):
            print(line + "  [libc/SDK]")
            return
        kids = callees(fn)
        line += f"  ({loc(fn.get_definition() or fn)})"
        if kids is None:
            line += "  [no body in TU]"
        repeat = name in expanded and kids
        if repeat:
            line += "  ↺ expanded above"
        truncated = kids and max_depth is not None and depth >= max_depth
        if truncated and not repeat:
            line += f"  … +{len(kids)} callees (depth cap)"
        print(line)
        if kids is None or repeat or truncated:
            return
        expanded.add(name)
        child_prefix = prefix + ("" if root else ("   " if is_last else "│  "))
        for i, (k, cnt) in enumerate(kids):
            rec(k, child_prefix, i == len(kids) - 1, cnt, depth=depth + 1)

    rec(root_fn, "", True, root=True)


# ---------------------------------------------------------------- layout
def layout(decl):
    t = (decl.get_definition() or decl).type
    size = t.get_size()
    rows = []
    for f in t.get_fields():
        off = t.get_offset(f.spelling)
        rows.append((f.spelling, off // 8 if off >= 0 else None,
                     f.type.spelling, f.type.get_size()))
    return size, rows


def rust_asserts(name, decl):
    size, rows = layout(decl)
    out = [f"const _: () = assert!(core::mem::size_of::<{name}>() == {size});"]
    for fname, off, _, _ in rows:
        if off is not None and fname:
            out.append(f"const _: () = assert!(core::mem::offset_of!({name}, {fname}) == {off});")
    return "\n".join(out)


# ------------------------------------------------------------------ main
def main():
    global MODULES, SRC_ROOT
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("module", nargs="?", choices=sorted(MODULES))
    ap.add_argument("symbol", nargs="?",
                    help="type name, or fn:Name for a function closure")
    ap.add_argument("--file", help="oracle-relative .c/.cpp to parse instead of "
                                   "the module entry header (needed for fn bodies)")
    ap.add_argument("--layout", action="store_true", help="field offsets for symbol")
    ap.add_argument("--asserts", action="store_true", help="emit Rust assert block")
    ap.add_argument("--tree", action="store_true", help="dependency tree instead of flat closure")
    ap.add_argument("--depth", type=int, help="max call-tree depth (functions only)")
    ap.add_argument("--source", choices=sorted(PROFILES), default="raven",
                    help="flag profile: raven vcprojs or OpenJK CMakeLists")
    ap.add_argument("--root", help="source tree root (required for --source openjk)")
    ap.add_argument("--list-modules", action="store_true")
    args = ap.parse_args()

    MODULES = PROFILES[args.source]
    if args.root:
        SRC_ROOT = Path(args.root).resolve()
    elif args.source != "raven":
        sys.exit(f"--source {args.source} needs --root <path-to-tree>")

    if args.list_modules or not (args.module and args.symbol):
        for prof, mods in PROFILES.items():
            print(f"[{prof}]")
            for m, cfg in mods.items():
                print(f"  {m:12} {cfg['lang']:3} {cfg['entry']}  -D{' -D'.join(cfg['defines'])}")
        return
    if args.module not in MODULES:
        sys.exit(f"module '{args.module}' not in profile '{args.source}' "
                 f"(has: {', '.join(sorted(MODULES))})")

    is_fn_target = (args.symbol or "").startswith("fn:")
    unity = is_fn_target and args.tree and not args.file
    tu = parse_tu(args.module, args.file, unity=unity)
    decls = named_decls(tu)
    ported, size_asserts = scan_ported(args.module.split("-")[0])
    alias = build_alias_map(decls)

    clo = Closure()
    is_fn = args.symbol.startswith("fn:")
    target = decls.get(args.symbol)
    if target is None:
        near = [k for k in decls if args.symbol.lower() in k.lower()][:10]
        sys.exit(f"'{args.symbol}' not found in {args.module} TU."
                 + (f" Near matches: {near}" if near else ""))

    callees = {}
    if is_fn:
        if args.tree:
            print()
            print_call_tree(target, max_depth=args.depth)
            return
        callees = function_closure(target, clo)
        print(f"function {target.spelling}  ({loc(target)})")
    else:
        cur = target
        if cur.kind == CursorKind.TYPEDEF_DECL:
            clo.add_type(cur.underlying_typedef_type)
        else:
            clo.add_type(cur.type)

    badge = make_badger(ported, size_asserts, alias)

    def display(name):
        return f"{name} ({alias[name]})" if name in alias else name

    if args.tree and not is_fn:
        d = clo.byval.get(args.symbol) or target
        if d.kind == CursorKind.TYPEDEF_DECL:
            inner, _ = peel(d.underlying_typedef_type)
            d = inner.get_declaration()
        print()
        print_tree(d, badge, display)
        unported = [n for n in clo.order if "UNPORTED" in badge(n)]
        print(f"\n{len(clo.order)} by-value types, {len(unported)} unported"
              + (f": {', '.join(unported)}" if unported else ""))
        return

    print(f"\n== by-value closure ({len(clo.order)}), port order (deps first) ==")
    for name in clo.order:
        d = clo.byval[name]
        sz = (d.get_definition() or d).type.get_size()
        kind = d.kind.name.split("_")[0].lower()
        print(f"  {display(name):32} {kind:6} {sz:>6}B  {badge(name, d):50} {loc(d)}")

    if clo.ptronly:
        print(f"\n== pointer-only deps ({len(clo.ptronly)}) — deferrable as opaque ==")
        for name, d in sorted(clo.ptronly.items()):
            print(f"  {display(name):32} {badge(name, d):50} {loc(d)}")

    if callees:
        print(f"\n== direct callees ({len(callees)}) ==")
        for name, c in sorted(callees.items()):
            st = "☑" if f"fn:{name}" in decls and False else " "
            print(f"  {name:32} {loc(c)}")

    if (args.layout or args.asserts) and not is_fn:
        d = clo.byval.get(args.symbol) or target
        if d.kind == CursorKind.TYPEDEF_DECL:
            inner, _ = peel(d.underlying_typedef_type)
            d = inner.get_declaration()
        name = args.symbol
        if args.layout:
            size, rows = layout(d)
            print(f"\n== layout {name}: size {size} ==")
            for fname, off, ftype, fsize in rows:
                print(f"  {off if off is not None else '?':>6}  {fname:28} {ftype}  ({fsize}B)")
        if args.asserts:
            print(f"\n{rust_asserts(name, d)}")


if __name__ == "__main__":
    main()
