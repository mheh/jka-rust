#!/usr/bin/env python3
"""PROTOTYPE — throwaway. Port-packet generator: given one Raven function,
emit a self-contained packet (verbatim body, cites, doc header, deps, globals,
syscalls, paste-ready markers) so a porting agent needs NO file access.

Usage:
  .venv/bin/python portpacket.py <module> <FunctionName> [--json] [--helper-cap N]
                                 [--source raven|openjk] [--root PATH]

Companion to closure.py (shared module profiles, unity parse, verified badges).
"""
import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import closure as C
from clang.cindex import CursorKind, StorageClass

RECORD_KINDS = C.RECORD_KINDS


def cite(cur):
    """repo-relative file:START-END for a cursor's full extent."""
    defn = cur.get_definition() or cur
    ext = defn.extent
    path = C.loc(defn).rsplit(":", 1)[0]
    if ext.start.line == ext.end.line:
        return f"{path}:{ext.start.line}"
    return f"{path}:{ext.start.line}-{ext.end.line}"


def body_lines(cur):
    """[(lineno, text)] of the cursor's verbatim source extent."""
    defn = cur.get_definition() or cur
    ext = defn.extent
    if ext.start.file is None:
        return []
    lines = Path(ext.start.file.name).read_text(errors="replace").splitlines()
    return [(n, lines[n - 1]) for n in range(ext.start.line, ext.end.line + 1)]


def clean_comment(raw):
    if not raw:
        return None
    out = []
    for ln in raw.splitlines():
        ln = ln.strip()
        for pre in ("///", "//", "/*", "*/", "*"):
            if ln.startswith(pre):
                ln = ln[len(pre):].strip()
        if ln.strip("=- "):
            out.append(ln)
    return "\n".join(out) or None


def preceding_comment(cur):
    """Comment block immediately above the definition, read straight from the
    source (clang's raw_comment mis-associates across neighboring decls)."""
    defn = cur.get_definition() or cur
    ext = defn.extent
    if ext.start.file is None:
        return None
    lines = Path(ext.start.file.name).read_text(errors="replace").splitlines()
    i = ext.start.line - 2  # 0-based index of the line above the definition
    block = []
    while i >= 0:
        s = lines[i].strip()
        if s.startswith("//"):
            block.append(s)
            i -= 1
        elif s.endswith("*/"):
            while i >= 0:
                block.append(lines[i].strip())
                if lines[i].lstrip().startswith("/*"):
                    break
                i -= 1
            i -= 1
        else:
            break
    block.reverse()
    return clean_comment("\n".join(block)) if block else None


def collect_globals(fn):
    """File-scope VAR_DECLs referenced in the body: name -> cursor."""
    seen = {}
    def walk(cur):
        for c in cur.get_children():
            if c.kind == CursorKind.DECL_REF_EXPR and c.referenced is not None \
                    and c.referenced.kind == CursorKind.VAR_DECL \
                    and c.referenced.semantic_parent is not None \
                    and c.referenced.semantic_parent.kind == CursorKind.TRANSLATION_UNIT:
                seen.setdefault(c.referenced.spelling, c.referenced)
            walk(c)
    walk(fn.get_definition() or fn)
    return seen


def classify_callee(cur, name):
    if name.startswith("trap_"):
        return "syscall"
    f = (cur.get_definition() or cur).location.file
    if f is not None and not f.name.startswith(str(C.SRC_ROOT)):
        return "libc/SDK"
    if cur.get_definition() is None:
        return "no body in TU"
    return "in-module"


def build_packet(module, fname, helper_cap):
    tu = C.parse_tu(module, None, unity=True)
    decls = C.named_decls(tu)
    fn = decls.get(f"fn:{fname}")
    if fn is None or fn.get_definition() is None:
        near = [k[3:] for k in decls if k.startswith("fn:") and fname.lower() in k.lower()][:10]
        sys.exit(f"function '{fname}' not found (with body) in {module}."
                 + (f" Near matches: {near}" if near else ""))
    fn = fn.get_definition()

    ported, size_asserts = C.scan_ported(module.split("-")[0])
    alias = C.build_alias_map(decls)
    badge = C.make_badger(ported, size_asserts, alias)

    clo = C.Closure()
    callees = C.function_closure(fn, clo)
    callees = {n: c for n, c in callees.items() if not n.startswith("__builtin")}
    globals_ = collect_globals(fn)

    pkt = {
        "target": fname,
        "module": module,
        "signature": f"{fn.result_type.spelling} {fn.displayname}",
        "cite": cite(fn),
        "raven_comment": preceding_comment(fn),
        "body": body_lines(fn),
        "callees": [],
        "helpers": [],
        "types_byval": [],
        "types_ptronly": [],
        "globals": [],
        "syscalls": [],
        "markers": [],
    }

    for name in sorted(callees):
        cur = callees[name]
        kind = classify_callee(cur, name)
        entry = {"name": name, "kind": kind,
                 "cite": cite(cur) if kind not in ("libc/SDK",) else None}
        pkt["callees"].append(entry)
        if kind == "syscall":
            pkt["syscalls"].append({"name": name, "cite": entry["cite"]})
        elif kind == "in-module":
            lines = body_lines(cur)
            if len(lines) <= helper_cap:
                pkt["helpers"].append({
                    "name": name, "cite": cite(cur),
                    "comment": preceding_comment(cur),
                    "body": lines})
            pkt["markers"].append({
                "subject": name,
                "source": cite(cur)})

    for name in clo.order:
        d = clo.byval[name]
        b = badge(name, d)
        sz = (d.get_definition() or d).type.get_size()
        pkt["types_byval"].append({"name": name, "size": sz, "badge": b,
                                   "cite": cite(d)})
        if "UNPORTED" in b:
            pkt["markers"].append({"subject": name, "source": cite(d)})
    for name, d in sorted(clo.ptronly.items()):
        b = badge(name, d)
        pkt["types_ptronly"].append({"name": name, "badge": b, "cite": cite(d)})
        if "UNPORTED" in b:
            pkt["markers"].append({"subject": name, "source": cite(d)})

    for name, cur in sorted(globals_.items()):
        sc = cur.storage_class
        pkt["globals"].append({
            "name": name,
            "type": cur.type.spelling,
            "storage": "static" if sc == StorageClass.STATIC else "extern/global",
            "cite": cite(cur)})

    return pkt


# ------------------------------------------------------------- rendering
def fence(lines, lang="c"):
    out = [f"```{lang}"]
    out += [f"{n:>5} | {t}" for n, t in lines]
    out.append("```")
    return "\n".join(out)


def render_md(pkt):
    o = []
    o.append(f"# PORT PACKET: {pkt['target']} ({pkt['module']})")
    o.append(f"- kind: function")
    o.append(f"- signature: `{pkt['signature']}`")
    o.append(f"- source: `{pkt['cite']}`")

    o.append("\n## Raven comment")
    o.append(pkt["raven_comment"] or "(none)")

    o.append("\n## Suggested Rust doc header")
    hdr = [f"/// Raven `{pkt['target']}` — <fill: one-line description>.", "///"]
    if pkt["raven_comment"]:
        first = pkt["raven_comment"].splitlines()[0]
        hdr.append(f"/// Raven: {first}")
    hdr.append(f"/// Source: `{pkt['cite']}`")
    o.append("```rust\n" + "\n".join(hdr) + "\n```")

    o.append(f"\n## Body (verbatim, `{pkt['cite']}`)")
    o.append(fence(pkt["body"]))

    o.append(f"\n## Callees ({len(pkt['callees'])})")
    for c in pkt["callees"]:
        loc = f" — `{c['cite']}`" if c["cite"] else ""
        o.append(f"- `{c['name']}`{loc}  [{c['kind']}]")

    o.append(f"\n## Inlined helpers ({len(pkt['helpers'])}) — in-module callees "
             f"small enough to read here")
    for h in pkt["helpers"]:
        o.append(f"\n### {h['name']} (`{h['cite']}`)")
        if h["comment"]:
            o.append(f"Raven: {h['comment'].splitlines()[0]}")
        o.append(fence(h["body"]))

    o.append(f"\n## Types — by value ({len(pkt['types_byval'])}), port order")
    for t in pkt["types_byval"]:
        o.append(f"- `{t['name']}` ({t['size']}B) {t['badge']} — `{t['cite']}`")
    o.append(f"\n## Types — pointer-only ({len(pkt['types_ptronly'])}), opaque ok")
    for t in pkt["types_ptronly"]:
        o.append(f"- `{t['name']}` {t['badge']} — `{t['cite']}`")

    o.append(f"\n## Globals referenced ({len(pkt['globals'])}) — thread as owned "
             f"state, do not port as Rust globals")
    for g in pkt["globals"]:
        o.append(f"- `{g['name']}`: `{g['type']}` [{g['storage']}] — `{g['cite']}`")

    o.append(f"\n## Syscalls required ({len(pkt['syscalls'])}) — the ABI seam "
             f"this function crosses")
    for s in pkt["syscalls"]:
        o.append(f"- `{s['name']}` — `{s['cite']}`")

    o.append(f"\n## Paste-ready markers for unported deps ({len(pkt['markers'])})")
    o.append("```rust")
    for m in pkt["markers"]:
        o.append(f"//TODO: Port {m['subject']}")
        o.append(f"// Source: {m['source']}")
    o.append("```")
    return "\n".join(o)


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("module")
    ap.add_argument("function")
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--helper-cap", type=int, default=15,
                    help="inline in-module callee bodies up to N lines (default 15)")
    ap.add_argument("--source", choices=sorted(C.PROFILES), default="raven")
    ap.add_argument("--root", help="source tree root (for --source openjk)")
    args = ap.parse_args()

    C.MODULES = C.PROFILES[args.source]
    if args.root:
        C.SRC_ROOT = Path(args.root).resolve()
    elif args.source != "raven":
        sys.exit(f"--source {args.source} needs --root <path-to-tree>")
    if args.module not in C.MODULES:
        sys.exit(f"module '{args.module}' not in profile '{args.source}'")

    pkt = build_packet(args.module, args.function, args.helper_cap)
    if args.json:
        pkt["body"] = [{"line": n, "text": t} for n, t in pkt["body"]]
        for h in pkt["helpers"]:
            h["body"] = [{"line": n, "text": t} for n, t in h["body"]]
        print(json.dumps(pkt, indent=2))
    else:
        print(render_md(pkt))


if __name__ == "__main__":
    main()
