#!/usr/bin/env python3
"""PROTOTYPE — throwaway. Total dependency-ordered port progression for the
DEDICATED-SERVER ENGINE (mp-engine-ded profile): every function/method in
bottom-up topological order, so porting in emitted order NEVER needs a stub —
every callee (and every fn-ptr-table target) is already ported when a symbol's
turn comes. SCC groups (mutual recursion) are emitted as one unit, ported
together.

Fixes two graph defects in enginesweep.py:
  1. analyze_fn only records callees whose referenced decl is FUNCTION_DECL,
     silently dropping C++ method calls (all of icarus/RMG/ghoul2-class
     internals) and address-taken references (dispatch-table targets).
  2. The unity TU is lossy: botlib headers lack include guards (redefinition
     cascades break struct types across be_*.cpp) and whole statement runs in
     sv_game.cpp (`VMA(...)` casts) drop with no diagnostic. This script
     parses ONE TU PER SOURCE FILE (like the real build) and merges the graph
     by USR — the standalone sv_game.cpp parse resolves 687 call sites where
     the unity parse kept 164.

And two compile-set defects:
  3. The profile srcglob is not the WinDed link set: it sweeps in 13 never-
     linked files (xbox/console/ppc variants, files.cpp vs the linked
     files_common/files_pc pair, icarus Interpreter/Tokenizer, hstring,
     CNetProfile) and MISSES server/NPCNav/ (CNavigator — real syscall
     targets) and the null/ dedicated client-stub layer. This script derives
     its source list from WinDed.vcproj directly, minus the platform seam
     (win32/, null/win_main.cpp — the Rust host implements Sys_*/net natively)
     and vendored third-party code (zlib32/, png/ — supplied by Rust crates
     per the established vendored-code policy).
  4. msg.cpp's netField_t offset macros ((int)&((entityState_t*)0)->x) are
     hard errors on an LP64 host and silently gut the delta tables; parses run
     with -fms-compatibility, which downgrades them to the MSVC-era warnings
     they were.

Edge capture per function:
  * CALL_EXPR -> referenced FUNCTION_DECL / CXX_METHOD / CONSTRUCTOR /
    DESTRUCTOR / CONVERSION_FUNCTION (virtual calls resolve to the static
    target — the named method is the 1:1 port dependency);
  * DECL_REF_EXPR / MEMBER_REF_EXPR referencing a function = address taken
    (a dispatch-table target must exist when the table is built) — a "ref"
    edge, same ordering strength as a call edge;
  * unresolved callees (not defined in the engine compile set) bucketed as
    externals per function: libc/OS, std::, and the already-ported shared
    q_shared/q_math surface the Rust qshared crates supply.

Outputs (out/engine/):
  engine-port-order.json  full machine artifact: waves -> units -> functions,
                          each with calls/refs (names), externals, globals r/w,
                          statics; plus corrected cross-subsystem matrix.
  engine-port-order.tsv   one row per function in port order (seq, wave, unit,
                          subsystem, file, line, loc, symbol, deps, externals).
  engine-port-order.md    human summary: headline, corrected matrix, per-wave
                          tables of every symbol.

Usage: .venv/bin/python engineorder.py [--module mp-engine-ded] [--out DIR]
"""
import argparse
import json
import re
import sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import closure as C
import fnsweep as F
import enginesweep as E
from clang.cindex import CursorKind

FN_KINDS = E.FN_KINDS | {CursorKind.CONVERSION_FUNCTION}
REF_KINDS = {CursorKind.DECL_REF_EXPR, CursorKind.MEMBER_REF_EXPR}
# The active module's subsystem order for sorting and matrix columns.
# build() sets it from the spec (subsystems + order.extra_subsystems).
SUBSYS: list = []


def realcase(rel, base):
    """vcproj paths are case-sloppy (rmg/ vs RMG/); resolve each component
    against the on-disk name so subsystem_of's case-sensitive match works."""
    p = base
    for part in rel.split("/"):
        hit = next((c for c in p.iterdir() if c.name.lower() == part.lower()),
                   None)
        if hit is None:
            return None
        p = hit
    return p


def vcproj_sources(order):
    """The actual vcproj link set (repo paths, real case), minus the
    platform seam and vendored third-party code (spec exclude sets)."""
    vcproj = C.SRC_ROOT / order["vcproj"]
    excluded = order.get("exclude_platform", set()) \
        | order.get("exclude_vendored", set())
    txt = vcproj.read_text(encoding="latin-1")
    rels = re.findall(r'RelativePath="\.?\\?([^"]+\.(?:cpp|c))"', txt)
    out = []
    for r in sorted({p.replace("\\", "/") for p in rels}):
        if r in excluded:
            continue
        p = realcase(r, vcproj.parent)
        assert p is not None, f"vcproj source missing on disk: {r}"
        out.append(p.resolve())
    return out


def module_sources(module):
    """The per-file TU list: the spec's vcproj link set when the order block
    names one (mp-engine-ded), else the profile srcglob."""
    sp = C.spec(module)
    order = sp.get("order", {})
    if "vcproj" in order:
        return vcproj_sources(order)
    files = {p.resolve() for g in sp["srcglob"] for p in C.SRC_ROOT.glob(g)}
    return sorted(files)


def subsystem_of(path):
    s = E.subsystem_of(path)
    if s == "?" and "/null/" in path.replace("\\", "/"):
        return "null"
    return s


def qualname(cur):
    sp = cur.semantic_parent
    if sp is not None and sp.kind in (CursorKind.CLASS_DECL, CursorKind.STRUCT_DECL,
                                      CursorKind.CLASS_TEMPLATE):
        return f"{sp.spelling}::{cur.spelling}"
    return cur.spelling


def collect_edges_raw(fn_cursor):
    """(calls, refs) for one function definition, as {usr: name} — resolution
    against the merged engine definition set happens after all TUs parse."""
    calls, refs = {}, {}

    def walk(cur):
        for c in cur.get_children():
            ref = c.referenced
            if ref is not None and ref.kind in FN_KINDS:
                u = ref.get_usr()
                nm = ref.spelling
                if u and nm and not nm.startswith("__builtin"):
                    if c.kind == CursorKind.CALL_EXPR:
                        calls[u] = nm
                    elif c.kind in REF_KINDS:
                        refs[u] = nm
            walk(c)

    walk(fn_cursor)
    return calls, refs


def build(module):
    global SUBSYS
    E.configure(module)
    SUBSYS = E.SUBSYSTEMS + list(
        C.spec(module).get("order", {}).get("extra_subsystems", []))
    C.MODULES[module]["flags"] = list(C.MODULES[module].get("flags", [])) + \
        ["-fms-compatibility"]
    # q_shared.h only declares the byte-swap family inside platform sections
    # the profile deliberately doesn't enable; Raven's own WIN32 branch defines
    # them as empty macros (little-endian identity). Same trick for the parse —
    # otherwise every LittleLong call site is an undeclared-identifier error
    # and its statement drops (be_aas_file, tr_model, cm_load).
    C.MODULES[module]["defines"] = list(C.MODULES[module]["defines"]) + \
        ["LittleShort=", "LittleLong=", "LittleFloat=", "LittleLong64="]
    sources = module_sources(module)
    allowed = {str(p) for p in sources}
    rel_sources = sorted(str(p.relative_to(C.SRC_ROOT)) for p in sources)

    funcs = []
    raw_calls, raw_refs = {}, {}      # fn usr -> {callee usr: name}
    seen_usr = set()
    diag_files = defaultdict(int)
    for i, rel in enumerate(rel_sources, 1):
        tu = C.parse_tu(module, rel, unity=False)
        for d in tu.diagnostics:
            if d.severity >= 3:
                f = d.location.file
                diag_files[Path(f.name).name if f else "?"] += 1
        new = 0
        for cur in E.collect(tu, allowed):
            usr = cur.get_usr()
            if usr in seen_usr:
                continue
            seen_usr.add(usr)
            info = E.analyze(cur)
            info["subsystem"] = subsystem_of(cur.location.file.name)
            info["qualname"] = qualname(cur)
            raw_calls[usr], raw_refs[usr] = collect_edges_raw(cur)
            funcs.append(info)
            new += 1
        print(f"[engineorder] [{i}/{len(rel_sources)}] {rel}: +{new} fns",
              flush=True)
    usr_to = {f["usr"]: f for f in funcs}
    engine_usrs = set(usr_to)

    # ---- resolve edges against the merged definition set
    calls_of, refs_of = {}, {}
    for f in funcs:
        u = f["usr"]
        calls_of[u] = {d for d in raw_calls[u] if d in engine_usrs and d != u}
        refs_of[u] = {d for d in raw_refs[u] if d in engine_usrs and d != u}
        refs_of[u] -= calls_of[u]
        f["externals"] = sorted({nm for d, nm in raw_calls[u].items()
                                 if d not in engine_usrs})

    nodes = [f["usr"] for f in funcs]
    edges = defaultdict(set)
    for u in nodes:
        edges[u] = calls_of[u] | refs_of[u]
    n_call = sum(len(v) for v in calls_of.values())
    n_ref = sum(len(v) for v in refs_of.values())

    sccs = F.tarjan_scc(nodes, edges)
    wave, comp_of = F.condensed_waves(sccs, edges)

    # ---- corrected cross-subsystem matrix (call+ref edges)
    matrix = defaultdict(lambda: defaultdict(int))
    for u in nodes:
        src = usr_to[u]["subsystem"]
        for d in edges[u]:
            matrix[src][usr_to[d]["subsystem"]] += 1

    # ---- units: one per SCC, ordered by (wave, subsystem, file, line)
    def fn_sort_key(f):
        return (SUBSYS.index(f["subsystem"]) if f["subsystem"] in SUBSYS
                else 99, f["file"], f["line"])

    units = []
    for ci, comp in enumerate(sccs):
        members = sorted((usr_to[u] for u in comp), key=fn_sort_key)
        units.append({
            "wave": wave[ci],
            "cyclic": len(comp) > 1,
            "members": members,
        })
    units.sort(key=lambda u: (u["wave"], fn_sort_key(u["members"][0])))

    # ---- no-stub verification: every dependency is in an earlier wave, or in
    # the same SCC unit. This is the property that makes the order stub-free.
    unit_idx = {}
    seq = 0
    for i, u in enumerate(units):
        for f in u["members"]:
            unit_idx[f["usr"]] = i
            seq += 1
            f["seq"], f["unit"] = seq, i
            f["wave"] = u["wave"]
    violations = 0
    for u in nodes:
        for d in edges[u]:
            if unit_idx[d] != unit_idx[u] and \
                    units[unit_idx[d]]["wave"] >= units[unit_idx[u]]["wave"]:
                violations += 1
    assert violations == 0, f"{violations} ordering violations — order is NOT stub-free"

    # externals census (what the Rust engine must supply from outside the port:
    # libc/OS/std:: plus the already-ported q_shared/q_math surface)
    ext_census = defaultdict(lambda: {"count": 0, "subsys": set()})
    for f in funcs:
        for e in f["externals"]:
            ext_census[e]["count"] += 1
            ext_census[e]["subsys"].add(f["subsystem"])

    stats = {
        "module": module,
        "parse_mode": "per-file TUs, merged by USR",
        "source_files": len(rel_sources),
        "functions": len(funcs),
        "edges_total": sum(len(v) for v in edges.values()),
        "edges_call": n_call,
        "edges_ref": n_ref,
        "scc_count": len(sccs),
        "nontrivial_scc_count": sum(1 for c in sccs if len(c) > 1),
        "largest_scc": max((len(c) for c in sccs), default=0),
        "wave_count": (max(wave.values()) + 1) if wave else 0,
        "wave_histogram": {},
        "cross_subsystem_matrix": {s: dict(d) for s, d in matrix.items()},
        "distinct_externals": len(ext_census),
        "no_stub_verified": True,
        "diag_files_top": dict(sorted(diag_files.items(),
                                      key=lambda kv: -kv[1])[:15]),
    }
    wh = defaultdict(int)
    for u in units:
        wh[u["wave"]] += len(u["members"])
    stats["wave_histogram"] = {str(k): wh[k] for k in sorted(wh)}
    return funcs, units, edges, calls_of, refs_of, ext_census, stats, usr_to


# ------------------------------------------------------------------ render
def render(funcs, units, calls_of, refs_of, ext_census, stats, usr_to, outdir):
    name_of = {u: f["qualname"] for u, f in usr_to.items()}
    module = stats["module"]
    prefix = E.module_label(module)
    title = C.spec(module).get("order", {}).get("md_title", module)

    # ---- JSON
    jwaves = defaultdict(list)
    for i, u in enumerate(units):
        jwaves[u["wave"]].append({
            "unit": i,
            "cyclic": u["cyclic"],
            "functions": [{
                "seq": f["seq"], "symbol": f["qualname"],
                "subsystem": f["subsystem"], "file": f["file"],
                "line": f["line"], "end_line": f["end_line"], "loc": f["loc"],
                "calls": sorted(name_of[d] for d in calls_of[f["usr"]]),
                "refs": sorted(name_of[d] for d in refs_of[f["usr"]]),
                "externals": f["externals"],
                "globals_read": f["globals_read"],
                "globals_write": f["globals_write"],
                "statics": f["statics"],
            } for f in u["members"]],
        })
    (outdir / f"{prefix}-port-order.json").write_text(json.dumps({
        "stats": stats,
        "waves": [{"wave": w, "units": jwaves[w]} for w in sorted(jwaves)],
        "externals_census": {k: {"callers": v["count"],
                                 "subsystems": sorted(v["subsys"])}
                             for k, v in sorted(ext_census.items(),
                                                key=lambda kv: -kv[1]["count"])},
        **C.freshness_stamp(),
    }, indent=1))

    # ---- TSV (one row per function, in port order)
    rows = ["seq\twave\tunit\tcyclic\tsubsystem\tfile\tline\tloc\tsymbol\tdeps\texternals"]
    for i, u in enumerate(units):
        for f in u["members"]:
            deps = sorted(name_of[d] for d in
                          (calls_of[f["usr"]] | refs_of[f["usr"]]))
            rows.append("\t".join([
                str(f["seq"]), str(u["wave"]), str(i),
                "1" if u["cyclic"] else "0", f["subsystem"], f["file"],
                str(f["line"]), str(f["loc"]), f["qualname"],
                ";".join(deps), ";".join(f["externals"])]))
    (outdir / f"{prefix}-port-order.tsv").write_text("\n".join(rows) + "\n")

    # ---- MD
    o = []
    o.append(f"# {title} — total port progression\n")
    o.append("Generated by `tools/closure-prototype/engineorder.py` (per-file "
             f"libclang TUs of the `{module}` profile, merged by USR — the "
             "unity TU used by `enginesweep.py` silently drops statements; see "
             "the script docstring). Every function/method of the engine in "
             "bottom-up dependency order: port in `seq` order and **no symbol "
             "is ever referenced before it is ported** — no stubs, no "
             "deferrals. Cyclic units (mutual recursion) are ported together "
             "as one unit. Dependency edges = resolved calls **plus "
             "address-taken references** (dispatch-table targets), and C++ "
             "method calls are captured.\n")
    o.append("## Headline\n")
    o.append(f"- **Source files parsed:** {stats['source_files']}")
    o.append(f"- **Functions + methods:** {stats['functions']}")
    o.append(f"- **Dependency edges:** {stats['edges_total']} "
             f"({stats['edges_call']} call + {stats['edges_ref']} address-taken)")
    o.append(f"- **SCCs:** {stats['scc_count']} "
             f"({stats['nontrivial_scc_count']} cyclic, largest "
             f"{stats['largest_scc']}) → **{stats['wave_count']} waves**")
    o.append(f"- **Externals (supplied by Rust std/libc/qshared crates, not "
             f"ported here):** {stats['distinct_externals']} distinct names")
    o.append(f"- **No-stub property:** machine-verified — every edge lands in "
             f"an earlier wave or inside the same cyclic unit.\n")
    o.append("Wave sizes: " + ", ".join(
        f"w{k}:{v}" for k, v in stats["wave_histogram"].items()) + "\n")

    o.append("## Corrected cross-subsystem dependency matrix\n")
    o.append("Rows = dependent (caller/referrer), columns = dependency. "
             "Includes the method calls, address-taken references, and "
             "unity-dropped statements `enginesweep.py` missed.\n")
    cols = SUBSYS
    o.append("| from \\ on | " + " | ".join(cols) + " |")
    o.append("| --- | " + " | ".join("---:" for _ in cols) + " |")
    mat = stats["cross_subsystem_matrix"]
    for r in cols:
        row = mat.get(r, {})
        cells = [f"**{row.get(c, 0)}**" if c == r else
                 (str(row.get(c, 0)) if row.get(c, 0) else "·") for c in cols]
        o.append(f"| **{r}** | " + " | ".join(cells) + " |")
    o.append("")

    o.append("## Top externals (the seam the Rust engine supplies)\n")
    o.append("| external | call sites | subsystems |")
    o.append("| --- | ---: | --- |")
    for k, v in sorted(ext_census.items(), key=lambda kv: -kv[1]["count"])[:30]:
        o.append(f"| `{k}` | {v['count']} | {', '.join(sorted(v['subsys']))} |")
    o.append("")

    o.append("## The progression\n")
    o.append("One line per function, in port order. `deps` counts resolved "
             "in-engine dependencies (all already ported at that point); "
             "`ext` counts externals. Cyclic units are marked ⟳ and listed "
             "together. Full dependency lists: "
             "`engine-port-order.{json,tsv}`.\n")
    for w in sorted({u["wave"] for u in units}):
        wunits = [u for u in units if u["wave"] == w]
        nfn = sum(len(u["members"]) for u in wunits)
        o.append(f"### Wave {w} — {nfn} functions\n")
        o.append("| seq | subsystem | symbol | file:line | loc | deps | ext |")
        o.append("| ---: | --- | --- | --- | ---: | ---: | ---: |")
        for u in wunits:
            mark = " ⟳" if u["cyclic"] else ""
            for f in u["members"]:
                nd = len(calls_of[f["usr"]] | refs_of[f["usr"]])
                o.append(f"| {f['seq']} | {f['subsystem']} | "
                         f"`{f['qualname']}`{mark} | {f['file']}:{f['line']} | "
                         f"{f['loc']} | {nd} | {len(f['externals'])} |")
        o.append("")
    (outdir / f"{prefix}-port-order.md").write_text("\n".join(o))


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--module", default="mp-engine-ded")
    ap.add_argument("--out", default=None,
                    help="default: out/<label> for the module")
    args = ap.parse_args()
    funcs, units, edges, calls_of, refs_of, ext_census, stats, usr_to = build(args.module)
    prefix = E.module_label(args.module)
    outdir = Path(args.out) if args.out else (
        Path(__file__).resolve().parent / "out" / prefix)
    outdir.mkdir(parents=True, exist_ok=True)
    render(funcs, units, calls_of, refs_of, ext_census, stats, usr_to, outdir)
    print(f"[engineorder] {stats['functions']} fns, {stats['edges_total']} edges "
          f"({stats['edges_call']} call + {stats['edges_ref']} ref), "
          f"{stats['wave_count']} waves, largest SCC {stats['largest_scc']}, "
          f"no-stub verified")
    print(f"[engineorder] wrote {outdir}/{prefix}-port-order.{{json,tsv,md}}")


if __name__ == "__main__":
    main()
