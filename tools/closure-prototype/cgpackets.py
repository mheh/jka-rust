#!/usr/bin/env python3
"""PROTOTYPE — throwaway. cgame logic-port packets, sibling of uipackets.py
(same packets3.py-derived machinery — one work order per file-per-wave slice
of the module), retargeted from ui to cgame for the client-port C0 stage
(DEC-45). Diff vs uipackets.py is confined to module retargeting; see the
inline notes at each divergence point for a side-by-side read.

The cgame port is WAVE-GATED the same way (client-port scoping doc): leaves
first, a fn enters a wave only when every in-module callee sits in a lower
wave. Packets are sliced per (wave, file) and sharded (LOC-balanced
contiguous fn ranges) via the packets3 policy — identical to uipackets.py.

Each packet carries, per the U0/C0 packet-generator agenda:
 (a) the FILE-SCOPE CONSTANTS a fn's own oracle slice omits (renderer-plan
     wave-1 gap fix, ported here — see `file_scope_constants` below: R3 lost
     ~10 fns to "constant not in my slice" guesses before this section
     existed; baking it in from cgame's first wave closes that gap here);
 (b) a THREADING DIGEST PER FN naming the state channel against the RATIFIED
     root-type bindings (CgWorld / CgContext / DisplayContext / MenuSystem —
     see the CONFIG block; ratified at the C2 root-type sit-down as DEC-46,
     2026-07-28) — cgpackets now matches uipackets.py, which regenerated
     post-ratification with DEC-36 bindings baked in as fact;
 (c) the cited verbatim oracle source slice of each fn + its oracle C signature;
 (d) resolved signatures of out-of-module callees — bg fns from crates/mp/bg
     (LAW, ported), ui_shared.c fns from crates/mp/uishared (LAW, ported — the
     THIRD satisfied-dependency kind cgame has that ui didn't: ui_shared.c
     rides cgame's srcglob as a callee body per closure.py's mp-cgame profile,
     already ported as its own crate for cgame+ui reuse), and trap wrappers
     via crates/mp/abi/src/cgame (the syscall ABI token + the cg_syscalls.c C
     contract; the ERGONOMIC `trap_*` wrapper existence is checked per name);
 (e) a STATICS-TO-FOLD list per fn — function-scope statics + file-scope
     globals it touches (fold into CgWorld / subsystem fields at C2).

Emits out/cgame/packets/ + out/cgame/packets-manifest.json.

Usage:
  .venv/bin/python cgpackets.py                   # all waves, all files
  .venv/bin/python cgpackets.py --wave 0          # only wave-0 packets
  .venv/bin/python cgpackets.py --only cg_main.c
  .venv/bin/python cgpackets.py --wave 0 --only cg_main.c   # union
"""
import argparse
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from pass2lib import scan_rs_file

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
ORACLE = REPO / "oracle"
CGDIR = ORACLE / "codemp" / "cgame"
BG_SRC = REPO / "crates" / "mp" / "bg" / "src"
# cgame-specific: ui_shared.c rides cgame's srcglob as a satisfied dependency
# (closure.py mp-cgame profile) — already ported in its own crate, shared with
# ui (scoping.md's MenuSystem). uipackets.py has no counterpart resolver: for
# the ui module ui_shared.c was itself a port target, not a dependency.
UISHARED_SRC = REPO / "crates" / "mp" / "uishared" / "src"
ABI_CGAME = REPO / "crates" / "mp" / "abi" / "src" / "cgame"
CG_SCOPING = REPO / "docs" / "plans" / "2026-07-24-client-port" / "scoping.md"
MANIFEST = HERE / "out" / "cgame" / "cgame-fn-manifest.json"
OUT = HERE / "out" / "cgame"

# packets3's shard caps — a (wave, file) slice shards past either.
SHARD_MAX_FNS = 35
SHARD_MAX_LOC = 3000

# =============================================================================
# CONFIG — root-type bindings, RATIFIED at the C2 sit-down as DEC-46
# (docs/decisions.md, 2026-07-28). Unlike the pre-C2 draft this replaces,
# these names are FINALIZED law transcribers must follow; a shape question
# DEC-46 doesn't cover is a real escalation, never an invention here.
#
#   CgWorld        — the owned cgame state spine: cg: CgState (cg_t) +
#                    cgs: CgsState (cgs_t) + entities: Box<[CEntity;
#                    MAX_GENTITIES]> (DEC-46.1), Raven field names preserved
#                    line-for-line (cg.time -> world.cg.time).
#   CgContext      — the threaded handle the vmMain entrypoints own and pass
#                    inward ({ world: &mut CgWorld, engine, … }). Analog of
#                    GameContext/UiContext; DEC-46 did not revisit this name.
#   DisplayContext — the ui-plan's same ~50-fn-pointer render/text/cvar/
#                    feeder/ownerDraw/sound vtable (Raven displayContextDef_t)
#                    cgame implements too (scoping.md: MenuSystem is generic
#                    over this trait, one host impl per module).
#   MenuSystem     — ui_shared.c's owned menu framework, ALREADY PORTED as
#                    mp_uishared (not placeholder — cited via the uishared
#                    resolver below, not this CONFIG block). Named here only
#                    so the threading digest can say "CgWorld.menus".
CG_WORLD = "CgWorld"
CG_CONTEXT = "CgContext"
DISPLAY_CONTEXT = "DisplayContext"
MENU_SYSTEM = "MenuSystem"
CONFIG_FINALIZED = True  # DEC-46 ratified the C2 root-type sit-down 2026-07-28
# =============================================================================

# bg_lib.c-origin callees that are NOT mp_bg fns — Raven's own libc shims. They
# resolve to native/std, not a bg signature (noted, never MISSING-WRAPPER).
# Identical to uipackets.py's table — cgame reaches the same shims through bg.
BG_LIB_SHIMS = {
    "atof": "Rust `str::parse::<f32>()` / the `Q_atof` helper",
    "atoi": "Rust `str::parse::<i32>()` / the `Q_atoi` helper",
    "qsort": "`native_sort` (DEC-34 canonical qsort)",
    "rand": "`BgState.rng` (never libc `rand` — parity-visible; cgame reaches "
            "the one generator through its bg/callback channel)",
}


# --------------------------------------------------------- source-slice helpers
_SRC = {}


def numbered_slice(cfile, a, b):
    lines = _SRC.get(cfile)
    if lines is None:
        lines = (CGDIR / cfile).read_text(errors="replace").splitlines()
        _SRC[cfile] = lines
    a = max(1, a); b = min(len(lines), b)
    return "\n".join(f"{n:>5} | {lines[n - 1]}" for n in range(a, b + 1))


def strip_c_comments(src):
    src = re.sub(r"/\*.*?\*/", " ", src, flags=re.S)
    src = re.sub(r"//[^\n]*", " ", src)
    return src


def oracle_c_sig(f):
    """The fn's oracle C signature, one line, from the manifest record."""
    params = ", ".join(f"{p['type']} {p['name']}".strip() for p in f["params"]) \
        or "void"
    star = "" if f["ret_type"].endswith("*") else " "
    return f"{f['ret_type']}{star}{f['name']}({params});"


# ------------------------------------------------------ out-of-module resolvers
def load_bg_sigs():
    """name -> (file, rust_sig) for every fn ported in crates/mp/bg."""
    sigs = {}
    for p in sorted(BG_SRC.rglob("*.rs")):
        fns, _ = scan_rs_file(p)
        for r in fns:
            if r["name"] in sigs:
                continue
            params = re.sub(r"\s+", " ", r["params"]).strip()
            ret = r["ret"].strip()
            recv = ""
            if r["is_method"]:
                recv = f"impl {r['impl_ty'] or '?'} :: "
            sigs[r["name"]] = (p.name,
                               f"{recv}pub fn {r['name']}({params})"
                               f"{(' ' + ret) if ret else ''}".rstrip())
    return sigs


def load_uishared_sigs():
    """name -> (file, rust_sig) for every fn ported in crates/mp/uishared —
    cgame's ui_shared.c satisfied dependency (mirrors load_bg_sigs)."""
    sigs = {}
    for p in sorted(UISHARED_SRC.rglob("*.rs")):
        fns, _ = scan_rs_file(p)
        for r in fns:
            if r["name"] in sigs:
                continue
            params = re.sub(r"\s+", " ", r["params"]).strip()
            ret = r["ret"].strip()
            recv = ""
            if r["is_method"]:
                recv = f"impl {r['impl_ty'] or '?'} :: "
            sigs[r["name"]] = (p.name,
                               f"{recv}pub fn {r['name']}({params})"
                               f"{(' ' + ret) if ret else ''}".rstrip())
    return sigs


def load_trap_seam():
    """(c_sig, enum, tokens): trap_* -> its cg_syscalls.c C contract and the
    CG_* syscall enum it dispatches; plus the set of abi syscall tokens that
    already exist under crates/mp/abi/src/cgame/syscalls/."""
    txt = strip_c_comments((CGDIR / "cg_syscalls.c").read_text(errors="replace"))
    c_sig, enum = {}, {}
    defre = re.compile(
        r"([A-Za-z_][\w\*\s]*?)\b(trap_[A-Za-z0-9_]+)\s*\(([^;{]*?)\)\s*\{", re.S)
    for m in defre.finditer(txt):
        ret = re.sub(r"\s+", " ", m.group(1)).strip()
        name = m.group(2)
        args = re.sub(r"\s+", " ", m.group(3)).strip() or "void"
        star = "" if ret.endswith("*") else " "
        c_sig[name] = f"{ret}{star}{name}({args});"
        em = re.search(r"syscall\w*\(\s*([A-Z0-9_]+)", txt[m.end():m.end() + 400])
        if em:
            enum[name] = em.group(1)
    tokens = {p.stem for p in (ABI_CGAME / "syscalls").glob("CG_*.rs")}
    return c_sig, enum, tokens


# ------------------------------------------------------- file-scope constants
# Renderer R3 wave-1 gap fix, ported here from rendererpackets.py (baked in
# from cgame's FIRST wave rather than discovered after ~10 false deferrals,
# as happened there — see rendererpackets.py's own note for the fn list that
# cost). A packet previously carried only each fn's own oracle slice, never
# the file-scope `#define`s / const tables a few lines above it in the same
# TU; transcribers stubbed fns on "constant not in my slice". This pulls
# every such define/table into the packet, verbatim with oracle line numbers.
_INCLUDE_GUARD_RE = re.compile(r'^[A-Za-z_][A-Za-z0-9_]*_H_?$')
_DEFINE_RE = re.compile(r'^\s*#\s*define\s+([A-Za-z_]\w*)')
_TABLE_START_RE = re.compile(
    r'^static\s+(?:const\s+)?[\w:<>]+(?:\s*\*+)?\s+\w+'
    r'(?:\s*\[[^\]]*\])*\s*=\s*\{')
_TABLE_MAX_LOC = 40


def file_scope_constants(cfile):
    """[(start_line, end_line), ...] — every file-scope `#define` (continuation
    lines included) and small `static const`/`static ...[] = {...}` table in
    one cgame TU, in source order. Identical heuristic to
    rendererpackets.file_scope_constants: a `#define` at any indent whose
    macro name doesn't look like an include guard is a constant; a
    `static ... = {` line starting at COLUMN 0 (this codebase always indents
    fn-local statics) opens a table, brace-matched to its closing `};` and
    capped at ~40 LOC — a longer block is real data, not a constant, and is
    skipped (not truncated, so nothing half-verbatim ships)."""
    ls = numbered_lines(cfile)
    n = len(ls)
    entries = []
    i = 0
    while i < n:
        line = ls[i]
        m = _DEFINE_RE.match(line)
        if m:
            if _INCLUDE_GUARD_RE.match(m.group(1)):
                i += 1
                continue
            start = i
            while ls[i].rstrip().endswith("\\") and i + 1 < n:
                i += 1
            entries.append((start + 1, i + 1))
            i += 1
            continue
        if _TABLE_START_RE.match(line):
            start = i
            depth = line.count("{") - line.count("}")
            j = i
            while depth > 0 and j + 1 < n:
                j += 1
                depth += ls[j].count("{") - ls[j].count("}")
            if depth == 0 and (j - start + 1) <= _TABLE_MAX_LOC:
                entries.append((start + 1, j + 1))
            i = j + 1
            continue
        i += 1
    return entries


def numbered_lines(cfile):
    lines = _SRC.get(cfile)
    if lines is None:
        lines = (CGDIR / cfile).read_text(errors="replace").splitlines()
        _SRC[cfile] = lines
    return lines


def render_file_scope_constants(cfile):
    """The packet's `## FILE-SCOPE CONSTANTS` section, or `None` if the TU has
    none. Verbatim, oracle-line-numbered — IN-PACKET, closing the "constant
    not in my slice" deferral gap (rendererpackets.py precedent)."""
    entries = file_scope_constants(cfile)
    if not entries:
        return None
    o = [
        "## FILE-SCOPE CONSTANTS (verbatim)",
        "",
        f"Every file-scope `#define` and small `static const` table in "
        f"`{cfile}` (oracle line numbers below). **These are IN-PACKET** — a "
        "constant listed here is not a deferral excuse; use it directly. A "
        "constant a fn below needs that is NEITHER here NOR in that fn's own "
        "oracle slice still follows the never-guess rule (§A2/porting-rules): "
        "leave a cited `// DEFERRED:`, never invent a value.",
        "",
        "```c",
    ]
    for a, b in entries:
        o.append(numbered_slice(cfile, a, b))
        o.append("")
    if o and o[-1] == "":
        o.pop()
    o.append("```")
    o.append("")
    return "\n".join(o)


# ----------------------------------------------------------- scoping-doc law
def plan_digest():
    """The "Owned-state designs" + "bg reuse" sections, sliced verbatim from
    the 2026-07-24 scoping doc — the only settled cgame design law at C0
    (there is no cgame-plan.md/marker-law doc yet; that lands at C2). uipackets
    pulls its MARKER LAW from a frozen ratified plan section; cgame has no
    such section yet, so this cites the scoping doc's PRE-sit-down material
    instead and the packet frames it as placeholder (CONFIG_FINALIZED)."""
    t = CG_SCOPING.read_text()
    a = t.index("## Owned-state designs")
    b = t.index("## ABI surface")
    return t[a:b].rstrip()


# ------------------------------------------------------------- sharding (packets3)
def shard_chunks(fns):
    """Contiguous, LOC-balanced shards; splits past SHARD_MAX_FNS or
    SHARD_MAX_LOC. Verbatim policy from packets3.shard_chunks / uipackets."""
    tot = sum(f["loc"] for f in fns)
    n = max((len(fns) + SHARD_MAX_FNS - 1) // SHARD_MAX_FNS,
            (tot + SHARD_MAX_LOC - 1) // SHARD_MAX_LOC)
    n = min(n, len(fns))
    if n <= 1:
        return [fns]
    while True:
        chunks = _split(fns, tot, n)
        if n == len(fns) or all(
                len(c) <= SHARD_MAX_FNS
                and sum(f["loc"] for f in c) <= SHARD_MAX_LOC for c in chunks):
            return chunks
        n += 1


def _split(fns, tot, n):
    chunks, cur, acc = [], [], 0
    for i, f in enumerate(fns):
        if cur and len(chunks) < n - 1:
            boundary = (len(chunks) + 1) * tot / n
            over = (acc + f["loc"]) - boundary
            if (len(cur) >= SHARD_MAX_FNS
                    or (over > 0 and over >= boundary - acc)
                    or len(fns) - i == n - len(chunks) - 1):
                chunks.append(cur); cur, acc = [], 0
        cur.append(f); acc += f["loc"]
    chunks.append(cur)
    return chunks


# ------------------------------------------------------------------- rendering
def render_packet(cfile, wave, chunk, shard, n_shards, law, bg_sigs,
                  uishared_sigs, trap_seam, inmod_sig):
    c_sig, enum, tokens = trap_seam
    own = {f["name"] for f in chunk}
    o = []
    title = f"{cfile} — wave {wave}" + (f" — shard {shard}/{n_shards}"
                                        if shard else "")
    o.append(f"# CGAME PORT PACKET: `{title}`")
    o.append("")
    o.append(f"Fill the **{len(chunk)}** open functions below — one file's open "
             f"fns from **wave {wave}** of the topological partition "
             "(`out/cgame/cgame-wave-partition.json`). Every in-module callee "
             f"these fns need was ported in a **lower wave (< {wave})**; bg "
             "and ui_shared (mp_uishared) fns and the trap seam already exist. "
             "Transcribe directly to the idiomatic shape. Where a genuine "
             "question remains leave a cited `// DEFERRED:` or a `// "
             "PORT-NOTE:` at the site — NEVER a bare `//TODO: Port` (a wave "
             "that adds one fails review).")
    o.append("")
    o.append(f"- open fns: **{len(chunk)}**  ·  oracle LOC: "
             f"**{sum(f['loc'] for f in chunk)}**  ·  wave: **{wave}**  ·  "
             f"file: `{cfile}`")
    o.append("")

    # ---- CONFIG / root-type bindings (RATIFIED — DEC-46)
    o.append("## ROOT-TYPE BINDINGS (RATIFIED — DEC-46, C2 root-type "
             "sit-down 2026-07-28)")
    o.append("")
    o.append("The cgame root types are **ratified** — `docs/decisions.md` "
             "DEC-46. Transcribe against these six dispositions; a shape "
             "question DEC-46 doesn't cover is a real **escalation**, never "
             "an invention:")
    o.append(f"- **DEC-46.1 — `{CG_WORLD}` spine, Raven names.** "
             "`cg_t`/`cgs_t`/`cg_entities` → `CgWorld { cg: CgState, cgs: "
             "CgsState, entities: Box<[CEntity; MAX_GENTITIES]> }`. "
             "Transcribe line-for-line with Raven field names (`cg.time` → "
             "`world.cg.time`); media/weapon/item registries hang off the "
             "spine.")
    o.append("- **DEC-46.2 — `CEntity` owned + resolution enums.** "
             "`centity_t` pointer fields become owned/resolution types: "
             "`playerState: *mut` → `PlayerStateRef { None | Predicted | "
             "Snap }` resolved via `CgWorld` at use sites; `m_pVehicle` → "
             "`Option<VehicleId>`; `npcClient` → owned `Option<Box<..>>`; "
             "`ghoul2`/`ghoul2weapon` stay opaque engine tokens (never "
             "dereferenced module-side). The `bgEntity_t` prefix carries "
             "**no layout obligation** — bg reaches entity data through the "
             "accessor seam only, never a pointer pun.")
    o.append("- **DEC-46.3 — pools → gen-counted slab + LRU queue.** "
             "`localEntity_t` (512) / `markPoly_t` (256) intrusive pools "
             "become a gen-counted slab with `active: VecDeque<Handle>` in "
             "age order; alloc at capacity frees the oldest (Raven's "
             "`CG_AllocLocalEntity` steal behavior preserved verbatim). "
             "Intrusive `prev`/`next` dissolves.")
    o.append("- **DEC-46.4 — fn-ptr tables → closed enums + `match`.** "
             "`thinkFn`/leType dispatch and `weaponInfo_t`'s trail/charge "
             "fn-ptrs become closed enums; each arm cites its Raven fn; "
             "exhaustive, no `unsafe extern` fields.")
    o.append("- **DEC-46.5 — `CgGameCallbacks` INERT-PER-IFDEF law.** The "
             "~30 QAGAME-gated trait methods return the neutral value that "
             "keeps the gated block unreachable (accessors → false/0/None; "
             "mutators → no-op) — cite the oracle `#ifdef QAGAME` proving "
             "Raven's cgame build compiled the block out. These methods "
             "must NEVER perform a live `cg_entities` read. The 16 "
             "DEC-36-D5 registration arms (sound/model/effect/vehicle/"
             "siege traps) are real implementations, not gated.")
    o.append("- **DEC-46.6 — `cg.sharedBuffer` = pinned buffer + copy-out "
             "decode.** `Box<[u8; 2048]>` pinned on `CgWorld`, registered "
             "once via `CG_SET_SHARED_BUFFER`; consuming vmcalls copy out "
             "and decode through the existing abi `TCG*` types at the call "
             "boundary — no Rust reference outlives a call into engine-"
             "mutated memory.")
    o.append(f"- **`{CG_CONTEXT}`/`{MENU_SYSTEM}`/`{DISPLAY_CONTEXT}`** "
             "carry over unamended from C0/scoping.md — DEC-46 did not "
             f"revisit them: `{CG_CONTEXT} = {{ world: &mut {CG_WORLD}, "
             f"engine }}` owned by the vmMain entrypoints; `{MENU_SYSTEM}` "
             "is ui_shared.c's ALREADY-PORTED menu framework "
             f"(`mp_uishared`), owned by composition `{CG_WORLD}.menus`; "
             f"`{DISPLAY_CONTEXT}` is the same idiomatic render/text/cvar/"
             "feeder/ownerDraw/sound trait ui's packets use, replacing "
             "`displayContextDef_t`.")
    o.append("")

    # ---- law + dictionary
    o.append("## SCOPING-DOC DESIGN LAW (2026-07-24 — apply verbatim, "
             "PRE-C2 — treat any gap as an escalation)")
    o.append("")
    o.append(law)
    o.append("")

    # ---- file-scope constants (renderer R3 gap fix, ported here from wave 0)
    const_section = render_file_scope_constants(cfile)
    if const_section:
        o.append(const_section)

    # ---- threading digest per fn
    o.append("## THREADING DIGEST — per open fn")
    o.append("")
    for f in chunk:
        traps = sorted({c["name"] for c in f["callees"]["syscall"]})
        bgc = sorted({c["name"] for c in f["callees"]["bg"]})
        uic = sorted({c["name"] for c in f["callees"].get("uishared", [])})
        statics = f.get("statics", [])
        greads = [g["name"] for g in f["globals_read"]]
        gwrites = [g["name"] for g in f["globals_write"]]
        fnptr = f.get("fnptr_writes", [])

        o.append(f"### `{f['name']}` — {cfile}:{f['line']}-{f['end_line']} "
                 f"({f['loc']} LOC, wave {f['wave']})")
        engine = "engine traps (via `%s`)" % CG_CONTEXT if traps else None
        chan = [c for c in (engine,
                            "bg (direct calls into mp_bg)" if bgc else None,
                            "ui_shared (direct calls into mp_uishared)"
                            if uic else None,
                            f"`{CG_WORLD}` state (statics/globals below)"
                            if (statics or greads or gwrites) else None,
                            f"`{MENU_SYSTEM}`/`{DISPLAY_CONTEXT}` fn-ptr "
                            "dispatch" if fnptr else None) if c]
        o.append("- **channel:** " + ("; ".join(chan) if chan
                 else "pure fn — no state channel (no traps/globals/bg/"
                      "uishared/fn-ptr)"))
        if traps:
            parts = []
            for t in traps:
                e = enum.get(t)
                tok = "abi token present" if e in tokens else "NO abi token"
                parts.append(f"`{t}` (**MISSING-WRAPPER** — C3 builds it; "
                             f"{tok}" + (f": `{e}`" if e else "") + ")")
            o.append("- **traps → seam:** " + "; ".join(parts))
        if bgc:
            o.append("- **bg calls:** " + ", ".join(f"`{b}`" for b in bgc))
        if uic:
            o.append("- **ui_shared calls:** " + ", ".join(f"`{b}`" for b in uic))
        # statics-to-fold
        fold = []
        if statics:
            fold.append("fn-scope statics: "
                        + ", ".join(f"`{s['name']}: {s['type']}`" for s in statics))
        if greads:
            fold.append("file globals READ: " + ", ".join(f"`{g}`" for g in greads))
        if gwrites:
            fold.append("file globals WRITTEN: "
                        + ", ".join(f"`{g}`" for g in gwrites))
        if fold:
            o.append(f"- **statics-to-fold (→ `{CG_WORLD}` fields per "
                     "DEC-46):** " + "  ·  ".join(fold))
        if fnptr:
            fps = ", ".join(f"`{w['field']}={w['target']}`" for w in fnptr)
            o.append(f"- **fn-ptr dispatch writes:** {fps} — a "
                     f"`{DISPLAY_CONTEXT}`/feeder/ownerDraw vtable slot, or "
                     "(cg_localents/cg_marks) a pool `thinkFn` — assign the "
                     "idiomatic trait method / `match` arm, not a raw fn ptr "
                     "(dictionary: fn-ptr tables → `match`, trait only where "
                     "the set is open).")
        o.append("")

    # ---- oracle source
    o.append("## ORACLE SOURCE (verbatim — transcribe these bodies)")
    o.append("")
    for f in chunk:
        o.append(f"### `{f['name']}` — {cfile}:{f['line']}-{f['end_line']}")
        o.append(f"Oracle C signature: `{oracle_c_sig(f)}`")
        o.append("")
        o.append("```c")
        o.append(numbered_slice(cfile, f["line"], f["end_line"]))
        o.append("```")
        o.append("")

    # ---- resolved call surface
    bg_calls, ui_calls, trap_calls, inmod, other = set(), set(), set(), {}, set()
    for f in chunk:
        for c in f["callees"]["bg"]:
            bg_calls.add(c["name"])
        for c in f["callees"].get("uishared", []):
            ui_calls.add(c["name"])
        for c in f["callees"]["syscall"]:
            trap_calls.add(c["name"])
        for c in f["callees"]["in-module"]:
            if c["name"] not in own:
                inmod[c["name"]] = c.get("cite")
        for c in f["callees"]["libc/other"]:
            other.add(c["name"])

    o.append("## RESOLVED CALL SURFACE — signatures are LAW, do not explore")
    o.append("")
    o.append(f"### bg (crates/mp/bg — ALREADY PORTED; call directly) — {len(bg_calls)}")
    o.append("```rust")
    for name in sorted(bg_calls):
        if name in bg_sigs:
            fn, sig = bg_sigs[name]
            o.append(f"// {fn}")
            o.append(sig + ";")
        elif name in BG_LIB_SHIMS:
            o.append(f"// {name}: NOT an mp_bg fn — {BG_LIB_SHIMS[name]}")
        else:
            o.append(f"// {name}: bg-tier, signature not resolved in crates/mp/bg "
                     "— confirm before use")
    o.append("```")
    o.append("")

    o.append(f"### ui_shared (crates/mp/uishared — ALREADY PORTED; call "
             f"directly) — {len(ui_calls)}")
    o.append("```rust")
    for name in sorted(ui_calls):
        if name in uishared_sigs:
            fn, sig = uishared_sigs[name]
            o.append(f"// {fn}")
            o.append(sig + ";")
        else:
            o.append(f"// {name}: uishared-tier, signature not resolved in "
                     "crates/mp/uishared — confirm before use")
    o.append("```")
    o.append("")

    o.append(f"### trap seam (crates/mp/abi/src/cgame) — {len(trap_calls)}")
    o.append(f"Every cgame `trap_*` ERGONOMIC wrapper is **MISSING-WRAPPER** "
             "until C3 confirms/builds it over the abi syscall token shown. "
             "The cg_syscalls.c C contract below is the LAW arg/return shape "
             "(apply the dictionary at the wrapper: `char*`→`&str`/`String`, "
             "`qboolean`→`bool`, out-params→returns). Call the wrapper C3 "
             "lands; do NOT hand-roll a syscall here.")
    o.append("```c")
    for name in sorted(trap_calls):
        e = enum.get(name)
        tok = "token PRESENT" if e in tokens else "token ABSENT"
        o.append(f"// MISSING-WRAPPER  ·  abi: {e or '?'} ({tok})")
        o.append(c_sig.get(name, f"/* {name}: no C sig found in cg_syscalls.c */"))
    o.append("```")
    o.append("")

    o.append(f"### in-module (ported in a LOWER wave — call directly) — {len(inmod)}")
    o.append("Oracle C signatures (these fns land already-idiomatic in an earlier "
             "wave; match the shape the earlier packet produced):")
    o.append("```c")
    for name in sorted(inmod):
        cite = inmod[name] or "cgame/"
        o.append(f"// wave < {wave}  ·  {cite}")
        o.append(inmod_sig.get(name, f"{name}(...);"))
    o.append("```")
    o.append("")

    if other:
        o.append(f"### qshared / libc helpers — {len(other)}")
        o.append("Ported qshared helpers (`native_math`/`mp_qshared`: `COM_Parse`, "
                 "`Com_sprintf`, `Q_stricmp`, `VectorNormalize`, `Info_ValueForKey`, "
                 "…) and true libc (`atoi`, `cos`, `strlen`, …). Use the ported "
                 "helper or the Rust-idiom equivalent per the dictionary "
                 "(`Com_sprintf`/`va` → `format!`); not individually LAW-listed:")
        o.append("- " + ", ".join(f"`{n}`" for n in sorted(other)))
        o.append("")

    return "\n".join(o)


# ------------------------------------------------------------------- main
def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--wave", type=int, default=None,
                    help="restrict to one wave (union with --only)")
    ap.add_argument("--only", nargs="*", default=None,
                    help="restrict to these oracle .c files (union with --wave)")
    args = ap.parse_args()

    manifest = json.loads(MANIFEST.read_text())
    funcs = manifest["functions"]
    law = plan_digest()
    bg_sigs = load_bg_sigs()
    uishared_sigs = load_uishared_sigs()
    trap_seam = load_trap_seam()
    inmod_sig = {f["name"]: oracle_c_sig(f) for f in funcs}

    # group by (wave, file)
    groups = {}
    for f in funcs:
        groups.setdefault((f["wave"], f["file"]), []).append(f)
    for v in groups.values():
        v.sort(key=lambda f: f["line"])

    # selection: if neither filter given -> all; else union of the two filters
    def selected(wave, cfile):
        if args.wave is None and args.only is None:
            return True
        return (args.wave is not None and wave == args.wave) or \
               (args.only is not None and cfile in set(args.only))

    (OUT / "packets").mkdir(parents=True, exist_ok=True)
    man = []
    for (wave, cfile) in sorted(groups):
        if not selected(wave, cfile):
            continue
        # cg_playeranimate.c is a 0-line oracle file (empty TU, no fns) — the
        # (wave, file) group is simply empty and never appears; nothing
        # special to special-case here (documented for the future reader).
        chunks = shard_chunks(groups[(wave, cfile)])
        n = len(chunks)
        for si, chunk in enumerate(chunks):
            shard = (si + 1) if n > 1 else None
            base = cfile[:-2]
            text = render_packet(cfile, wave, chunk, shard, n, law,
                                 bg_sigs, uishared_sigs, trap_seam, inmod_sig)
            fname = f"{base}.wave{wave}" + (f".shard{shard}" if shard else "") + ".md"
            (OUT / "packets" / fname).write_text(text)
            man.append({
                "file": cfile, "wave": wave, "packet": f"packets/{fname}",
                "fns": len(chunk), "loc": sum(f["loc"] for f in chunk),
                **({"shard": shard, "shards_total": n} if shard else {})})

    man.sort(key=lambda e: (e["wave"], e["file"], e.get("shard", 0)))
    (OUT / "packets-manifest.json").write_text(json.dumps(man, indent=1))
    print(f"[cgpackets] {len(man)} packets, "
          f"{sum(e['fns'] for e in man)} fns, {sum(e['loc'] for e in man):,} LOC "
          f"-> out/cgame/packets/")
    if CONFIG_FINALIZED is False:
        print("[cgpackets] NOTE: root-type bindings are PLACEHOLDER (C2 pending)")
    else:
        print("[cgpackets] root-type bindings are RATIFIED (DEC-46)")


if __name__ == "__main__":
    main()
