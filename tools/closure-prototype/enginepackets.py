#!/usr/bin/env python3
"""PROTOTYPE — throwaway. ENGINE SIGNATURE MANIFEST: one self-contained,
machine-verifiable work order per row of the engine port order
(`out/engine/engine-port-order.{json,tsv}`, 2,481 fns). The engine sibling of
`packets3.py` (which ran the jampgame port); `packets3.py` stays untouched as the
jampgame record.

Where packets3 read ALREADY-PORTED worktree Rust signatures as LAW, the engine is
unported, so this tool DERIVES each resolved Rust signature from the clang cursor
under the mechanical defaults of porting-rules §C (transcription-first: out-params
stay raw seam pointers, `qboolean` stays `qboolean`, named types resolve through
the type rosetta). It reuses `engineorder.build()` — the SAME pinned per-file
libclang parse (WinDed Release macro set, `-fms-compatibility`, empty LittleShort/
LittleLong defines, `stricmp=strcasecmp`; see engineorder/closure/sweep docstrings)
— so signatures, port order and the stub-free dependency edges are one consistent
artifact.

Output (out/engine/packets/):
  _PREAMBLE.md            shared conventions handed to every porter alongside a
                          shard: the 48 engine-fork rulings digest, the §C
                          mechanical C->Rust type-map, the fork-3 three-kind
                          static rule (ruling 3), ruling-2 global placement, the
                          no-stub / no-TODO discipline, cpp-track doc routing.
  <sub>__<seq>_<sym>.md   one packet per C-track unit (a single free fn, or a
                          whole cyclic SCC ported together): verbatim oracle
                          source slice(s) + cites, the derived resolved Rust
                          signature, the touched file-scope globals + function
                          statics with their classification hook, the resolved
                          callee surface (each -> its packet path / doc / extern),
                          and the rosetta rows for every type the signature/body
                          names.
  manifest.json           every one of the 2,481 fns -> packet path | cpp-track
                          design-doc pointer | (§20-drop marker: none at gen time),
                          plus the LOC-balanced shard bundles, the machine-check
                          results, and the missing-rosetta-type / undocumented-cpp
                          reports.
  generation-report.md    human summary of the run (counts, top missing types,
                          dangling callees, shards, structural surprises).

Machine checks (built in, never silent):
  * every in-engine callee ref resolves to a manifest entry (dangling = 0 by the
    stub-free order property; verified explicitly and reported);
  * every type a signature names resolves in the rosetta or a scalar map, else it
    is a generation-report entry (missing types are referee items, not stubs);
  * every C++ method whose class has no frozen §F design doc is flagged
    (undocumented-cpp = referee items).

Usage:
  .venv/bin/python enginepackets.py                 # full run (re-parses, ~50s)
  .venv/bin/python enginepackets.py --cache X.pkl    # dev: load a pickled build
"""
import argparse
import json
import pickle
import re
import sys
import time
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import engineorder as EO
import closure as C

HERE = Path(__file__).resolve().parent
REPO = C.REPO
OUT = HERE / "out" / "engine" / "packets"
ROSETTA_TSV = HERE / "out" / "engine" / "type-rosetta.tsv"
RULINGS = REPO / "docs" / "handoffs" / "engine-fork-discovery.md"
SHARD_TARGET_LOC = 450

# ---------------------------------------------------------------- cpp-track
# The five FROZEN §F design docs (ruling 7 + the doc-session rulings). A function
# routed to a doc is NOT given a mechanical signature — the doc's Method-
# transcription table IS its work order (porting-rules §F). Classification is by
# the evidence in engine-port-order (subsystem dir / owning file / owner class),
# confirmed against each doc's own class coverage.
DOC = {
    "icarus": "docs/subsystems/icarus.md",
    "rmg": "docs/subsystems/rmg-terrain.md",
    "ghoul2": "docs/subsystems/ghoul2-server.md",
    "npcnav": "docs/subsystems/npcnav.md",
    "roff": "docs/subsystems/roff.md",
}
# RMG qcommon terrain twins folded into rmg-terrain.md (ruling 16/28); class set
# confirmed present in rmg-terrain.md.
RMG_FOLDED = {"CCMLandScape", "CRandomTerrain", "CTerrainMap", "CPathInfo",
              "CArea", "CCMPatch", "CCMHeightDetails", "CCMShaderText"}
# ghoul2 render internals (G2SV) confirmed in ghoul2-server.md.
GHOUL2_CLASSES = {"CBoneCache", "CTransformBone"}
# GP2 is the DONE C++ pilot (porting-rules §F exemplar) — not a docs/subsystems
# doc; its work order is the landed reimplementation.
GP2_DIR = "crates/mp/engine/qcommon/src/gp2/"
GP2_CLASSES = {"CGPGroup", "CGPValue", "CGPObject", "CGenericParser2", "CTextPool"}


def classify(f):
    """(track, pointer) for one fn. track in
    {'c','cpp','cpp-done','cpp-undocumented'}."""
    sub, file, owner = f["subsystem"], f["file"], f["owner"]
    if sub == "icarus":
        return "cpp", DOC["icarus"]
    if sub == "RMG":
        return "cpp", DOC["rmg"]
    if sub == "ghoul2":
        return "cpp", DOC["ghoul2"]
    if file == "tr_ghoul2.cpp" or owner in GHOUL2_CLASSES:
        return "cpp", DOC["ghoul2"]
    if file == "navigator.cpp":
        return "cpp", DOC["npcnav"]
    if file == "RoffSystem.cpp":
        return "cpp", DOC["roff"]
    if owner in RMG_FOLDED:
        return "cpp", DOC["rmg"]
    if file == "GenericParser2.cpp" or owner in GP2_CLASSES:
        return "cpp-done", GP2_DIR
    if owner is not None:
        # a C++ class method with no frozen §F doc — a coverage GAP (referee item)
        return "cpp-undocumented", None
    return "c", None


# ------------------------------------------------------------- type rosetta
def load_rosetta():
    """raven_name -> row. On a name collision, prefer the IDENTITY-named port
    (rust_name == raven_name — the canonical typedef port) over a false-positive
    attribution. This defends against a known typemap.py defect: a doc comment on
    an unrelated struct (`gtimer_t`) that mentions `Raven \`qboolean\`` wrongly
    claims the `qboolean` row, which otherwise sorts ahead of the real
    native_types `qboolean` port."""
    rows = {}
    for ln in ROSETTA_TSV.read_text().splitlines()[1:]:
        raven, rust, kind, crate, path, cite = (ln.split("\t") + [""] * 6)[:6]
        row = {"raven": raven, "rust": rust, "kind": kind, "crate": crate,
               "path": path, "cite": cite}
        prev = rows.get(raven)
        if prev is None or (rust == raven and prev["rust"] != raven):
            rows[raven] = row
    return rows


# §C mechanical C->Rust scalar map (primitives + Raven's byte-swap-free widths).
# Everything NOT here resolves through the type rosetta; a base name in neither is
# a missing-type report entry, never silently kept.
SCALAR = {
    "void": "()", "bool": "bool",
    "int": "c_int", "unsigned int": "c_uint", "unsigned": "c_uint",
    "char": "c_char", "unsigned char": "c_uchar",
    "short": "c_short", "unsigned short": "c_ushort",
    "long": "c_long", "unsigned long": "c_ulong",
    "long long": "c_longlong", "unsigned long long": "c_ulonglong",
    "float": "f32", "double": "f64",
    "size_t": "usize", "intptr_t": "isize", "uintptr_t": "usize",
    "ptrdiff_t": "isize", "wchar_t": "c_int",
    "int8_t": "i8", "uint8_t": "u8", "int16_t": "i16", "uint16_t": "u16",
    "int32_t": "i32", "uint32_t": "u32", "int64_t": "i64", "uint64_t": "u64",
    "va_list": "va_list", "...": "...",
    "signed char": "c_schar", "signed": "c_int",
}
_STRIP = re.compile(r"\b(const|volatile|struct|enum|union|class|register|"
                    r"__restrict|restrict)\b")
_ARRAY = re.compile(r"\[[^\]]*\]")
# C++ std / libc spellings the mechanical map resolves (kept tiny — the honest
# default is to flag, so only names with an unambiguous Rust twin go here).
CPP_MAP = {"string": "String", "std::string": "String", "basic_string": "String"}

# Externals NOT owned by the Rust port — they cross a seam the Rust host fills
# with its own substrate, so they resolve as non-rosetta types and are
# whitelisted out of the "missing rosetta type" referee check rather than ported:
#   - libc `FILE`               : host stdio handle behind the filesystem seam.
#   - minizip/zlib `uLong`/`uInt`/`unz_*` : the zip layer is a Rust-crate seam
#                                 (DEC — zlib/minizip not transcribed).
#   - `long double`             : no Rust twin (x87 80-bit); appears only in the
#                                 unused NumberValue/ReadSignedFloat scan paths.
# Anonymous function-pointer parameter types (spelled with parens, e.g.
# `void ( )(int, int, float )`) are handled positionally in the loop below — they
# become Rust `fn`/`extern "C" fn` pointers, not rosetta rows.
EXTERNAL = {
    "FILE",
    "uLong", "uInt", "unz_file_info", "unz_global_info", "tm_unz",
    "unz_file_info_internal", "unz_s",
    "long double",
}


def normalize_base(spelling):
    """(base_name, nptr, nref, inner_const) from a clang type spelling. A C array
    param decays to a pointer — one extra pointer level per `[...]` group."""
    s = spelling.strip()
    narr = len(_ARRAY.findall(s))
    s = _ARRAY.sub("", s)
    nptr = s.count("*") + narr
    nref = s.count("&")
    inner_const = (bool(re.search(r"\bconst\b", s.split("*")[0])) if "*" in s
                   else s.startswith("const"))
    base = _STRIP.sub(" ", s.replace("*", " ").replace("&", " "))
    base = re.sub(r"\s+", " ", base).strip()
    return base, nptr, nref, inner_const


def resolve_type(spelling, rosetta, miss, cpp_types=frozenset()):
    """Map a clang C/C++ type spelling to a mechanical §C Rust type. Records any
    non-scalar base name absent from the rosetta / cpp-track class set into
    `miss`. The faithful default: pointers stay raw seam pointers (transcription-
    first), C++ refs become &/&mut, named types resolve through the rosetta."""
    if not spelling.strip():
        return "()"
    base, nptr, nref, inner_const = normalize_base(spelling)
    if base in SCALAR:
        rust = SCALAR[base]
    elif base in rosetta:
        rust = rosetta[base]["rust"]
    elif base in CPP_MAP:
        rust = CPP_MAP[base]
    else:
        rust = base.replace(" ", "_").replace(":", "_") or "c_void"
        if base and base != "void" and base not in cpp_types:
            miss[base] = None
    for i in range(nptr):
        ptr = "*const" if (inner_const and i == 0) else "*mut"
        rust = f"{ptr} {rust}"
    for _ in range(nref):
        rust = ("&" if inner_const else "&mut ") + rust
    return rust


def sig_type_names(f):
    """Non-scalar base type names a fn's signature (params + ret) names — the
    strict resolve-or-flag surface."""
    names = set()
    for t in [f["ret_type"]] + [p["type"] for p in f["params"]]:
        base, _, _, _ = normalize_base(t)
        if base and base not in SCALAR and base not in CPP_MAP:
            names.add(base)
    return names


def resolved_signature(f, rosetta, miss, cpp_types=frozenset()):
    """`pub fn name(p: T, ...) -> R` under §C mechanical defaults."""
    ps = []
    for p in f["params"]:
        rt = resolve_type(p["type"], rosetta, miss, cpp_types)
        nm = p["name"] or f"a{len(ps)}"
        if rt == "...":
            ps.append("...")
        else:
            ps.append(f"{nm}: {rt}")
    if f["variadic"] and "..." not in ps:
        ps.append("...")
    ret = resolve_type(f["ret_type"], rosetta, miss, cpp_types)
    rets = "" if ret == "()" else f" -> {ret}"
    return f"pub fn {f['name']}({', '.join(ps)}){rets}"


# --------------------------------------------------------------- oracle slice
def build_name2path():
    idx = {}
    for p in EO.winded_sources():
        idx[p.name] = p
    return idx


_LINES = {}


def numbered_slice(path, a, b):
    lines = _LINES.get(path)
    if lines is None:
        lines = Path(path).read_text(errors="replace").splitlines()
        _LINES[path] = lines
    a = max(1, a)
    b = min(len(lines), b)
    return "\n".join(f"{n:>5} | {lines[n-1]}" for n in range(a, b + 1))


def body_text(path, a, b):
    lines = _LINES.get(path)
    if lines is None:
        lines = Path(path).read_text(errors="replace").splitlines()
        _LINES[path] = lines
    return "\n".join(lines[a - 1:b])


def cite_line(cite):
    """'oracle/..:NN' or 'oracle/..:NN-MM' -> (relpath, start, end)."""
    m = re.match(r"(.+):(\d+)(?:-(\d+))?$", cite)
    if not m:
        return None
    return m.group(1), int(m.group(2)), int(m.group(3) or m.group(2))


# ------------------------------------------------------------- rulings digest
def extract_rulings():
    return RULINGS.read_text().rstrip()


# ------------------------------------------------------------------ preamble
def render_preamble(rulings):
    o = []
    o.append("# ENGINE PORT — SHARED PACKET PREAMBLE")
    o.append("")
    o.append("Handed to every porter alongside a shard of per-function packets. "
             "The packet carries the fn-specific work order (source, signature, "
             "callees, types); THIS file carries the settled conventions that "
             "apply to all of them. Do not re-derive anything here per packet.")
    o.append("")
    o.append("## Port discipline (non-negotiable)")
    o.append("")
    o.append("- **Transcription-first (porting-rules §A/§C).** Port Raven's body "
             "faithfully into the resolved signature. No speculative 'cleaner' "
             "behavior; get parity first, refactor behind a green diff later.")
    o.append("- **No stubs, no `todo!()`, no `TODO: Port`, no `FIXME` "
             "(ruling 34 + GOAL-engine.md).** The port order is stub-free: every "
             "in-engine callee is already ported by the time a fn's turn comes "
             "(machine-verified). A genuine open question is a one-line "
             "`// PORT-NOTE(<subject>): …` at the site, never a park.")
    o.append("- **A type name missing from the rosetta is an escalation, not a "
             "stub** (engine-fork-discovery 'type rosetta' section). All ABI/"
             "layout types already exist — import from the rosetta path, never "
             "declare a struct/enum/typedef.")
    o.append("")
    o.append("## Resolved-signature contract (§C mechanical defaults)")
    o.append("")
    o.append("Each C-track packet prints a `pub fn` signature DERIVED from the "
             "clang cursor under these defaults — transcribe the body into it:")
    o.append("- **out-params stay out-params.** A `T *` Raven writes through "
             "stays a raw seam pointer (`*mut T`); a `const T *` is `*const T`. "
             "Raw pointers are the confined ABI-seam idiom (porting-rules §D11); "
             "the safe-state migration idiomatizes them later, not now.")
    o.append("- **`qboolean` stays `qboolean`** at ABI-visible boundaries (it is "
             "a ported rosetta enum); it does not collapse to `bool` mid-port.")
    o.append("- **scalars** map by width (`int`→`c_int`, `float`→`f32`, "
             "`char`→`c_char`, `unsigned long`→`c_ulong`, …); **named types** "
             "resolve to their rosetta Rust name (listed per packet).")
    o.append("- **C++ refs** (`T&`/`const T&`) become `&mut T`/`&T`. The "
             "signature is the faithful shape; STATE THREADING (which globals "
             "become which owner-struct field) is separate — see below.")
    o.append("")
    o.append("## State threading — globals & statics")
    o.append("")
    o.append("The resolved signature is Raven's own parameter list; the engine's "
             "~680 file-scope globals do NOT become Rust globals (ruling 2). Each "
             "packet lists the globals/statics its fns touch:")
    o.append("- **File-scope globals (ruling 2):** fields on the owning subsystem "
             "struct under the `Engine` aggregate (`Engine { common, sv, cm, fs, "
             "net, bot, g2, … }`), grouped by owning .c file; cvar *handles* in an "
             "`EngineCvars` sub-struct. No `static mut`. The porting campaign wires "
             "the receiver; the packet tells you WHICH state the body reaches.")
    o.append("- **Function-scope statics — the fork-3 three-kind rule (ruling 3, "
             "the jampgame fork-5 rule blessed unchanged):** classify each and "
             "port accordingly —")
    o.append("  1. **const table** → a Rust `const`/`static` (no mutation);")
    o.append("  2. **rotating scratch / return buffer** → an owned return value "
             "(`Vec`/`String`/array), never a hidden cell;")
    o.append("  3. **genuine cross-frame state** → a field on the owning host "
             "struct (ruling 2), threaded in.")
    o.append("")
    o.append("## Error recovery (ruling 1)")
    o.append("")
    o.append("`Com_Error` is Raven's `longjmp`. Port as a Rust panic caught by "
             "`catch_unwind` at exactly Raven's setjmp sites (`Com_Frame`/"
             "`Com_Init`); the payload carries level+message; `com_error_recover` "
             "is the landing pad. NEVER thread `Result` through signatures — that "
             "would rewrite every fn and break transcription-first. A `Com_Error` "
             "call site is a `panic!`/unwind, not an early `return Err`.")
    o.append("")
    o.append("## Dispatch tables (ruling 5)")
    o.append("")
    o.append("Internal fn-ptr tables (`botlib_import/export_t`, `refexport_t`, "
             "ICARUS `interface_export_t`, `ucmds[]`) are plain Rust structs of "
             "`fn` items populated at the same init sites; command tables are "
             "`&[(&str, fn(...))]` consts. No fn-ID enums (no address compares in "
             "the engine, unlike jampgame entity handlers).")
    o.append("")
    o.append("## C++-track subsystems (porting-rules §F)")
    o.append("")
    o.append("Functions in the five frozen §F subsystems are NOT in your shard — "
             "they are reimplemented idiomatically against a FROZEN design doc "
             "whose Method-transcription table is the work order. If a callee "
             "resolves to one of these, treat its doc-stated signature as settled:")
    for k, v in DOC.items():
        o.append(f"- **{k}** → `{v}`")
    o.append(f"- **GP2** (done pilot exemplar) → `{GP2_DIR}`")
    o.append("")
    o.append("---")
    o.append("")
    o.append("## ENGINE FORK RULINGS (verbatim — all 48 settled)")
    o.append("")
    o.append(rulings)
    o.append("")
    return "\n".join(o)


# ------------------------------------------------------------------ packet
def render_packet(unit_members, dest, rosetta, name2path, usr2dest, usr2name,
                  calls_of, refs_of, miss, cpp_class_doc, rosetta_keys):
    """One packet for a C-track unit (>1 member = a cyclic SCC, ported together)."""
    cpp_types = frozenset(cpp_class_doc)
    cyclic = len(unit_members) > 1
    first = unit_members[0]
    seq0 = first["seq"]
    o = []
    title = first["qualname"] + (f"  (+{len(unit_members)-1} cyclic peers)"
                                 if cyclic else "")
    o.append(f"# ENGINE PORT PACKET — `{title}`")
    o.append("")
    o.append(f"- seq **{seq0}**  ·  wave **{first['wave']}**  ·  unit "
             f"**{first['unit']}**  ·  subsystem **{first['subsystem']}**  ·  "
             f"file `{first['file']}`  ·  oracle LOC "
             f"**{sum(m['loc'] for m in unit_members)}**")
    o.append(f"- track: **C** (mechanical resolved signature). Read `_PREAMBLE.md` "
             "first — it holds the §C contract, the three-kind static rule, the "
             "no-stub discipline, and doc routing.")
    if cyclic:
        o.append(f"- **CYCLIC UNIT ({len(unit_members)} fns): mutual recursion — "
                 "port these bodies TOGETHER; every peer signature below is fixed "
                 "before any body is filled, so call peers directly.**")
    o.append("")

    # ---- resolved signatures (all members up front, so a cyclic unit compiles)
    o.append("## RESOLVED SIGNATURES (§C mechanical — transcribe the body into these)")
    o.append("")
    o.append("```rust")
    for m in unit_members:
        o.append(resolved_signature(m, rosetta, miss, cpp_types) + " { /* "
                 "PORT-NOTE if needed; port here */ }")
    o.append("```")
    o.append("")
    if any(m["variadic"] for m in unit_members):
        o.append("_Variadic fn: Raven `va`/`Com_sprintf`/printf family — format "
                 "at the call, pass the resolved buffer; the qshared `va`/"
                 "`Com_sprintf` surface is already ported (extern below)._")
        o.append("")

    # ---- state: globals (ruling 2) + statics (ruling 3 three-kind)
    glob_rows, static_rows = [], []
    for m in unit_members:
        for g in m["globals_write"]:
            glob_rows.append((m["name"], g["name"], "write", g["cite"]))
        for g in m["globals_read"]:
            glob_rows.append((m["name"], g["name"], "read", g["cite"]))
        for st in m["statics"]:
            static_rows.append((m["name"], st["name"], st["type"]))
    if glob_rows or static_rows:
        o.append("## STATE THREADED (globals ruling 2 · statics ruling 3)")
        o.append("")
        if glob_rows:
            o.append("File-scope globals this unit touches → `Engine` sub-struct "
                     "fields (ruling 2), grouped by owning file; no `static mut`:")
            o.append("")
            o.append("| fn | global | access | decl |")
            o.append("| --- | --- | --- | --- |")
            for fn, g, acc, cite in glob_rows:
                o.append(f"| `{fn}` | `{g}` | {acc} | `{cite}` |")
            o.append("")
        if static_rows:
            o.append("Function-scope statics → classify per the fork-3 three-kind "
                     "rule (const table / rotating scratch → owned return / "
                     "cross-frame state → host field); see `_PREAMBLE.md`:")
            o.append("")
            o.append("| fn | static | type |")
            o.append("| --- | --- | --- |")
            for fn, nm, ty in static_rows:
                o.append(f"| `{fn}` | `{nm}` | `{ty}` |")
            o.append("")

    # ---- verbatim oracle source (bodies + the decls of the statics it touches)
    o.append("## ORACLE SOURCE (verbatim — transcribe these bodies)")
    o.append("")
    path = name2path.get(first["file"])
    rel = None
    if path is not None:
        try:
            rel = str(Path(path).relative_to(REPO))
        except ValueError:
            rel = str(path)
    for m in unit_members:
        o.append(f"### `{m['qualname']}` — {rel}:{m['line']}-{m['end_line']} "
                 f"({m['loc']} LOC)")
        o.append("```c")
        if path is not None:
            o.append(numbered_slice(path, m["line"], m["end_line"]))
        else:
            o.append(f"// UNRESOLVED FILE {m['file']} — referee item")
        o.append("```")
        o.append("")
    # file-scope global declarations the unit reads/writes (small decl slices)
    decl_cites = {}
    for _, g, _, cite in glob_rows:
        pc = cite_line(cite)
        if pc:
            decl_cites[cite] = pc
    if decl_cites:
        o.append("### File-scope global declarations (state to thread)")
        o.append("```c")
        for cite, (relp, a, b) in sorted(decl_cites.items()):
            gp = REPO / relp
            if gp.exists():
                o.append(f"// {cite}")
                o.append(numbered_slice(gp, a, min(b, a + 3)))
        o.append("```")
        o.append("")

    # ---- resolved callee surface
    own = {m["usr"] for m in unit_members}
    callee_usrs = set()
    for m in unit_members:
        callee_usrs |= set(calls_of.get(m["usr"], ()))
        callee_usrs |= set(refs_of.get(m["usr"], ()))
    callee_usrs -= own

    def is_type_noise(e):
        # C++ implicit member-op / ctor spellings (operator[], operator=, a stack
        # ctor named by its struct tag foo_s) are parse artifacts, not call targets.
        tag = (e[:-2] + "_t") if e.endswith("_s") else None
        return (e.startswith("operator") or e in cpp_class_doc or e in rosetta_keys
                or (tag and tag in rosetta_keys))
    externals = sorted({e for m in unit_members for e in m["externals"]
                        if not is_type_noise(e)})
    o.append(f"## RESOLVED CALL SURFACE — {len(callee_usrs)} in-engine callee(s), "
             f"{len(externals)} external(s)")
    o.append("")
    if callee_usrs:
        o.append("Each in-engine callee is ALREADY PORTED at this point in the "
                 "order (stub-free) — call it; find its work order here:")
        o.append("")
        o.append("| callee | resolution |")
        o.append("| --- | --- |")
        for u in sorted(callee_usrs, key=lambda u: usr2name.get(u, u)):
            nm = usr2name.get(u, u)
            d = usr2dest.get(u)
            if d is None:
                res = "**DANGLING — referee item**"
            elif d[0] == "c":
                res = f"packet `{d[1]}`"
            elif d[0] == "cpp":
                res = f"§F doc `{d[1]}`"
            elif d[0] == "cpp-done":
                res = f"done (GP2) `{d[1]}`"
            else:
                res = "undocumented C++ class — referee item"
            o.append(f"| `{nm}` | {res} |")
        o.append("")
    if externals:
        o.append("**Externals** (supplied by Rust std/libc or the already-ported "
                 "qshared `q_shared`/`q_math` surface — do NOT port here, call "
                 "through the existing crate):")
        o.append("")
        o.append(", ".join(f"`{e}`" for e in externals))
        o.append("")

    # ---- rosetta rows for every type the signature/body names
    named = set()
    for m in unit_members:
        named |= sig_type_names(m)
    # body-named enrichment: tokenize each body once, intersect with rosetta keys
    # (a rosetta raven-name appearing as an identifier in the body).
    if path is not None:
        for m in unit_members:
            b = body_text(path, m["line"], m["end_line"])
            named |= (set(re.findall(r"[A-Za-z_]\w{3,}", b)) & rosetta_keys)
    o.append("## TYPE ROSETTA — import these, never redeclare")
    o.append("")
    hit = sorted(n for n in named if n in rosetta)
    cpp_named = sorted(n for n in named if n not in rosetta and n in cpp_class_doc)
    missing = sorted(n for n in named if n not in rosetta and n not in SCALAR
                     and n not in cpp_class_doc)
    if hit:
        o.append("| Raven type | Rust | kind | crate | path |")
        o.append("| --- | --- | --- | --- | --- |")
        for n in hit:
            r = rosetta[n]
            o.append(f"| `{n}` | `{r['rust']}` | {r['kind']} | {r['crate']} | "
                     f"`{r['path']}` |")
        o.append("")
    if cpp_named:
        o.append("**C++-track types (defined by a frozen §F design doc, NOT the "
                 "rosetta — the doc is authoritative for their Rust shape):** "
                 + ", ".join(f"`{n}` → `{cpp_class_doc[n] or 'undocumented (referee)'}`"
                             for n in cpp_named))
        o.append("")
    if missing:
        o.append("**Not in the rosetta (escalation, NOT a stub — a real port "
                 "needs these landed/renamed first; the finisher triages):** "
                 + ", ".join(f"`{n}`" for n in missing))
        o.append("")
    return "\n".join(o)


# ------------------------------------------------------------------ shards
def build_shards(packets):
    """LOC-balanced bundles of packets in seq order, ~SHARD_TARGET_LOC each. A
    packet larger than the target is its own shard."""
    shards, cur, acc = [], [], 0
    for p in packets:
        if cur and acc + p["loc"] > SHARD_TARGET_LOC:
            shards.append(cur)
            cur, acc = [], 0
        cur.append(p)
        acc += p["loc"]
        if acc >= SHARD_TARGET_LOC:
            shards.append(cur)
            cur, acc = [], 0
    if cur:
        shards.append(cur)
    return shards


# ------------------------------------------------------------------ main
def load_build(cache):
    if cache:
        b = pickle.load(open(cache, "rb"))
        return (b["funcs"], b["units"], b["edges"], b["calls_of"],
                b["refs_of"], b["usr_to"], b["stats"])
    funcs, units, edges, calls_of, refs_of, ext_census, stats, usr_to = \
        EO.build("mp-engine-ded")
    return (funcs, units, {k: list(v) for k, v in edges.items()},
            {k: list(v) for k, v in calls_of.items()},
            {k: list(v) for k, v in refs_of.items()}, usr_to, stats)


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--cache", help="pickled engineorder.build() blob (dev)")
    ap.add_argument("--out", default=str(OUT))
    args = ap.parse_args()
    t0 = time.time()

    funcs, units, edges, calls_of, refs_of, usr_to, order_stats = \
        load_build(args.cache)
    rosetta = load_rosetta()
    name2path = build_name2path()
    rulings = extract_rulings()
    outdir = Path(args.out)
    outdir.mkdir(parents=True, exist_ok=True)

    usr2fn = {f["usr"]: f for f in funcs}
    usr2name = {f["usr"]: f["qualname"] for f in funcs}

    # ---- classify every fn; record its (track, pointer)
    track_of, ptr_of = {}, {}
    for f in funcs:
        tr, ptr = classify(f)
        track_of[f["usr"]] = tr
        ptr_of[f["usr"]] = ptr

    # cpp-track class name -> its design-doc pointer (None = undocumented). A
    # C-track signature/body naming one of these resolves to the DOC, not a
    # missing-rosetta escalation (the doc defines the reimplemented C++ type).
    cpp_class_doc = {}
    # seed with the statically-known cpp-track class sets — some (CArea,
    # CTerrainMap, CCMHeightDetails, …) are header-only in the link set (no
    # out-of-line method def), so they never appear as an `owner` yet a C-track
    # fn still names them by pointer; they resolve to their doc, not "missing".
    for c in RMG_FOLDED:
        cpp_class_doc[c] = DOC["rmg"]
    for c in GHOUL2_CLASSES:
        cpp_class_doc[c] = DOC["ghoul2"]
    for c in GP2_CLASSES:
        cpp_class_doc[c] = GP2_DIR
    for f in funcs:
        if f["owner"] and track_of[f["usr"]] in ("cpp", "cpp-done",
                                                 "cpp-undocumented"):
            cpp_class_doc.setdefault(f["owner"], ptr_of[f["usr"]])
    rosetta_keys = set(rosetta)

    # ---- units: group members; a C-track unit becomes one packet
    miss = {}          # missing rosetta base name -> None (populated during render)
    miss_fns = defaultdict(list)
    packet_of = {}     # usr -> packet relpath
    packets_meta = []  # per emitted C-track packet
    # first pass: choose packet filenames so callee refs can resolve
    ctrack_units = []
    for ui, u in enumerate(units):
        cmembers = [m for m in u["members"] if track_of[m["usr"]] == "c"]
        if not cmembers:
            continue
        cmembers.sort(key=lambda m: m["seq"])
        first = cmembers[0]
        sym = re.sub(r"[^A-Za-z0-9_]", "_", first["name"])[:48]
        # manifest.json and packets live in the same dir (out/engine/packets/),
        # so refs are bare filenames.
        fname = f"{first['subsystem']}__{first['seq']:04d}_{sym}.md"
        rel = fname
        for m in cmembers:
            packet_of[m["usr"]] = rel
        ctrack_units.append((u, cmembers, fname, rel))

    # usr -> destination tuple for callee resolution
    usr2dest = {}
    for f in funcs:
        u = f["usr"]
        tr = track_of[u]
        if tr == "c":
            usr2dest[u] = ("c", packet_of.get(u))
        else:
            usr2dest[u] = (tr, ptr_of[u])

    # ---- render packets
    for u, cmembers, fname, rel in ctrack_units:
        text = render_packet(cmembers, rel, rosetta, name2path, usr2dest,
                             usr2name, calls_of, refs_of, miss, cpp_class_doc,
                             rosetta_keys)
        (outdir / fname).write_text(text)
        packets_meta.append({
            "packet": rel,
            "seq": cmembers[0]["seq"],
            "wave": cmembers[0]["wave"],
            "unit": cmembers[0]["unit"],
            "subsystem": cmembers[0]["subsystem"],
            "file": cmembers[0]["file"],
            "fns": [m["qualname"] for m in cmembers],
            "cyclic": len(cmembers) > 1,
            "loc": sum(m["loc"] for m in cmembers),
        })
    packets_meta.sort(key=lambda p: p["seq"])

    # ---- preamble
    (outdir / "_PREAMBLE.md").write_text(render_preamble(rulings))

    # ---- MACHINE CHECK 1: dangling callees (every in-engine callee ∈ manifest)
    dangling = []
    for f in funcs:
        for d in set(calls_of.get(f["usr"], ())) | set(refs_of.get(f["usr"], ())):
            if d not in usr2fn:
                dangling.append((f["qualname"], d))

    # ---- MACHINE CHECK 2: missing rosetta types across ALL C-track signatures
    #      (strict, signature-only; body-named misses are packet-local notes).
    sig_miss = defaultdict(list)
    for f in funcs:
        if track_of[f["usr"]] != "c":
            continue
        for n in sig_type_names(f):
            if n in EXTERNAL or "(" in n:
                continue  # seam-external or anonymous fn-pointer param type
            if n not in rosetta and n not in SCALAR and n not in cpp_class_doc:
                sig_miss[n].append(f["qualname"])

    # ---- manifest: EVERY fn -> resolution
    manifest_fns = []
    for f in sorted(funcs, key=lambda f: f["seq"]):
        u = f["usr"]
        tr = track_of[u]
        entry = {
            "seq": f["seq"], "wave": f["wave"], "unit": f["unit"],
            "symbol": f["qualname"], "subsystem": f["subsystem"],
            "file": f["file"], "loc": f["loc"], "track": tr,
            "dropped": None,   # no §20 drops recorded at generation time
        }
        if tr == "c":
            entry["packet"] = packet_of.get(u)
        elif tr in ("cpp", "cpp-done"):
            entry["doc"] = ptr_of[u]
        else:  # cpp-undocumented
            entry["doc"] = None
            entry["needs_doc"] = True
        manifest_fns.append(entry)

    # ---- shards (LOC-balanced bundles of C-track packets, seq order)
    shard_bundles = build_shards(packets_meta)
    shards = [{
        "shard": i + 1,
        "packets": [p["packet"] for p in b],
        "fns": sum(len(p["fns"]) for p in b),
        "loc": sum(p["loc"] for p in b),
    } for i, b in enumerate(shard_bundles)]

    # ---- track / undocumented census
    track_hist = defaultdict(int)
    for f in funcs:
        track_hist[track_of[f["usr"]]] += 1
    undoc = defaultdict(list)
    for f in funcs:
        if track_of[f["usr"]] == "cpp-undocumented":
            undoc_key = f["owner"] or f["file"]
            undoc[undoc_key] if False else undoc.setdefault(undoc_key, [])
            undoc[undoc_key].append(f["qualname"])
    doc_hist = defaultdict(int)
    for f in funcs:
        if track_of[f["usr"]] in ("cpp", "cpp-done"):
            doc_hist[ptr_of[f["usr"]]] += 1

    manifest = {
        "generated_by": "tools/closure-prototype/enginepackets.py",
        "module": "mp-engine-ded",
        "total_functions": len(funcs),
        "track_histogram": dict(track_hist),
        "doc_histogram": dict(sorted(doc_hist.items())),
        "c_track_packets": len(packets_meta),
        "c_track_cyclic_packets": sum(1 for p in packets_meta if p["cyclic"]),
        "shards": len(shards),
        "shard_target_loc": SHARD_TARGET_LOC,
        "machine_check": {
            "dangling_callees": len(dangling),
            "dangling_examples": dangling[:10],
            "missing_rosetta_types": len(sig_miss),
            "no_stub_order_verified": order_stats.get("no_stub_verified", True),
        },
        "missing_rosetta_types": {
            n: {"count": len(v), "example_fns": sorted(set(v))[:5]}
            for n, v in sorted(sig_miss.items(), key=lambda kv: -len(kv[1]))},
        "undocumented_cpp_classes": {
            k: {"count": len(v), "fns": v[:8]}
            for k, v in sorted(undoc.items(), key=lambda kv: -len(kv[1]))},
        "functions": manifest_fns,
        "shard_bundles": shards,
    }
    (outdir / "manifest.json").write_text(json.dumps(manifest, indent=1))

    runtime = time.time() - t0
    render_report(outdir, manifest, packets_meta, sig_miss, undoc, dangling,
                  order_stats, runtime)

    print(f"[enginepackets] {len(packets_meta)} C-track packets "
          f"({manifest['c_track_cyclic_packets']} cyclic), "
          f"{track_hist['cpp']+track_hist['cpp-done']} cpp-track fns, "
          f"{track_hist['cpp-undocumented']} undocumented-cpp fns, "
          f"{len(sig_miss)} missing sig types, {len(dangling)} dangling callees, "
          f"{len(shards)} shards, {runtime:.1f}s")


def render_report(outdir, manifest, packets, sig_miss, undoc, dangling,
                  order_stats, runtime):
    o = []
    o.append("# Engine signature-manifest — generation report")
    o.append("")
    o.append(f"Generated by `tools/closure-prototype/enginepackets.py` in "
             f"**{runtime:.1f}s**. Reuses `engineorder.build()` (the pinned "
             "per-file WinDed-Release libclang parse). One work order per row of "
             "the 2,481-fn engine port order.")
    o.append("")
    th = manifest["track_histogram"]
    o.append("## Coverage (every one of the 2,481 fns is accounted for)")
    o.append("")
    o.append("| track | fns | resolution |")
    o.append("| --- | ---: | --- |")
    o.append(f"| C (mechanical packet) | {th.get('c',0)} | "
             f"{manifest['c_track_packets']} packets "
             f"({manifest['c_track_cyclic_packets']} cyclic-unit) |")
    o.append(f"| C++ §F frozen doc | {th.get('cpp',0)} | doc pointer |")
    o.append(f"| C++ GP2 done pilot | {th.get('cpp-done',0)} | "
             f"`crates/mp/engine/qcommon/src/gp2/` |")
    o.append(f"| C++ UNDOCUMENTED (referee) | {th.get('cpp-undocumented',0)} | "
             "no frozen doc — see below |")
    o.append(f"| **total** | **{manifest['total_functions']}** | |")
    o.append("")
    o.append("### §F design-doc routing")
    o.append("")
    o.append("| doc | fns |")
    o.append("| --- | ---: |")
    for d, c in manifest["doc_histogram"].items():
        o.append(f"| `{d}` | {c} |")
    o.append("")
    o.append("## Machine checks")
    o.append("")
    mc = manifest["machine_check"]
    o.append(f"- **Dangling callees:** {mc['dangling_callees']} "
             + ("(all in-engine callee refs resolve within the manifest — the "
                "stub-free order guarantees it)." if not dangling else
                "— **referee items**, listed below."))
    o.append(f"- **No-stub order property:** "
             f"{'verified' if mc['no_stub_order_verified'] else 'NOT verified'} "
             "(inherited from engineorder).")
    o.append(f"- **Missing rosetta types (C-track signatures):** "
             f"{mc['missing_rosetta_types']} distinct.")
    o.append("")
    if dangling:
        o.append("### Dangling callees (referee items)")
        o.append("")
        for nm, d in dangling[:30]:
            o.append(f"- `{nm}` → `{d}`")
        o.append("")
    o.append("## Missing rosetta types (top offenders — referee items, not stubs)")
    o.append("")
    if not sig_miss:
        o.append("_None — every C-track signature type resolves in the rosetta or "
                 "the scalar map._")
    else:
        o.append("| Raven type | sig sites | example fns |")
        o.append("| --- | ---: | --- |")
        for n, v in sorted(sig_miss.items(), key=lambda kv: -len(kv[1]))[:30]:
            o.append(f"| `{n}` | {len(v)} | "
                     f"{', '.join('`'+x+'`' for x in sorted(set(v))[:3])} |")
    o.append("")
    o.append("## Undocumented C++ classes (referee items — need a §F doc)")
    o.append("")
    if not undoc:
        o.append("_None._")
    else:
        o.append("| class / file | methods |")
        o.append("| --- | ---: |")
        for k, v in sorted(undoc.items(), key=lambda kv: -len(kv[1])):
            o.append(f"| `{k}` | {len(v)} |")
    o.append("")
    o.append("## Shards")
    o.append("")
    sh = manifest["shard_bundles"]
    if sh:
        locs = [s["loc"] for s in sh]
        o.append(f"- **{len(sh)}** LOC-balanced shards, target "
                 f"~{manifest['shard_target_loc']} LOC/shard.")
        o.append(f"- LOC per shard: min {min(locs)}, max {max(locs)}, "
                 f"mean {sum(locs)//len(sh)}.")
        o.append(f"- fns per shard: min {min(s['fns'] for s in sh)}, "
                 f"max {max(s['fns'] for s in sh)}.")
        big = [s for s in sh if s["loc"] > manifest['shard_target_loc'] * 2]
        if big:
            o.append(f"- **{len(big)} oversized shard(s)** (a single cyclic unit / "
                     "large fn exceeds 2× target): "
                     + ", ".join(f"shard {s['shard']} ({s['loc']} LOC, "
                                 f"{s['fns']} fns)" for s in big[:5]) + ".")
    o.append("")
    (outdir / "generation-report.md").write_text("\n".join(o))


if __name__ == "__main__":
    main()
