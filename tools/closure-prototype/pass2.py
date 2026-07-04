#!/usr/bin/env python3
"""PROTOTYPE — pass-2 needs-ctx generator (writes out/pass2/needs-ctx.json).

Per-fn ctx classification for the pass-3 packet generator. The base rule is the
fixpoint in `pass2lib.compute_needs_ctx` (a fn needs ctx iff its body references
`trap_*` / any file-scope global / a static, OR transitively calls a needs-ctx
fn). This generator ADDS the bg-tier global-disposition refinement so a bg fn is
not blindly tagged a `PmoveContext` method just because it reads a const table.

Global disposition (bg tier only matters, but computed for the whole TU):
  * `pmove`       — one of the 7 pmove working-set globals (`pm`, `pml`,
                    `pm_entSelf`, `pm_entVeh`, `pm_flying`, `gPMDoSlowFall`,
                    `pm_cancelOutZoom`). Lives on `PmoveContext`.
  * `state`       — mutable runtime state: written anywhere in the game/bg tree
                    (manifest `globals_write`), OR has no static initializer
                    (a runtime-parsed buffer like `SaberParms`/`bg_pool`), OR is
                    address-taken as a whole non-const object. Now on `BgState`.
  * `const-table` — file-scope, statically initialized (`= …` at decl), never
                    written, never non-const whole-object address-taken
                    (`bgForcePowerCost`, `bg_itemlist`, `saberMoveData`, …).
                    Raven leaves these non-`const`; we detect them by the
                    static-initializer + never-mutated signature, not by the C
                    `const` qualifier (Raven rarely uses it).

Per-fn bucket (heaviest wins; pmove > bgstate > none):
  own(f)  = pmove   if f touches a pmove global
          = bgstate if f touches a state global OR uses any `trap_`/`strap_`
          = none    otherwise (only const tables / nothing).
  bucket(f) = max(own(f), max(bucket(c) for c in bg/qshared callees of f)).
  Game-tier callees are NOT propagated: a bg fn reaches game state only through a
  `GameCallbacks` upcall, so a heavy game callee does not make the bg caller
  pmove/bgstate.

bg-tier ctx_kind (NEW, replaces the old flat `pmove-or-bg`):
  * bucket pmove   -> ctx_kind "pmove"   (impl PmoveContext method, unchanged)
  * bucket bgstate -> ctx_kind "bgstate" (free fn taking `bg: &mut BgState` /
                      `&BgState`, plus `&dyn BgTraps` when it hits a trap — NOT a
                      PmoveContext method)
  * bucket none    -> needs_ctx FALSE    (plain free fn)
game / qshared tiers keep the original fixpoint needs_ctx and their ctx_kind.

Schema is backward-compatible: `needs_ctx` / `why` / `ctx_kind` keep their names;
this adds `globals` (per-fn [name, disposition, r|w]) and, for bg fns, `bucket`.

Usage:  .venv/bin/python pass2.py
"""
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import closure as C
import pass2lib as L
from clang.cindex import CursorKind, StorageClass

OUT = L.HERE / "out" / "pass2"

# The pmove working set (fork rulings 12-14): these globals become `PmoveContext`
# fields; a bg fn that touches any of them is a PmoveContext method.
PMOVE = {"pm", "pml", "pm_entSelf", "pm_entVeh", "pm_flying",
         "gPMDoSlowFall", "pm_cancelOutZoom"}

RANK = {"none": 0, "bgstate": 1, "pmove": 2}
UNRANK = {v: k for k, v in RANK.items()}


# ---------------------------------------------------- global disposition (clang)
def decl_has_initializer(cur):
    """True iff the VAR_DECL carries a real `= …` initializer. Token scan at
    bracket/paren depth 0 so the array-size expr in `char buf[N]` is NOT counted
    as an initializer (that was a real bug — runtime buffers looked const)."""
    depth = 0
    for t in cur.get_tokens():
        s = t.spelling
        if s in "([{":
            depth += 1
        elif s in ")]}":
            depth -= 1
        elif depth == 0 and s == "=":
            return True
        elif depth == 0 and s == ";":
            break
    return False


def scan_globals(tu):
    """Return (has_init, whole_addr): name-keyed dicts over file-scope VAR_DECLs.
    `has_init[name]` prefers the defining decl over an `extern`. `whole_addr` is
    the set of non-const globals whose WHOLE object is address-taken (`&g`) — a
    conservative mutation signal the manifest's lvalue-store scan can miss."""
    has_init, whole_addr = {}, set()

    def is_tu_var(ref):
        return (ref is not None and ref.kind == CursorKind.VAR_DECL
                and ref.semantic_parent is not None
                and ref.semantic_parent.kind == CursorKind.TRANSLATION_UNIT)

    for cur in tu.cursor.walk_preorder():
        if cur.kind == CursorKind.VAR_DECL and cur.semantic_parent is not None \
                and cur.semantic_parent.kind == CursorKind.TRANSLATION_UNIT:
            hi = decl_has_initializer(cur)
            prev = has_init.get(cur.spelling)
            if prev is None or (hi and not prev):
                has_init[cur.spelling] = hi
        elif cur.kind == CursorKind.UNARY_OPERATOR:
            toks = [t.spelling for t in cur.get_tokens()]
            if toks and toks[0] == "&":
                kids = list(cur.get_children())
                if kids and kids[0].kind == CursorKind.DECL_REF_EXPR:
                    ref = kids[0].referenced
                    if is_tu_var(ref) and not ref.type.is_const_qualified():
                        whole_addr.add(ref.spelling)
    return has_init, whole_addr


def disposition_map(F, has_init, whole_addr):
    """name -> 'pmove' | 'state' | 'const-table'. Conservative: a global whose
    decl we never saw (extern-only in this TU) or that is written / uninitialised
    / whole-object address-taken is `state`."""
    written = set()
    for f in F:
        for g in f["globals_write"]:
            written.add(g["name"])
    referenced = set()
    for f in F:
        for g in f["globals_read"] + f["globals_write"]:
            referenced.add(g["name"])
    disp = {}
    for name in referenced:
        if name in PMOVE:
            disp[name] = "pmove"
        elif (name in written or name in whole_addr
              or not has_init.get(name, False)):
            disp[name] = "state"
        else:
            disp[name] = "const-table"
    return disp


# ------------------------------------------------------------- bucket fixpoint
def own_bucket(f, disp):
    touched = {g["name"] for g in f["globals_read"] + f["globals_write"]}
    if touched & PMOVE:
        return RANK["pmove"]
    has_trap = bool(f["callees"]["syscall"])
    if has_trap or any(disp.get(n) == "state" for n in touched):
        return RANK["bgstate"]
    return RANK["none"]


def _bg_qshared_edges(F):
    tier_of = {f["usr"]: L.tier(f["file"]) for f in F}
    return {f["usr"]: [u for u in f["callee_usrs"]
                       if tier_of.get(u) in ("bg", "qshared")] for f in F}


def compute_buckets(F, disp):
    """Fixpoint bucket over the bg+qshared call subgraph (game callees excluded —
    they are GameCallbacks upcalls, not inherited context)."""
    bucket = {f["usr"]: own_bucket(f, disp) for f in F}
    edges = _bg_qshared_edges(F)
    changed = True
    while changed:
        changed = False
        for u, outs in edges.items():
            hi = bucket[u]
            for c in outs:
                if bucket.get(c, 0) > hi:
                    hi = bucket[c]
            if hi != bucket[u]:
                bucket[u] = hi
                changed = True
    return bucket


def compute_state_mut(F, disp):
    """Fixpoint: a fn needs `&mut BgState` (vs `&BgState`) iff it, or any of its
    bg/qshared callees, WRITES a `state` global. A bgstate fn that only reads
    state directly but calls e.g. `BG_TempAlloc` (writes `bg_pool`) still needs
    `&mut`."""
    def own(f):
        return any(g["name"] and disp.get(g["name"]) == "state"
                   for g in f["globals_write"])
    mut = {f["usr"]: own(f) for f in F}
    edges = _bg_qshared_edges(F)
    changed = True
    while changed:
        changed = False
        for u, outs in edges.items():
            if not mut[u] and any(mut.get(c) for c in outs):
                mut[u] = True
                changed = True
    return mut


# ------------------------------------------------------------------------ main
def main():
    m = L.load_manifest()
    F = m["functions"]
    tu = C.parse_tu("mp-game", None, unity=True)
    has_init, whole_addr = scan_globals(tu)
    disp = disposition_map(F, has_init, whole_addr)
    bucket = compute_buckets(F, disp)
    state_mut = compute_state_mut(F, disp)

    # base fixpoint (game/qshared keep this; bg is overridden by bucket)
    need, seed = L.compute_needs_ctx(F)

    written = set()
    for f in F:
        for g in f["globals_write"]:
            written.add(g["name"])

    fns = []
    counts_bg = {"pmove": 0, "bgstate": 0, "none": 0}
    moved = {"pmove->bgstate": 0, "pmove->none": 0}
    for f in F:
        t = L.tier(f["file"])
        usr = f["usr"]
        why = []
        if f["callees"]["syscall"]:
            why.append("trap")
        if f["globals_read"]:
            why.append("global-read")
        if f["globals_write"]:
            why.append("global-write")
        seeded = usr in seed
        globals_list = []
        for g in f["globals_read"]:
            globals_list.append([g["name"], disp.get(g["name"], "state"), "r"])
        for g in f["globals_write"]:
            globals_list.append([g["name"], disp.get(g["name"], "state"), "w"])

        rec = {
            "name": f["name"], "file": f["file"], "tier": t,
            "seeded": seeded,
            "why": why if seeded else (["transitive"] if usr in need else []),
            "globals": globals_list,
        }
        if t == "bg":
            b = UNRANK[bucket[usr]]
            rec["needs_ctx"] = (b != "none")
            rec["bucket"] = b
            if b == "bgstate":
                rec["state_mut"] = bool(state_mut[usr])
            rec["ctx_kind"] = ("pmove" if b == "pmove"
                               else "bgstate" if b == "bgstate" else "none")
            counts_bg[b] += 1
            # summary: what the OLD flat rule ('pmove-or-bg' for any needs-ctx bg
            # fn) would have said vs the refined bucket.
            old_needs = usr in need
            if old_needs and b == "bgstate":
                moved["pmove->bgstate"] += 1
            elif old_needs and b == "none":
                moved["pmove->none"] += 1
        else:
            n = usr in need
            rec["needs_ctx"] = n
            rec["ctx_kind"] = ("game" if (n and t == "game")
                               else "rng-or-qshared" if (n and t == "qshared")
                               else "none")
        fns.append(rec)

    by_tier = {}
    for r in fns:
        if r["needs_ctx"]:
            by_tier[r["tier"]] = by_tier.get(r["tier"], 0) + 1

    out = {
        "generator": "tools/closure-prototype/pass2.py",
        "definition": (
            "needs_ctx iff body references trap_* OR file-scope globals/statics "
            "OR (transitively) calls a needs-ctx fn. Fixpoint over the "
            "in-module+bg call graph (cycles fine). bg tier is REFINED by global "
            "disposition: const-table-only bg fns are needs_ctx FALSE (plain free "
            "fn); state-touching bg fns are ctx_kind 'bgstate' (bg: &mut BgState "
            "+ &dyn BgTraps, NOT a PmoveContext method); pmove-touching bg fns are "
            "ctx_kind 'pmove' (PmoveContext method). Bucket transitivity: a caller "
            "inherits the heaviest bucket of its bg/qshared callees "
            "(pmove > bgstate > none)."),
        "counts": {
            "total_fns": len(fns),
            "needs_ctx": sum(1 for r in fns if r["needs_ctx"]),
            "seed": len(seed),
            "by_tier_needs_ctx": by_tier,
            "bg_bucket": counts_bg,
            "bg_moved_from_flat_pmove": moved,
        },
        "fns": fns,
    }
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "needs-ctx.json").write_text(json.dumps(out, indent=1))
    print(f"[pass2] wrote out/pass2/needs-ctx.json — {len(fns)} fns, "
          f"{out['counts']['needs_ctx']} needs_ctx")
    print(f"[pass2] bg buckets: pmove={counts_bg['pmove']} "
          f"bgstate={counts_bg['bgstate']} none={counts_bg['none']}")
    print(f"[pass2] bg fns moved off flat-pmove: "
          f"pmove->bgstate={moved['pmove->bgstate']} "
          f"pmove->none={moved['pmove->none']}")


if __name__ == "__main__":
    main()
