#!/usr/bin/env python3
"""PROTOTYPE — throwaway. One-parse batch operations for the porting workflow:

  sweep.py <module> --header game/bg_public.h [--packets] [--json]
      Inventory every type the header owns (name, kind, size, mechanical tier,
      verified badge, cite; with --packets also verbatim source + assert block)
      from a SINGLE TU parse. Replaces per-type scout agent tool calls.

  sweep.py <module> --badges Type1,Type2,...  [--json]
      Verified badges for many types from one parse. Replaces the per-type
      verify loop (N parses -> 1).

Companion to closure.py (shared module profiles, parse, badges).
"""
import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import closure as C
from clang.cindex import CursorKind, TypeKind

STRUCTY = {CursorKind.STRUCT_DECL, CursorKind.UNION_DECL, CursorKind.CLASS_DECL}


def decl_in_header(cur, header):
    f = cur.location.file
    return f is not None and f.name.replace("\\", "/").endswith(header)


def source_slice(path, start, end):
    lines = Path(path).read_text(errors="replace").splitlines()
    return "\n".join(f"{n:>5} | {lines[n - 1]}" for n in range(start, min(end, len(lines)) + 1))


def pointer_bearing(decl):
    defn = decl.get_definition() or decl
    for f in defn.type.get_fields():
        t = f.type
        while t.kind in (TypeKind.CONSTANTARRAY, TypeKind.INCOMPLETEARRAY):
            t = t.element_type
        if t.get_canonical().kind == TypeKind.POINTER:
            return True
    return False


def classify(kind, size, ptr, nfields):
    if kind in ("alias", "enum", "fnptr"):
        return "trivial"
    if size >= 200 or (ptr and nfields > 12):
        return "heavy"
    return "medium"


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("module")
    ap.add_argument("--header", help="oracle-relative header suffix, e.g. game/bg_public.h")
    ap.add_argument("--badges", help="comma-separated type names to badge-check")
    ap.add_argument("--packets", action="store_true", help="include source + assert block per type")
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--source", choices=sorted(C.PROFILES), default="raven")
    ap.add_argument("--root")
    args = ap.parse_args()

    C.MODULES = C.PROFILES[args.source]
    if args.root:
        C.SRC_ROOT = Path(args.root).resolve()
    elif args.source != "raven":
        sys.exit(f"--source {args.source} needs --root")
    if args.module not in C.MODULES:
        sys.exit(f"module '{args.module}' not in profile (has {', '.join(sorted(C.MODULES))})")

    tu = C.parse_tu(args.module, None)
    decls = C.named_decls(tu)
    ported, size_asserts = C.scan_ported(args.module.split("-")[0])
    alias = C.build_alias_map(decls)
    badge = C.make_badger(ported, size_asserts, alias)

    # ---------------------------------------------------------- badges mode
    if args.badges:
        rows = []
        for name in [n.strip() for n in args.badges.split(",") if n.strip()]:
            cur = decls.get(name)
            if cur is None:
                rows.append({"name": name, "badge": "?? not found in TU"})
                continue
            d = cur
            if d.kind == CursorKind.TYPEDEF_DECL:
                inner, _ = C.peel(d.underlying_typedef_type)
                dd = inner.get_declaration()
                if dd.kind in C.RECORD_KINDS:
                    d = dd
            sz = (d.get_definition() or d).type.get_size() if d.kind in C.RECORD_KINDS else None
            rows.append({"name": name, "badge": badge(d.spelling or name, d), "sizeB": sz})
        if args.json:
            print(json.dumps(rows, indent=1))
        else:
            for r in rows:
                print(f"  {r['name']:32} {r['badge']}")
        return

    # ----------------------------------------------------------- sweep mode
    if not args.header:
        sys.exit("need --header or --badges")
    header = args.header

    # anonymous enums in the header, for the `enum {...}; typedef int X;` pattern
    anon_enums = []
    def walk_anon(cur):
        for c in cur.get_children():
            if c.kind == CursorKind.ENUM_DECL and (not c.spelling or c.is_anonymous()) \
                    and decl_in_header(c, header):
                anon_enums.append(c)
            if c.kind in (CursorKind.UNEXPOSED_DECL, CursorKind.LINKAGE_SPEC):
                walk_anon(c)
    walk_anon(tu.cursor)

    seen_records = set()
    rows = []
    for name, cur in sorted(decls.items()):
        if name.startswith("fn:"):
            continue
        target = cur.get_definition() or cur
        if not decl_in_header(target, header):
            continue
        ext = target.extent
        entry = None
        if cur.kind in STRUCTY or cur.kind == CursorKind.ENUM_DECL:
            kind = ("enum" if cur.kind == CursorKind.ENUM_DECL
                    else cur.kind.name.split("_")[0].lower())
            sz = target.type.get_size()
            ptr = cur.kind in STRUCTY and pointer_bearing(target)
            nf = len(list(target.type.get_fields())) if cur.kind in STRUCTY else 0
            entry = dict(name=name, kind=kind, sizeB=sz, pointerBearing=ptr,
                         tier=classify(kind, sz or 0, ptr, nf))
            seen_records.add(name)
        elif cur.kind == CursorKind.TYPEDEF_DECL:
            under, _ = C.peel(cur.underlying_typedef_type)
            udecl = under.get_declaration()
            if udecl.kind in C.RECORD_KINDS and udecl.spelling \
                    and decl_in_header(udecl.get_definition() or udecl, header):
                continue  # named record covered by its own row (alias map links them)
            canon = cur.underlying_typedef_type.get_canonical()
            if canon.kind == TypeKind.POINTER and canon.get_pointee().kind == TypeKind.FUNCTIONPROTO:
                kind = "fnptr"
            elif udecl.kind == CursorKind.ENUM_DECL and (not udecl.spelling or udecl.is_anonymous()):
                kind = "enum"  # typedef enum {...} X with anonymous tag
            else:
                kind = "alias"
            entry = dict(name=name, kind=kind, sizeB=canon.get_size(),
                         pointerBearing=False, tier="trivial")
            # `enum {...}; typedef int X;` — pull the adjacent anon enum into the cite
            if kind == "alias":
                for ae in anon_enums:
                    if 0 <= ext.start.line - ae.extent.end.line <= 5:
                        ext = type("E", (), {"start": ae.extent.start, "end": ext.end})()
                        break
        if entry is None:
            continue
        rel = C.loc(target).rsplit(":", 1)[0]
        entry["cite"] = f"{rel}:{ext.start.line}-{ext.end.line}"
        entry["badge"] = badge(name, target if cur.kind in STRUCTY else None)
        if args.packets:
            entry["source"] = source_slice(ext.start.file.name, ext.start.line, ext.end.line)
            if cur.kind in STRUCTY and (target.type.get_size() or 0) > 0:
                entry["asserts"] = C.rust_asserts(alias.get(name, name), target)
        rows.append(entry)

    if args.json:
        print(json.dumps(rows, indent=1))
    else:
        for r in rows:
            print(f"  {r['name']:32} {r['kind']:6} {str(r['sizeB']):>6}B {r['tier']:7} {r['badge']:44} {r['cite']}")
        print(f"\n{len(rows)} types owned by {header}")


if __name__ == "__main__":
    main()
