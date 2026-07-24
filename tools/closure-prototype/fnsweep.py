#!/usr/bin/env python3
"""PROTOTYPE — throwaway. Batch FUNCTION sweep / manifest generator for the
logic port (the piece CLAUDE.md names as missing next to closure.py +
portpacket.py).

One unity libclang parse of a whole game module -> a JSON manifest describing
EVERY function definition: signature, LOC, callee buckets, function-scope
statics, global reads/writes, function-pointer dispatch writes, plus a stats
summary (histogram, top-N, fn-ptr field census, Tarjan SCCs of the in-module
call graph, SCC-condensed topological wave structure).

Usage:
  .venv/bin/python fnsweep.py [--module mp-game] [--out DIR]
                              [--source raven|openjk] [--root PATH]

Companion to closure.py (shared module profiles, unity parse, ported badges)
and portpacket.py (per-function packet). fnskel.py consumes this manifest.
"""
import argparse
import json
import sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import closure as C
from clang.cindex import CursorKind, StorageClass, TypeKind

RECURSE_KINDS = {CursorKind.UNEXPOSED_DECL, CursorKind.LINKAGE_SPEC,
                 CursorKind.NAMESPACE}
ASSIGN_OPS = {"=", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "<<=", ">>="}
# lvalue-transparent wrappers we descend through to find the rooted decl
LVALUE_WRAP = {CursorKind.MEMBER_REF_EXPR, CursorKind.ARRAY_SUBSCRIPT_EXPR,
               CursorKind.PAREN_EXPR, CursorKind.UNEXPOSED_EXPR,
               CursorKind.CSTYLE_CAST_EXPR, CursorKind.UNARY_OPERATOR}
WRAP_EXPR = {CursorKind.PAREN_EXPR, CursorKind.UNEXPOSED_EXPR,
             CursorKind.CSTYLE_CAST_EXPR}


def basename(cur):
    f = (cur.get_definition() or cur).location.file
    return Path(f.name).name if f else ""


# ------------------------------------------------- module-parameterized filter
# A module profile decides two things fnsweep must not hardwire: which .c files
# are PORT TARGETS (collected as call-graph nodes), and how a callee's defining
# file BUCKETS (in-module vs an already-satisfied dependency vs the trap seam).
#
# mp-game (default) is unchanged: every `/game/*.c` is a port target — including
# bg_*.c and g_syscalls.c — and bg_*.c callees bucket as "bg" purely by name.
#
# mp-ui adds two exclusions that are satisfied DEPENDENCIES, not port targets:
#   (a) codemp/game/bg_*.c — already ported as mp_bg; calls resolve to "bg".
#   (b) codemp/ui/ui_syscalls.c — the trap seam (owned by crates/mp/abi/src/ui);
#       every fn it defines (trap_* wrappers + PASSFLOAT/dllEntry) buckets as
#       "syscall" so it never enters the in-module port graph.
MODULE_FILTERS = {
    "mp-game": dict(port_dir="/game/", seam_files=set()),
    "mp-ui": dict(port_dir="/ui/", seam_files={"ui_syscalls.c"}),
}
# Active filter — set by build_manifest; defaults keep the mp-game path identical.
_FILTER = MODULE_FILTERS["mp-game"]


def is_port_target_c(cur):
    """True if `cur` is defined in a .c file this module ports (a call-graph
    node). Non-port dependency TUs (bg_*.c, ui_syscalls.c) are excluded here and
    surface only as classified callees."""
    f = cur.location.file
    if not f:
        return False
    p = f.name.replace("\\", "/")
    if not (p.endswith(".c") and _FILTER["port_dir"] in p):
        return False
    return Path(p).name not in _FILTER["seam_files"]


def is_file_scope_var(cur):
    return (cur.kind == CursorKind.VAR_DECL and cur.semantic_parent is not None
            and cur.semantic_parent.kind == CursorKind.TRANSLATION_UNIT)


def cite(cur):
    defn = cur.get_definition() or cur
    ext = defn.extent
    path = C.loc(defn).rsplit(":", 1)[0]
    a, b = ext.start.line, ext.end.line
    return f"{path}:{a}" if a == b else f"{path}:{a}-{b}"


# --------------------------------------------------------- callee bucketing
def classify_callee(callee):
    """One of: syscall | bg | in-module | libc/other.

    `trap_*` and any fn defined in a module `seam_file` (ui_syscalls.c) bucket as
    "syscall" (the trap seam); bg_*.c bodies bucket as "bg" (satisfied dep); all
    other in-tree bodies are "in-module" port targets."""
    name = callee.spelling
    if name.startswith("trap_"):
        return "syscall"
    defn = callee.get_definition()
    if defn is not None:
        f = defn.location.file
        if f and f.name.startswith(str(C.SRC_ROOT)):
            bn = basename(defn)
            if bn.startswith("bg_"):
                return "bg"
            if bn in _FILTER["seam_files"]:
                return "syscall"
            return "in-module"
        return "libc/other"
    # no body in the TU: engine import or libc/SDK prototype
    return "libc/other"


# ------------------------------------------- lvalue / rvalue root resolution
def root_var(cur):
    """Descend an lvalue expr to the file-scope VAR_DECL it is rooted in;
    return (name, cite) or None. `level.foo`, `g_entities[i].health`, etc."""
    c = cur
    for _ in range(64):
        if c is None:
            return None
        if c.kind == CursorKind.DECL_REF_EXPR:
            ref = c.referenced
            if ref is not None and is_file_scope_var(ref):
                return ref.spelling, cite(ref)
            return None
        if c.kind in LVALUE_WRAP:
            kids = list(c.get_children())
            c = kids[0] if kids else None
            continue
        return None
    return None


def rhs_function(cur):
    """Descend an rvalue expr to the FUNCTION_DECL it names (through &, casts,
    parens). Returns the function name or None."""
    c = cur
    for _ in range(64):
        if c is None:
            return None
        if c.kind == CursorKind.DECL_REF_EXPR:
            ref = c.referenced
            if ref is not None and ref.kind == CursorKind.FUNCTION_DECL:
                return ref.spelling
            return None
        if c.kind in (CursorKind.UNARY_OPERATOR, CursorKind.PAREN_EXPR,
                      CursorKind.UNEXPOSED_EXPR, CursorKind.CSTYLE_CAST_EXPR):
            kids = list(c.get_children())
            c = kids[0] if kids else None
            continue
        return None
    return None


def binop_assign_op(cur):
    """If cur is an assignment BINARY_OPERATOR, return its operator spelling
    ('='/'+='/...), else None. Operator token sits after the first operand."""
    kids = list(cur.get_children())
    if len(kids) != 2:
        return None
    end0 = kids[0].extent.end
    for t in cur.get_tokens():
        loc = t.location
        if (loc.line, loc.column) >= (end0.line, end0.column) \
                and t.spelling in ASSIGN_OPS:
            return t.spelling
    return None


def lhs_field(cur):
    """Name of the lvalue being assigned: the outermost struct field for
    `ent->think` -> 'think', or the variable name for a bare `NPC_PainFunc = Fn`
    fn-pointer variable. Returns the name or None."""
    if cur.kind == CursorKind.MEMBER_REF_EXPR:
        return cur.spelling
    if cur.kind == CursorKind.DECL_REF_EXPR:
        return cur.spelling
    # peel transparent wrappers that can sit above the member (casts/parens)
    for c in (cur.get_children() if cur.kind in WRAP_EXPR else []):
        r = lhs_field(c)
        if r:
            return r
    return None


def field_is_fnptr(cur):
    """True if the lvalue's static type is a function pointer."""
    t = cur.type
    if t is None:
        return False
    canon = t.get_canonical()
    return (canon.kind == TypeKind.POINTER
            and canon.get_pointee().kind == TypeKind.FUNCTIONPROTO)


# ----------------------------------------------------- per-function analysis
def analyze_fn(fn):
    fn = fn.get_definition() or fn
    ext = fn.extent
    info = {
        "name": fn.spelling,
        "file": basename(fn),
        "line": ext.start.line,
        "end_line": ext.end.line,
        "loc": ext.end.line - ext.start.line + 1,
        "usr": fn.get_usr(),
        "static": fn.storage_class == StorageClass.STATIC,
        "ret_type": fn.result_type.spelling,
        "params": [{"name": a.spelling, "type": a.type.spelling}
                   for a in fn.get_arguments()],
        # FUNCTIONNOPROTO (old-style `void f()` with no prototype) has no
        # variadic query — ui carries a few; treat them as non-variadic.
        "variadic": (fn.type.kind == TypeKind.FUNCTIONPROTO
                     and fn.type.is_function_variadic()),
        "callees": {"in-module": [], "bg": [], "syscall": [], "libc/other": []},
        "callee_usrs": [],          # in-module + bg targets, for the call graph
        "statics": [],              # function-scope static VAR_DECLs
        "globals_read": [],
        "globals_write": [],
        "fnptr_writes": [],         # {field, target, fnptr}
    }
    seen_callee = defaultdict(int)
    reads, writes = {}, {}

    def note_assignment(node):
        op = binop_assign_op(node)
        if op is None:
            return
        kids = list(node.get_children())
        lhs = kids[0]
        rv = root_var(lhs)
        if rv is not None:
            writes[rv[0]] = rv[1]
            if op != "=":                       # compound assign also reads
                reads.setdefault(rv[0], rv[1])
        # function-pointer dispatch write: field = &Fn / field = Fn
        fnname = rhs_function(kids[1])
        if fnname is not None and (field_is_fnptr(lhs) or lhs_field(lhs)):
            info["fnptr_writes"].append({
                "field": lhs_field(lhs) or "?",
                "target": fnname,
                "is_fnptr_typed": field_is_fnptr(lhs)})

    def walk(cur):
        k = cur.kind
        if k == CursorKind.CALL_EXPR and cur.referenced is not None \
                and cur.referenced.kind == CursorKind.FUNCTION_DECL:
            nm = cur.referenced.spelling
            if not nm.startswith("__builtin"):
                seen_callee[cur.referenced.get_usr() or nm] = cur.referenced
        elif k == CursorKind.VAR_DECL and cur.storage_class == StorageClass.STATIC:
            info["statics"].append({"name": cur.spelling,
                                    "type": cur.type.spelling})
        elif k == CursorKind.DECL_REF_EXPR and cur.referenced is not None \
                and is_file_scope_var(cur.referenced):
            reads.setdefault(cur.referenced.spelling, cite(cur.referenced))
        elif k == CursorKind.BINARY_OPERATOR:
            note_assignment(cur)
        elif k in (CursorKind.COMPOUND_ASSIGNMENT_OPERATOR,):
            note_assignment(cur)
        elif k == CursorKind.UNARY_OPERATOR:
            toks = [t.spelling for t in cur.get_tokens()]
            if "++" in toks or "--" in toks:
                kids = list(cur.get_children())
                if kids:
                    rv = root_var(kids[0])
                    if rv is not None:
                        writes[rv[0]] = rv[1]
        for c in cur.get_children():
            walk(c)

    walk(fn)

    for usr, callee in seen_callee.items():
        bucket = classify_callee(callee)
        entry = {"name": callee.spelling, "cite":
                 cite(callee) if bucket in ("in-module", "bg", "syscall") else None}
        info["callees"][bucket].append(entry)
        if bucket in ("in-module", "bg"):
            tgt = callee.get_definition()
            if tgt is not None:
                info["callee_usrs"].append(tgt.get_usr())

    # globals that are written are recorded as writes; keep pure-read separate
    info["globals_write"] = [{"name": n, "cite": c} for n, c in sorted(writes.items())]
    info["globals_read"] = [{"name": n, "cite": c} for n, c in sorted(reads.items())
                            if n not in writes]
    for b in info["callees"].values():
        b.sort(key=lambda e: e["name"])
    return info


# --------------------------------------------------------------- collection
def collect_functions(tu):
    out = []
    seen = set()

    def visit(cur):
        for c in cur.get_children():
            if c.kind == CursorKind.FUNCTION_DECL and c.is_definition() \
                    and is_port_target_c(c):
                usr = c.get_usr()
                if usr in seen:
                    continue
                seen.add(usr)
                out.append(c)
            elif c.kind in RECURSE_KINDS:
                visit(c)
    visit(tu.cursor)
    return out


# ------------------------------------------------------------- Tarjan SCC
def tarjan_scc(nodes, edges):
    """nodes: list of ids. edges: id -> set(id). Returns list of SCCs (each a
    list of ids), in reverse-topological order (a callee-SCC before its
    caller-SCC — i.e. deps first)."""
    index = {}
    low = {}
    onstack = {}
    stack = []
    result = []
    counter = [0]

    import sys as _sys
    _sys.setrecursionlimit(1 << 20)

    def strongconnect(v):
        work = [(v, iter(edges.get(v, ())))]
        index[v] = low[v] = counter[0]
        counter[0] += 1
        stack.append(v)
        onstack[v] = True
        while work:
            node, it = work[-1]
            advanced = False
            for w in it:
                if w not in index:
                    index[w] = low[w] = counter[0]
                    counter[0] += 1
                    stack.append(w)
                    onstack[w] = True
                    work.append((w, iter(edges.get(w, ()))))
                    advanced = True
                    break
                elif onstack.get(w):
                    low[node] = min(low[node], index[w])
            if advanced:
                continue
            work.pop()
            if work:
                parent = work[-1][0]
                low[parent] = min(low[parent], low[node])
            if low[node] == index[node]:
                comp = []
                while True:
                    w = stack.pop()
                    onstack[w] = False
                    comp.append(w)
                    if w == node:
                        break
                result.append(comp)

    for v in nodes:
        if v not in index:
            strongconnect(v)
    return result


def build_call_graph(funcs):
    """Return (nodes, edges) over in-module+bg function definitions, keyed by
    USR (handles same-named statics in different files)."""
    nodes = [f["usr"] for f in funcs]
    valid = set(nodes)
    edges = defaultdict(set)
    for f in funcs:
        for u in f["callee_usrs"]:
            if u in valid and u != f["usr"]:
                edges[f["usr"]].add(u)
    return nodes, edges


def condensed_waves(sccs, edges):
    """SCC-condensed topological waves: wave(scc) = 1 + max wave over callee
    SCCs (leaves = wave 0 = call nothing in-module). Mirrors the type port's
    'deps first' levels, but over the SCC condensation (call graphs cycle)."""
    comp_of = {}
    for i, comp in enumerate(sccs):
        for n in comp:
            comp_of[n] = i
    cedges = defaultdict(set)  # comp -> callee comps
    for src, dsts in edges.items():
        cs = comp_of[src]
        for d in dsts:
            cd = comp_of[d]
            if cs != cd:
                cedges[cs].add(cd)
    wave = {}

    def w(ci):
        if ci in wave:
            return wave[ci]
        wave[ci] = 0  # break any residual cycle defensively
        best = 0
        for cd in cedges.get(ci, ()):
            best = max(best, w(cd) + 1)
        wave[ci] = best
        return best

    for ci in range(len(sccs)):
        w(ci)
    return wave, comp_of


# ------------------------------------------------------------------- main
def build_manifest(module):
    global _FILTER
    _FILTER = MODULE_FILTERS.get(module, MODULE_FILTERS["mp-game"])
    tu = C.parse_tu(module, None, unity=True)
    diags = [d for d in tu.diagnostics if d.severity >= 3]
    parse_notes = defaultdict(int)
    for d in diags:
        f = d.location.file
        key = f"{Path(f.name).name}:{d.spelling[:60]}" if f else d.spelling[:60]
        parse_notes[key] += 1

    cursors = collect_functions(tu)
    funcs = [analyze_fn(c) for c in cursors]
    funcs.sort(key=lambda f: (f["file"], f["line"]))

    # global fn-ptr target set: any function stored into a fn pointer anywhere
    stored_targets = set()
    for f in funcs:
        for w in f["fnptr_writes"]:
            stored_targets.add(w["target"])
    for f in funcs:
        f["stored_as_fnptr"] = f["name"] in stored_targets

    # ---- call graph, SCCs, waves
    nodes, edges = build_call_graph(funcs)
    sccs = tarjan_scc(nodes, edges)
    wave, comp_of = condensed_waves(sccs, edges)
    usr_to_name = {f["usr"]: f["name"] for f in funcs}
    for f in funcs:
        ci = comp_of[f["usr"]]
        f["scc"] = ci
        f["wave"] = wave[ci]

    # ---- stats
    by_file = defaultdict(lambda: {"functions": 0, "loc": 0})
    total_loc = 0
    for f in funcs:
        by_file[f["file"]]["functions"] += 1
        by_file[f["file"]]["loc"] += f["loc"]
        total_loc += f["loc"]

    field_census = defaultdict(lambda: {"count": 0, "targets": set()})
    for f in funcs:
        for w in f["fnptr_writes"]:
            fc = field_census[w["field"]]
            fc["count"] += 1
            fc["targets"].add(w["target"])

    top20 = sorted(funcs, key=lambda f: -f["loc"])[:20]

    nontrivial_sccs = [c for c in sccs if len(c) > 1]
    wave_hist = defaultdict(int)
    for ci in range(len(sccs)):
        wave_hist[wave[ci]] += 1

    # callee-edge census over ALL call sites (how the outgoing surface resolves):
    # in-module port targets vs already-satisfied bg vs the trap seam vs libc.
    callee_census = defaultdict(int)
    callee_distinct = {"in-module": set(), "bg": set(), "syscall": set(),
                       "libc/other": set()}
    for f in funcs:
        for bucket, entries in f["callees"].items():
            callee_census[bucket] += len(entries)
            for e in entries:
                callee_distinct[bucket].add(e["name"])

    lbl = {"mp-game": "jampgame", "mp-ui": "ui"}.get(module, module)
    srcdesc = {"mp-ui": "oracle/codemp/ui/*.c (port targets; bg_*.c + "
               "ui_syscalls.c parsed as satisfied deps/seam)"}.get(
                   module, "oracle/codemp/game/*.c")

    stats = {
        "module": module,
        "label": lbl,
        "srcdesc": srcdesc,
        "callee_census": {
            "edges": {k: callee_census[k] for k in
                      ("in-module", "bg", "syscall", "libc/other")},
            "distinct": {k: len(callee_distinct[k]) for k in
                         ("in-module", "bg", "syscall", "libc/other")},
        },
        "total_functions": len(funcs),
        "total_loc": total_loc,
        "files_with_functions": len(by_file),
        "histogram_by_file": {k: v for k, v in sorted(
            by_file.items(), key=lambda kv: -kv[1]["functions"])},
        "top20_largest": [{"name": f["name"], "file": f["file"],
                           "line": f["line"], "loc": f["loc"]} for f in top20],
        "fnptr_field_census": {
            k: {"count": v["count"], "distinct_targets": len(v["targets"]),
                "targets": sorted(v["targets"])}
            for k, v in sorted(field_census.items(), key=lambda kv: -kv[1]["count"])},
        "call_graph": {
            "nodes": len(nodes),
            "edges": sum(len(v) for v in edges.values()),
            "scc_count": len(sccs),
            "nontrivial_scc_count": len(nontrivial_sccs),
            "largest_scc": max((len(c) for c in sccs), default=0),
            "nontrivial_sccs": sorted(
                ([usr_to_name.get(n, n) for n in c] for c in nontrivial_sccs),
                key=lambda names: -len(names)),
            "wave_count": (max(wave.values()) + 1) if wave else 0,
            "wave_histogram": {str(k): wave_hist[k] for k in sorted(wave_hist)},
        },
        "parse_notes": dict(sorted(parse_notes.items(), key=lambda kv: -kv[1])),
        "parse_error_total": len(diags),
    }

    return {"functions": funcs, "stats": stats}


def render_stats_md(manifest, skel_samples):
    s = manifest["stats"]
    cg = s["call_graph"]
    label = s.get("label", "jampgame")
    srcdesc = s.get("srcdesc", "oracle/codemp/game/*.c")
    o = []
    o.append(f"# {label} function manifest — stats ({s['module']})\n")
    o.append(f"Generated by `tools/closure-prototype/fnsweep.py` from a single "
             f"unity libclang parse of `{srcdesc}`.\n")
    o.append("## Headline\n")
    o.append(f"- **Functions:** {s['total_functions']}")
    o.append(f"- **Total LOC (function extents):** {s['total_loc']:,}")
    o.append(f"- **Files with functions:** {s['files_with_functions']}")
    o.append(f"- **Call graph:** {cg['nodes']} nodes, {cg['edges']} in-module/bg edges")
    o.append(f"- **SCCs:** {cg['scc_count']} "
             f"({cg['nontrivial_scc_count']} non-trivial, largest = {cg['largest_scc']} fns)")
    o.append(f"- **Topological waves (SCC-condensed, deps-first):** {cg['wave_count']}\n")

    cc = s.get("callee_census")
    if cc:
        o.append("## Callee resolution census\n")
        o.append("Every call site from a port-target fn, bucketed by where the "
                 "callee resolves. `bg` = already ported (mp_bg); `syscall` = trap "
                 "seam (crates/mp/abi/src/ui, incl. ui_syscalls.c wrappers); "
                 "`in-module` = ui port targets; `libc/other` = C runtime / engine "
                 "prototypes with no body in the TU.\n")
        o.append("| bucket | call-site edges | distinct callees |")
        o.append("| --- | ---: | ---: |")
        for k in ("in-module", "bg", "syscall", "libc/other"):
            o.append(f"| {k} | {cc['edges'][k]} | {cc['distinct'][k]} |")
        o.append("")

    o.append("## Wave structure (SCC-condensed)\n")
    o.append("Wave 0 = calls nothing in-module (leaves); wave N = 1 + max wave "
             "of its in-module/bg callees. Count is number of SCCs per wave.\n")
    o.append("| wave | SCCs |")
    o.append("| ---: | ---: |")
    for k, v in cg["wave_histogram"].items():
        o.append(f"| {k} | {v} |")
    o.append("")

    o.append("## Non-trivial SCCs (mutual recursion — must port together)\n")
    if not cg["nontrivial_sccs"]:
        o.append("_None._\n")
    else:
        for comp in cg["nontrivial_sccs"][:30]:
            o.append(f"- ({len(comp)}) {', '.join(sorted(comp))}")
        if len(cg["nontrivial_sccs"]) > 30:
            o.append(f"- … +{len(cg['nontrivial_sccs']) - 30} more")
        o.append("")

    o.append("## Function-pointer dispatch field census\n")
    o.append("Assignments `x->FIELD = Fn` inside function bodies — the "
             "think/touch/die/... dispatch surface that needs fn-pointer "
             "resolution in the logic port.\n")
    o.append("| field | writes | distinct targets |")
    o.append("| --- | ---: | ---: |")
    for k, v in s["fnptr_field_census"].items():
        o.append(f"| `{k}` | {v['count']} | {v['distinct_targets']} |")
    o.append("")

    o.append("## Top-20 largest functions\n")
    o.append("| LOC | function | file:line |")
    o.append("| ---: | --- | --- |")
    for f in s["top20_largest"]:
        o.append(f"| {f['loc']} | `{f['name']}` | {f['file']}:{f['line']} |")
    o.append("")

    o.append("## Histogram by file\n")
    o.append("| file | functions | LOC |")
    o.append("| --- | ---: | ---: |")
    for k, v in s["histogram_by_file"].items():
        o.append(f"| {k} | {v['functions']} | {v['loc']} |")
    o.append("")

    o.append("## Parse notes\n")
    o.append(f"Total clang diagnostics (severity>=error): **{s['parse_error_total']}**. "
             "These are MSVC-isms in shared headers (repeated once per included "
             ".c), not per-function failures — function bodies and callees "
             "extract cleanly (spot-checked). Top clusters:\n")
    o.append("| count | file:diagnostic |")
    o.append("| ---: | --- |")
    for k, v in list(s["parse_notes"].items())[:15]:
        o.append(f"| {v} | {k} |")
    o.append("")

    if skel_samples:
        o.append("## Sample skeleton renderings (from fnskel.py)\n")
        o.append("Three files eyeballed for shape — small, medium, large:\n")
        for label, path, text in skel_samples:
            o.append(f"### {label}: `{path}`\n")
            o.append("```rust")
            o.append(text)
            o.append("```\n")

    return "\n".join(o)


def build_wave_partition(manifest):
    """Leaves-first topological wave partition over the in-module call graph.
    Each wave is a list of SCC groups; a mutual-recursion SCC (>1 fn) is one
    atomic group that must port together. Waves are exactly the SCC-condensed
    levels the manifest already stamped on every fn (wave 0 = calls nothing
    in-module)."""
    funcs = manifest["functions"]
    by_scc = defaultdict(list)
    scc_wave = {}
    for f in funcs:
        by_scc[f["scc"]].append(
            {"name": f["name"], "file": f["file"], "line": f["line"],
             "loc": f["loc"]})
        scc_wave[f["scc"]] = f["wave"]
    waves = defaultdict(list)
    for scc, members in by_scc.items():
        members.sort(key=lambda m: (m["file"], m["line"]))
        waves[scc_wave[scc]].append({
            "scc": scc, "size": len(members),
            "mutual_recursion": len(members) > 1, "fns": members})
    out_waves = []
    for w in sorted(waves):
        groups = sorted(waves[w], key=lambda g: (-g["size"], g["fns"][0]["name"]))
        out_waves.append({
            "wave": w,
            "scc_groups": len(groups),
            "fns": sum(g["size"] for g in groups),
            "loc": sum(m["loc"] for g in groups for m in g["fns"]),
            "mutual_recursion_groups": sum(1 for g in groups if g["mutual_recursion"]),
            "groups": groups})
    return {
        "module": manifest["stats"]["module"],
        "wave_count": len(out_waves),
        "note": "Leaves-first topological waves over the in-module call graph "
                "(bg/trap/libc callees excluded — they are satisfied deps). "
                "wave 0 = calls nothing in-module; a fn enters wave N only when "
                "every in-module callee sits in a lower wave. Each mutual-"
                "recursion SCC (>1 fn) is one atomic group.",
        "waves": out_waves}


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--module", default="mp-game")
    ap.add_argument("--out", default=str(Path(__file__).resolve().parent / "out"))
    ap.add_argument("--source", choices=sorted(C.PROFILES), default="raven")
    ap.add_argument("--root")
    args = ap.parse_args()

    C.MODULES = C.PROFILES[args.source]
    if args.root:
        C.SRC_ROOT = Path(args.root).resolve()
    elif args.source != "raven":
        sys.exit(f"--source {args.source} needs --root")
    if args.module not in C.MODULES:
        sys.exit(f"module '{args.module}' not in profile '{args.source}'")

    manifest = build_manifest(args.module)
    outdir = Path(args.out)
    outdir.mkdir(parents=True, exist_ok=True)
    prefix = manifest["stats"]["label"]
    mpath = outdir / f"{prefix}-fn-manifest.json"
    mpath.write_text(json.dumps(manifest, indent=1))
    spath = outdir / f"{prefix}-fn-stats.md"
    spath.write_text(render_stats_md(manifest, skel_samples=None))
    wpath = outdir / f"{prefix}-wave-partition.json"
    wpath.write_text(json.dumps(build_wave_partition(manifest), indent=1))
    cc = manifest["stats"]["callee_census"]["edges"]
    print(f"[fnsweep] {manifest['stats']['total_functions']} functions, "
          f"{manifest['stats']['total_loc']:,} LOC")
    print(f"[fnsweep] callee edges: in-module={cc['in-module']} bg={cc['bg']} "
          f"syscall={cc['syscall']} libc/other={cc['libc/other']}")
    print(f"[fnsweep] wrote {mpath}")
    print(f"[fnsweep] wrote {spath}")
    print(f"[fnsweep] wrote {wpath}")


if __name__ == "__main__":
    main()
