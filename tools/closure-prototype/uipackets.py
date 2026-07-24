#!/usr/bin/env python3
"""PROTOTYPE — throwaway. UI logic-port packets, built on the packets3.py
pattern (one work order per file-per-wave slice of the ui module), for the
client-port U4 function-port stage.

The ui port is WAVE-GATED (ui-plan §"Minimal-deferral strategy" step 3):
leaves first, a fn enters a wave only when every in-module callee sits in a
lower wave. So packets are sliced per (wave, file) — one coherent packet holds
one file's open fns from ONE wave — and sharded (LOC-balanced contiguous fn
ranges) the way packets3 shards large files. Port wave 0 fully, gate, wave 1, …

Each packet carries, per the U0 packet-generator agenda:
 (a) the ui-plan MARKER LAW + TRANSLATION DICTIONARY (sliced verbatim from
     docs/plans/2026-07-24-client-port/ui-plan.md — authoritative, not retyped);
 (b) a THREADING DIGEST PER FN naming the state channel in PLACEHOLDER terms
     (UiWorld / UiContext / DisplayContext / MenuSystem — see the CONFIG block;
     field-level bindings are finalized at the U2 root-type sit-down and are
     INTENTIONALLY deferred here);
 (c) the cited verbatim oracle source slice of each fn + its oracle C signature;
 (d) resolved signatures of out-of-module callees — bg fns from crates/mp/bg
     (LAW, ported), trap wrappers via crates/mp/abi/src/ui (the syscall ABI
     token + the ui_syscalls.c C contract; the ERGONOMIC `trap_*` wrapper does
     not exist yet → MISSING-WRAPPER, U1 builds it over the present token);
 (e) a STATICS-TO-FOLD list per fn — function-scope statics + file-scope globals
     it touches (these fold into UiWorld / subsystem fields at U2).

Emits out/ui/packets/ + out/ui/packets-manifest.json.

Usage:
  .venv/bin/python uipackets.py                 # all waves, all files
  .venv/bin/python uipackets.py --wave 0        # only wave-0 packets
  .venv/bin/python uipackets.py --only ui_gameinfo.c
  .venv/bin/python uipackets.py --wave 0 --only ui_gameinfo.c   # union
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
UIDIR = ORACLE / "codemp" / "ui"
BG_SRC = REPO / "crates" / "mp" / "bg" / "src"
ABI_UI = REPO / "crates" / "mp" / "abi" / "src" / "ui"
UI_PLAN = REPO / "docs" / "plans" / "2026-07-24-client-port" / "ui-plan.md"
MANIFEST = HERE / "out" / "ui" / "ui-fn-manifest.json"
OUT = HERE / "out" / "ui"

# packets3's shard caps — a (wave, file) slice shards past either.
SHARD_MAX_FNS = 35
SHARD_MAX_LOC = 3000

# =============================================================================
# CONFIG — placeholder root-type bindings (finalized at the U2 sit-down)
# -----------------------------------------------------------------------------
# The ui root types are NOT designed yet (ui-plan stage U2 ratifies them before
# any transcription). These packets thread state in PLACEHOLDER terms only; the
# FIELD-LEVEL mapping (which static lands on which struct field) is intentionally
# deferred. When U2 settles, update these four bindings + regenerate.
#
#   UiWorld        — the owned ui state spine (uiInfo_t + ui_force.c globals +
#                    per-file statics folded in). Analog of GameWorld.
#   UiContext      — the threaded handle the vmMain entrypoints own and pass
#                    inward ({ world: &mut UiWorld, engine, menus, dc, … }).
#                    Analog of GameContext.
#   DisplayContext — the ~50-fn-pointer render/text/cvar/feeder/ownerDraw/sound
#                    vtable (Raven displayContextDef_t), as an idiomatic trait
#                    (REPLACE-vs-WRAP decided at U2).
#   MenuSystem     — ui_shared.c's owned menu framework (menuDef/itemDef arena +
#                    indices, String_Alloc intern pool → owned table, open-menu
#                    stack as indices). Owned by composition: UiWorld.menus.
UI_WORLD = "UiWorld"
UI_CONTEXT = "UiContext"
DISPLAY_CONTEXT = "DisplayContext"
MENU_SYSTEM = "MenuSystem"
CONFIG_FINALIZED = True  # U2 ratified 2026-07-24 (DEC-36, D1-D8)
# =============================================================================

# bg_lib.c-origin callees that are NOT mp_bg fns — Raven's own libc shims. They
# resolve to native/std, not a bg signature (noted, never MISSING-WRAPPER).
BG_LIB_SHIMS = {
    "atof": "Rust `str::parse::<f32>()` / the `Q_atof` helper",
    "atoi": "Rust `str::parse::<i32>()` / the `Q_atoi` helper",
    "qsort": "`native_sort` (DEC-34 canonical qsort)",
    "rand": "`BgState.rng` (never libc `rand` — parity-visible; ui reaches the "
            "one generator through its bg/callback channel)",
}


# --------------------------------------------------------- source-slice helpers
_SRC = {}


def numbered_slice(cfile, a, b):
    lines = _SRC.get(cfile)
    if lines is None:
        lines = (UIDIR / cfile).read_text(errors="replace").splitlines()
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


def load_trap_seam():
    """(c_sig, enum, tokens): trap_* -> its ui_syscalls.c C contract and the
    UI_* syscall enum it dispatches; plus the set of abi syscall tokens that
    already exist under crates/mp/abi/src/ui/syscalls/."""
    txt = strip_c_comments((UIDIR / "ui_syscalls.c").read_text(errors="replace"))
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
    tokens = {p.stem for p in (ABI_UI / "syscalls").glob("UI_*.rs")}
    return c_sig, enum, tokens


# ----------------------------------------------------------- ui-plan law slice
def plan_digest():
    """The MARKER LAW + TRANSLATION DICTIONARY, sliced verbatim from the ui-plan
    (authoritative source, never retyped here)."""
    t = UI_PLAN.read_text()
    a = t.index("## Marker law")
    b = t.index("## Minimal-deferral strategy")
    return t[a:b].rstrip()


# ------------------------------------------------------------- sharding (packets3)
def shard_chunks(fns):
    """Contiguous, LOC-balanced shards; splits past SHARD_MAX_FNS or
    SHARD_MAX_LOC. Verbatim policy from packets3.shard_chunks."""
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
def render_packet(cfile, wave, chunk, shard, n_shards, law, bg_sigs, trap_seam,
                  inmod_sig):
    c_sig, enum, tokens = trap_seam
    own = {f["name"] for f in chunk}
    o = []
    title = f"{cfile} — wave {wave}" + (f" — shard {shard}/{n_shards}"
                                        if shard else "")
    o.append(f"# UI PORT PACKET: `{title}`")
    o.append("")
    o.append(f"Fill the **{len(chunk)}** open functions below — one file's open "
             f"fns from **wave {wave}** of the topological partition "
             "(`out/ui/ui-wave-partition.json`). Every in-module callee these "
             f"fns need was ported in a **lower wave (< {wave})**; bg fns and the "
             "trap seam already exist. Transcribe directly to the idiomatic shape "
             "(ui-plan ruling 2026-07-24: blind-faithful pass retired). Where a "
             "genuine question remains leave a cited `// DEFERRED:` or a "
             "`// PORT-NOTE:` at the site — NEVER a bare `//TODO: Port` (a wave "
             "that adds one fails review).")
    o.append("")
    o.append(f"- open fns: **{len(chunk)}**  ·  oracle LOC: "
             f"**{sum(f['loc'] for f in chunk)}**  ·  wave: **{wave}**  ·  "
             f"file: `{cfile}`")
    o.append("")

    # ---- CONFIG / root-type bindings (DEC-36, ratified 2026-07-24)
    o.append("## ROOT-TYPE BINDINGS (RATIFIED — DEC-36, U2 sit-down 2026-07-24)")
    o.append("")
    o.append("The ui root types are RATIFIED (D1-D8; `docs/decisions.md` "
             "DEC-36). Transcribe against these shapes — a shape question a "
             "packet cannot answer is an escalation, never an invention:")
    o.append(f"- **`{UI_CONTEXT}`** (D4) — `{{ world: &mut {UI_WORLD}, engine }}`, "
             "owned by the vmMain entrypoints and passed inward; analog of "
             "`GameContext`. State is THREADED, not reached — no `static mut`, "
             "no ambient cells (the G_SoundIndex lesson is day-one law).")
    o.append(f"- **`{UI_WORLD}`** (D1) — owned ui spine: `uiInfo_t` + ui_force.c "
             "globals + the per-file statics listed under each fn below, folded "
             "into fields — `String`/`bool`/`Vec` throughout (all Class C), "
             "Raven field names kept.")
    o.append(f"- **`{MENU_SYSTEM}`** (D2) — ui_shared.c's owned menu framework: "
             "menuDef/itemDef arena + index handles (no `*mut` graph), "
             "`String_Alloc` intern pool → owned string table, open-menu stack "
             f"as indices; owned by composition `{UI_WORLD}.menus`.")
    o.append(f"- **`{DISPLAY_CONTEXT}`** (D3) — an idiomatic trait REPLACING "
             "Raven's `displayContextDef_t` fn-pointer struct (repr(C) asserts "
             "retired by ruling; cgame later implements the same trait).")
    o.append("- **bg arms** (D5) — bg's `#ifdef WE_ARE_IN_THE_UI`/`UI_EXPORTS` "
             "branches are trait dispatch: ui implements `GameCallbacks` over "
             "its trap layer (sound→`S_RegisterSound`, shader→"
             "`R_RegisterShaderNoMip`); ui reuses mp_bg's animation module "
             "(PORT-NOTE at the `UI_ParseAnimationFile` site).")
    o.append("- **ABI arm** (D6) — legacy `vmMain`+`dllEntry` exports.")
    o.append("- **Dead surface** (D7) — ui_players.c/ui_util.c and ui_main.c's "
             "`UI_DrawOpponent`/`UI_DrawPlayerModel` static family are §20 dead "
             "(not compiled in retail vcproj/q3asm; deleted in OpenJK): drop "
             "with PORT-NOTEs, never port.")
    o.append("")

    # ---- law + dictionary
    o.append("## MARKER LAW + TRANSLATION DICTIONARY (ui-plan — apply verbatim)")
    o.append("")
    o.append(law)
    o.append("")

    # ---- threading digest per fn
    o.append("## THREADING DIGEST — per open fn")
    o.append("")
    for f in chunk:
        body = strip_c_comments("\n".join(
            _SRC.setdefault(cfile, (UIDIR / cfile).read_text(
                errors="replace").splitlines())[f["line"] - 1:f["end_line"]]))
        traps = sorted({c["name"] for c in f["callees"]["syscall"]})
        bgc = sorted({c["name"] for c in f["callees"]["bg"]})
        statics = f.get("statics", [])
        greads = [g["name"] for g in f["globals_read"]]
        gwrites = [g["name"] for g in f["globals_write"]]
        fnptr = f.get("fnptr_writes", [])

        o.append(f"### `{f['name']}` — {cfile}:{f['line']}-{f['end_line']} "
                 f"({f['loc']} LOC, wave {f['wave']})")
        engine = "engine traps (via `%s`)" % UI_CONTEXT if traps else None
        chan = [c for c in (engine,
                            "bg (direct calls into mp_bg)" if bgc else None,
                            f"`{UI_WORLD}` state (statics/globals below)"
                            if (statics or greads or gwrites) else None,
                            f"`{MENU_SYSTEM}`/`{DISPLAY_CONTEXT}` fn-ptr dispatch"
                            if fnptr else None) if c]
        o.append("- **channel:** " + ("; ".join(chan) if chan
                 else "pure fn — no state channel (no traps/globals/bg/fn-ptr)"))
        if traps:
            parts = []
            for t in traps:
                e = enum.get(t)
                tok = "abi token present" if e in tokens else "NO abi token"
                parts.append(f"`{t}` (**MISSING-WRAPPER** — U1 builds it; {tok}"
                             + (f": `{e}`" if e else "") + ")")
            o.append("- **traps → seam:** " + "; ".join(parts))
        if bgc:
            o.append("- **bg calls:** " + ", ".join(f"`{b}`" for b in bgc))
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
            o.append(f"- **statics-to-fold (→ `{UI_WORLD}` fields at U2):** "
                     + "  ·  ".join(fold))
        if fnptr:
            fps = ", ".join(f"`{w['field']}={w['target']}`" for w in fnptr)
            o.append(f"- **fn-ptr dispatch writes:** {fps} — a "
                     f"`{DISPLAY_CONTEXT}`/feeder/ownerDraw vtable slot; assign the "
                     "idiomatic trait method / `match` arm, not a raw fn ptr "
                     "(dictionary: fn-ptr tables → `match`, trait only where the "
                     "set is open).")
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
    bg_calls, trap_calls, inmod, other = set(), set(), {}, set()
    for f in chunk:
        for c in f["callees"]["bg"]:
            bg_calls.add(c["name"])
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

    o.append(f"### trap seam (crates/mp/abi/src/ui) — {len(trap_calls)}")
    o.append(f"Every ui `trap_*` ERGONOMIC wrapper is **MISSING-WRAPPER** — none "
             "exist yet; U1 builds them over the abi syscall token shown. The "
             "ui_syscalls.c C contract below is the LAW arg/return shape (apply "
             "the dictionary at the wrapper: `char*`→`&str`/`String`, "
             "`qboolean`→`bool`, out-params→returns). Call the wrapper U1 lands; "
             "do NOT hand-roll a syscall here.")
    o.append("```c")
    for name in sorted(trap_calls):
        e = enum.get(name)
        tok = "token PRESENT" if e in tokens else "token ABSENT"
        o.append(f"// MISSING-WRAPPER  ·  abi: {e or '?'} ({tok})")
        o.append(c_sig.get(name, f"/* {name}: no C sig found in ui_syscalls.c */"))
    o.append("```")
    o.append("")

    o.append(f"### in-module (ported in a LOWER wave — call directly) — {len(inmod)}")
    o.append("Oracle C signatures (these fns land already-idiomatic in an earlier "
             "wave; match the shape the earlier packet produced):")
    o.append("```c")
    for name in sorted(inmod):
        cite = inmod[name] or "ui/"
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
        chunks = shard_chunks(groups[(wave, cfile)])
        n = len(chunks)
        for si, chunk in enumerate(chunks):
            shard = (si + 1) if n > 1 else None
            base = cfile[:-2]
            text = render_packet(cfile, wave, chunk, shard, n, law,
                                 bg_sigs, trap_seam, inmod_sig)
            fname = f"{base}.wave{wave}" + (f".shard{shard}" if shard else "") + ".md"
            (OUT / "packets" / fname).write_text(text)
            man.append({
                "file": cfile, "wave": wave, "packet": f"packets/{fname}",
                "fns": len(chunk), "loc": sum(f["loc"] for f in chunk),
                **({"shard": shard, "shards_total": n} if shard else {})})

    man.sort(key=lambda e: (e["wave"], e["file"], e.get("shard", 0)))
    (OUT / "packets-manifest.json").write_text(json.dumps(man, indent=1))
    print(f"[uipackets] {len(man)} packets, "
          f"{sum(e['fns'] for e in man)} fns, {sum(e['loc'] for e in man):,} LOC "
          f"-> out/ui/packets/")
    if CONFIG_FINALIZED is False:
        print("[uipackets] NOTE: root-type bindings are PLACEHOLDER (U2 pending)")


if __name__ == "__main__":
    main()
