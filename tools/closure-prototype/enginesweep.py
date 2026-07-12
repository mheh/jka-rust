#!/usr/bin/env python3
"""PROTOTYPE — throwaway. DEDICATED-SERVER ENGINE sweep (Part-1 deliverable of
the mp-engine build-out plan). Sibling of fnsweep.py, but scoped to the whole
codemp engine (qcommon/server/ghoul2/botlib/icarus/RMG + the model-loading
renderer files WinDed links) instead of a single game module.

Differences from fnsweep.py that the engine forced:
  * collects CXX_METHOD / CONSTRUCTOR / DESTRUCTOR too — icarus/ghoul2/RMG put
    their logic in classes; FUNCTION_DECL-only misses ~all of RMG (a finding).
  * per-subsystem breakdown (functions, LOC, statics, globals, SCC/waves) keyed
    off the source directory, plus a cross-subsystem CALL MATRIX (who calls
    whom) — this determines safe port order across tier boundaries.
  * file-scope global-variable census (mutable non-const TU-scope VAR_DECLs =
    shared engine state to thread) on top of fnsweep's function-scope statics.

Reuses closure.py (profile + unity parse + ported badges) and fnsweep.py
(analyze_fn, tarjan_scc, condensed_waves). One unity libclang parse of the
mp-engine-ded profile.

Usage: .venv/bin/python enginesweep.py [--module mp-engine-ded] [--out DIR]
"""
import argparse
import json
import sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import closure as C
import fnsweep as F
from clang.cindex import CursorKind, StorageClass

FN_KINDS = {CursorKind.FUNCTION_DECL, CursorKind.CXX_METHOD,
            CursorKind.CONSTRUCTOR, CursorKind.DESTRUCTOR,
            CursorKind.FUNCTION_TEMPLATE, CursorKind.CONVERSION_FUNCTION}
CONTAINER_KINDS = {CursorKind.NAMESPACE, CursorKind.UNEXPOSED_DECL,
                   CursorKind.LINKAGE_SPEC, CursorKind.CLASS_DECL,
                   CursorKind.STRUCT_DECL, CursorKind.CLASS_TEMPLATE}
SUBSYSTEMS = ["qcommon", "server", "ghoul2", "botlib", "icarus", "RMG",
              "renderer"]


def subsystem_of(path: str) -> str:
    p = path.replace("\\", "/")
    for s in SUBSYSTEMS:
        if f"/{s}/" in p:
            return s
    return "?"


def in_scope(cur, allowed: set) -> bool:
    f = cur.location.file
    if not f:
        return False
    return str(Path(f.name).resolve()) in allowed


def srcglob_files(module) -> set:
    cfg = C.MODULES[module]
    out = set()
    for g in cfg.get("srcglob", []):
        for p in C.SRC_ROOT.glob(g):
            out.add(str(p.resolve()))
    return out


# ------------------------------------------------------------- collection
def collect(tu, allowed):
    out, seen = [], set()

    def visit(cur):
        for c in cur.get_children():
            if c.kind in FN_KINDS and c.is_definition() and in_scope(c, allowed):
                usr = c.get_usr()
                if usr in seen:
                    continue
                seen.add(usr)
                out.append(c)
            if c.kind in CONTAINER_KINDS or c.kind in FN_KINDS:
                # methods can nest (local classes); classes hold methods
                visit(c)
    visit(tu.cursor)
    return out


def collect_globals(tu, allowed):
    """File-scope (TU-scope) VAR_DECLs in engine sources: the shared mutable
    engine state that a port must thread. Static + extern + const classified."""
    out, seen = [], set()

    def visit(cur):
        for c in cur.get_children():
            if c.kind == CursorKind.VAR_DECL and F.is_file_scope_var(c) \
                    and in_scope(c, allowed):
                key = c.get_usr() or (c.spelling, F.basename(c))
                if key in seen:
                    continue
                seen.add(key)
                t = c.type
                out.append({
                    "name": c.spelling, "type": t.spelling,
                    "file": F.basename(c),
                    "subsystem": subsystem_of(c.location.file.name),
                    "static": c.storage_class == StorageClass.STATIC,
                    "extern": c.storage_class == StorageClass.EXTERN,
                    "const": t.is_const_qualified(),
                    "size": t.get_size() if t.get_size() > 0 else None,
                })
            if c.kind in (CursorKind.NAMESPACE, CursorKind.UNEXPOSED_DECL,
                          CursorKind.LINKAGE_SPEC):
                visit(c)
    visit(tu.cursor)
    return out


# ------------------------------------------------------------ per-fn record
def analyze(cur):
    info = F.analyze_fn(cur)
    info["subsystem"] = subsystem_of((cur.get_definition() or cur).location.file.name)
    # method owner (class::), for the C++ census
    sp = cur.semantic_parent
    info["owner"] = sp.spelling if sp and sp.kind in (
        CursorKind.CLASS_DECL, CursorKind.STRUCT_DECL,
        CursorKind.CLASS_TEMPLATE) else None
    info["is_method"] = cur.kind in (CursorKind.CXX_METHOD, CursorKind.CONSTRUCTOR,
                                     CursorKind.DESTRUCTOR, CursorKind.CONVERSION_FUNCTION)
    return info


# ---------------------------------------------------------- graph helpers
def scc_over(subset_usrs, edges):
    """Tarjan + wave over the subgraph induced by subset_usrs (intra-subset
    edges only)."""
    sub = set(subset_usrs)
    sedges = defaultdict(set)
    for s in sub:
        for d in edges.get(s, ()):
            if d in sub and d != s:
                sedges[s].add(d)
    sccs = F.tarjan_scc(list(sub), sedges)
    wave, comp_of = F.condensed_waves(sccs, sedges)
    return sccs, wave


# ------------------------------------------------------------------ build
def build(module):
    tu = C.parse_tu(module, None, unity=True)
    allowed = srcglob_files(module)
    diags = [d for d in tu.diagnostics if d.severity >= 3]
    parse_notes = defaultdict(int)
    for d in diags:
        f = d.location.file
        key = f"{Path(f.name).name}:{d.spelling[:60]}" if f else d.spelling[:60]
        parse_notes[key] += 1

    cursors = collect(tu, allowed)
    funcs = [analyze(c) for c in cursors]
    funcs.sort(key=lambda f: (f["subsystem"], f["file"], f["line"]))
    globs = collect_globals(tu, allowed)

    # ---- global call graph (whole engine)
    nodes, edges = F.build_call_graph(funcs)
    sccs = F.tarjan_scc(nodes, edges)
    wave, comp_of = F.condensed_waves(sccs, edges)
    usr_to = {f["usr"]: f for f in funcs}
    for f in funcs:
        ci = comp_of[f["usr"]]
        f["scc"], f["wave"] = ci, wave[ci]

    # ---- cross-subsystem call matrix (edge src-subsystem -> dst-subsystem)
    matrix = defaultdict(lambda: defaultdict(int))
    for f in funcs:
        src = f["subsystem"]
        for u in f["callee_usrs"]:
            g = usr_to.get(u)
            if g:
                matrix[src][g["subsystem"]] += 1

    # ---- per-subsystem stats
    per = {}
    for s in SUBSYSTEMS + ["?"]:
        sub = [f for f in funcs if f["subsystem"] == s]
        if not sub:
            continue
        sub_usrs = [f["usr"] for f in sub]
        s_sccs, s_wave = scc_over(sub_usrs, edges)
        nontriv = [c for c in s_sccs if len(c) > 1]
        wave_hist = defaultdict(int)
        for c in s_sccs:
            # wave keyed by comp index in s_wave
            pass
        # recompute wave hist over the induced subgraph
        comp_of_s = {}
        for i, comp in enumerate(s_sccs):
            for n in comp:
                comp_of_s[n] = i
        wh = defaultdict(int)
        for i in range(len(s_sccs)):
            wh[s_wave[i]] += 1
        loc = sum(f["loc"] for f in sub)
        statics = sum(len(f["statics"]) for f in sub)
        static_fns = [f for f in sub if f["statics"]]
        gwrites = sum(len(f["globals_write"]) for f in sub)
        methods = sum(1 for f in sub if f["is_method"])
        files = sorted({f["file"] for f in sub})
        loc_hist = defaultdict(int)
        for f in sub:
            b = ("1" if f["loc"] <= 10 else "11-30" if f["loc"] <= 30
                 else "31-80" if f["loc"] <= 80 else "81-200" if f["loc"] <= 200
                 else "200+")
            loc_hist[b] += 1
        per[s] = {
            "functions": len(sub), "methods": methods, "loc": loc,
            "files": len(files),
            "loc_histogram": {k: loc_hist[k] for k in
                              ["1", "11-30", "31-80", "81-200", "200+"] if loc_hist[k]},
            "function_scope_statics": statics,
            "functions_with_statics": len(static_fns),
            "global_writes": gwrites,
            "scc_count": len(s_sccs),
            "nontrivial_scc_count": len(nontriv),
            "largest_scc": max((len(c) for c in s_sccs), default=0),
            "wave_count": (max(s_wave.values()) + 1) if s_wave else 0,
            "wave_histogram": {str(k): wh[k] for k in sorted(wh)},
            "top_functions": [{"name": f["name"], "owner": f["owner"],
                               "file": f["file"], "line": f["line"], "loc": f["loc"]}
                              for f in sorted(sub, key=lambda f: -f["loc"])[:8]],
        }

    # ---- fn-ptr dispatch-table census (x->field = Fn ; the engine's API tables)
    field_census = defaultdict(lambda: {"count": 0, "targets": set(), "subsys": set()})
    for f in funcs:
        for w in f["fnptr_writes"]:
            fc = field_census[w["field"]]
            fc["count"] += 1
            fc["targets"].add(w["target"])
            fc["subsys"].add(f["subsystem"])

    # ---- static / global census (engine-wide)
    all_statics = []
    for f in funcs:
        for st in f["statics"]:
            all_statics.append({"fn": f["name"], "subsystem": f["subsystem"],
                                "file": f["file"], "name": st["name"],
                                "type": st["type"]})
    mut_globals = [g for g in globs if not g["const"] and not g["extern"]]

    nontriv_g = [c for c in sccs if len(c) > 1]
    stats = {
        "module": module,
        "total_functions": len(funcs),
        "total_methods": sum(1 for f in funcs if f["is_method"]),
        "total_loc": sum(f["loc"] for f in funcs),
        "total_function_scope_statics": len(all_statics),
        "total_file_scope_globals": len(globs),
        "total_mutable_globals": len(mut_globals),
        "per_subsystem": per,
        "cross_subsystem_matrix": {s: dict(d) for s, d in matrix.items()},
        "global_call_graph": {
            "nodes": len(nodes), "edges": sum(len(v) for v in edges.values()),
            "scc_count": len(sccs), "nontrivial_scc_count": len(nontriv_g),
            "largest_scc": max((len(c) for c in sccs), default=0),
            "wave_count": (max(wave.values()) + 1) if wave else 0,
        },
        "fnptr_dispatch_census": {
            k: {"writes": v["count"], "distinct_targets": len(v["targets"]),
                "subsystems": sorted(v["subsys"]), "targets": sorted(v["targets"])}
            for k, v in sorted(field_census.items(), key=lambda kv: -kv[1]["count"])},
        "parse_error_total": len(diags),
        "parse_notes": dict(sorted(parse_notes.items(), key=lambda kv: -kv[1])[:25]),
    }
    dispatch_sites = [{"subsystem": f["subsystem"], "name": f["name"],
                       "file": f["file"], "n": len(f["fnptr_writes"])}
                      for f in funcs if len(f["fnptr_writes"]) >= 3]
    return {"functions": funcs, "globals": globs, "statics_census": all_statics,
            "_dispatch_sites": dispatch_sites, "stats": stats}


# ------------------------------------------------------------------ render
def bar(n, mx, width=24):
    return "#" * max(1, round(n / mx * width)) if n else ""


def render_md(man):
    s = man["stats"]
    o = []
    o.append("# mp-engine (DEDICATED server) — function sweep stats\n")
    o.append(f"Generated by `tools/closure-prototype/enginesweep.py` from one "
             f"unity libclang parse of the `mp-engine-ded` profile "
             f"(WinDed.vcproj Release compile set: "
             f"qcommon+server+ghoul2+botlib+icarus+RMG + the 9 model-loading "
             f"renderer sources, `-DDEDICATED -DBOTLIB`).\n")

    o.append("## Headline\n")
    o.append(f"- **Functions + methods:** {s['total_functions']} "
             f"({s['total_methods']} C++ methods)")
    o.append(f"- **Total LOC (definition extents):** {s['total_loc']:,}")
    o.append(f"- **Function-scope statics** (determinism/reentrancy hazards): "
             f"{s['total_function_scope_statics']}")
    o.append(f"- **File-scope globals:** {s['total_file_scope_globals']} "
             f"({s['total_mutable_globals']} mutable non-extern = shared state to thread)")
    cg = s["global_call_graph"]
    o.append(f"- **Whole-engine call graph:** {cg['nodes']} nodes, {cg['edges']} edges, "
             f"{cg['scc_count']} SCCs ({cg['nontrivial_scc_count']} non-trivial, "
             f"largest = {cg['largest_scc']}), {cg['wave_count']} topological waves")
    o.append(f"- **Parse diagnostics (severity>=error):** {s['parse_error_total']} "
             f"(benign MSVC-isms / unity redefinitions; bodies+callees extract — "
             f"see parse notes)\n")

    o.append("## Per-subsystem breakdown\n")
    o.append("| subsystem | fns | methods | LOC | files | fn-statics | mut writes | "
             "SCCs | largest SCC | waves |")
    o.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for name, p in man["stats"]["per_subsystem"].items():
        o.append(f"| **{name}** | {p['functions']} | {p['methods']} | {p['loc']:,} | "
                 f"{p['files']} | {p['function_scope_statics']} | {p['global_writes']} | "
                 f"{p['scc_count']} | {p['largest_scc']} | {p['wave_count']} |")
    o.append("")

    o.append("### LOC histograms + wave shape, per subsystem\n")
    for name, p in man["stats"]["per_subsystem"].items():
        o.append(f"**{name}** — LOC buckets: " +
                 ", ".join(f"{k}:{v}" for k, v in p["loc_histogram"].items()) +
                 f" · waves(SCCs per wave): " +
                 ", ".join(f"w{k}:{v}" for k, v in p["wave_histogram"].items()) +
                 f" · nontrivial SCCs: {p['nontrivial_scc_count']}")
        tops = ", ".join(f"`{t['name']}`({t['loc']})" for t in p["top_functions"][:5])
        o.append(f"  · largest: {tops}\n")

    o.append("## Cross-subsystem call matrix\n")
    o.append("Rows = caller subsystem, columns = callee subsystem "
             "(count of resolved in-engine call edges). Leaf-most (calls few "
             "others) ports first; the matrix justifies tier boundaries.\n")
    cols = SUBSYSTEMS
    o.append("| caller \\ callee | " + " | ".join(cols) + " | (out) |")
    o.append("| --- | " + " | ".join("---:" for _ in cols) + " | ---: |")
    mat = man["stats"]["cross_subsystem_matrix"]
    for r in cols:
        row = mat.get(r, {})
        selfc = row.get(r, 0)
        outc = sum(v for k, v in row.items() if k != r)
        cells = []
        for c in cols:
            v = row.get(c, 0)
            cells.append(f"**{v}**" if c == r else (str(v) if v else "·"))
        o.append(f"| **{r}** | " + " | ".join(cells) + f" | {outc} |")
    o.append("")

    o.append("## Dispatch-table population sites (fn-ptr tables assembled here)\n")
    o.append("Functions that do many `field = Fn` assigns in one body = where an "
             "API vtable is built. These are the engine's indirect-dispatch "
             "seams: renderer refexport (`GetRefAPI`), botlib import/export "
             "(`GetBotLibAPI`/`Init_*_Export`/`SV_BotInitBotLib`), ICARUS game "
             "interface (`Interface_Init`). (VM dispatch is switch-based, not a "
             "table: `SV_GameSystemCalls` + the vmMain trampoline.)\n")
    o.append("| subsystem | function | file | fn-ptr assigns |")
    o.append("| --- | --- | --- | ---: |")
    pop = sorted(man["_dispatch_sites"], key=lambda x: -x["n"])[:15]
    for d in pop:
        o.append(f"| {d['subsystem']} | `{d['name']}` | {d['file']} | {d['n']} |")
    o.append("")

    o.append("## Function-pointer dispatch fields (individual writes)\n")
    o.append("`obj->FIELD = Fn` / `import.FIELD = Fn` writes — the engine's "
             "indirect-dispatch surface (renderer refexport/refimport, botlib "
             "import/export, VM entry, filesystem/collision callbacks). Each is "
             "a seam that needs explicit fn-pointer resolution in the port.\n")
    o.append("| field | writes | distinct targets | subsystems |")
    o.append("| --- | ---: | ---: | --- |")
    for k, v in list(man["stats"]["fnptr_dispatch_census"].items())[:40]:
        o.append(f"| `{k}` | {v['writes']} | {v['distinct_targets']} | "
                 f"{', '.join(v['subsystems'])} |")
    o.append("")

    o.append("## Function-scope statics census (top hazards)\n")
    o.append("Each is a hidden persistent cell — a determinism/reentrancy "
             "hazard to port deliberately (thread through state, not `static`).\n")
    o.append("| subsystem | function | static | type |")
    o.append("| --- | --- | --- | --- |")
    for st in sorted(man["statics_census"],
                     key=lambda x: (x["subsystem"], x["fn"]))[:60]:
        o.append(f"| {st['subsystem']} | `{st['fn']}` | `{st['name']}` | "
                 f"`{st['type']}` |")
    if len(man["statics_census"]) > 60:
        o.append(f"\n_… +{len(man['statics_census']) - 60} more (full list in "
                 f"manifest JSON)._")
    o.append("")

    o.append("## Largest mutable file-scope globals (shared engine state)\n")
    o.append("| subsystem | global | type | bytes |")
    o.append("| --- | --- | --- | ---: |")
    mut = [g for g in man["globals"] if not g["const"] and not g["extern"]]
    for g in sorted(mut, key=lambda x: -(x["size"] or 0))[:40]:
        o.append(f"| {g['subsystem']} | `{g['name']}` | `{g['type']}` | "
                 f"{g['size'] or '?'} |")
    o.append("")

    o.append("## Parse notes\n")
    o.append(f"Total diagnostics: **{s['parse_error_total']}**. Top clusters "
             "(benign — MSVC inline-asm in `SnapVector`, unity-TU header "
             "redefinitions across botlib .cpp, win32 identifiers):\n")
    o.append("| count | file:diagnostic |")
    o.append("| ---: | --- |")
    for k, v in list(s["parse_notes"].items())[:15]:
        o.append(f"| {v} | {k} |")
    o.append("")
    return "\n".join(o)


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--module", default="mp-engine-ded")
    ap.add_argument("--out", default=str(Path(__file__).resolve().parent / "out" / "engine"))
    args = ap.parse_args()
    man = build(args.module)
    outdir = Path(args.out)
    outdir.mkdir(parents=True, exist_ok=True)
    (outdir / "engine-fn-manifest.json").write_text(json.dumps(man, indent=1))
    (outdir / "engine-fn-stats.md").write_text(render_md(man))
    # per-subsystem stats slices for convenience
    for name, p in man["stats"]["per_subsystem"].items():
        (outdir / f"subsys-{name}.json").write_text(json.dumps(p, indent=1))
    s = man["stats"]
    print(f"[enginesweep] {s['total_functions']} fns ({s['total_methods']} methods), "
          f"{s['total_loc']:,} LOC, {s['total_function_scope_statics']} statics, "
          f"{s['total_file_scope_globals']} globals")
    print(f"[enginesweep] wrote {outdir}/engine-fn-manifest.json + engine-fn-stats.md")


if __name__ == "__main__":
    main()
