#!/usr/bin/env python3
"""PROTOTYPE — throwaway. Per-FILE port packets for the jampgame mega-pass:
one self-contained markdown work order per oracle .c file, so a porter reads
ONLY its packet + its skeleton module and writes Rust — zero oracle access,
zero exploration.

Packet layout (in order):
  (a) header — the file's manifest stats (fns, LOC, SCC memberships, waves)
  (b) RULINGS DIGEST — verbatim from docs/handoffs/jampgame-fork-discovery.md
      (script-extracted from the '## Fork classes' heading to EOF; never
      paraphrased)
  (c) the COMPLETE oracle source of the .c file, line-numbered
  (d) the manifest rows for this file's functions (callee buckets, globals,
      fn-ptr writes, statics)
  (e) resolved Rust signatures for every OUT-OF-FILE symbol the file calls:
      in-module/bg from the manifest via the fnskel rules table; traps from
      docs/abi-traps.md + the fngen wrapper name; libc/other listed;
      unresolvable symbols marked UNPORTED (porter parks per protocol)

Usage:
  .venv/bin/python fnpacket.py [--manifest out/jampgame-fn-manifest.json]
                               [--out out/packets] [--only g_combat.c]
"""
import argparse
import json
import re
import sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import closure as C
import fnskel
from fngen import parse_trap_tokens, parse_abi_traps

REPO = C.REPO
GAME = C.ORACLE / "codemp" / "game"
RULINGS_DOC = REPO / "docs" / "handoffs" / "jampgame-fork-discovery.md"


def extract_rulings():
    """Verbatim slice of the rulings doc from '## Fork classes' to EOF (the
    numbered rulings + riders + bless-the-rule appendix; intro dropped)."""
    text = RULINGS_DOC.read_text()
    i = text.index("## Fork classes")
    return text[i:].rstrip()


def numbered_source(path):
    lines = path.read_text(errors="replace").splitlines()
    return "\n".join(f"{n:>5} | {ln}" for n, ln in enumerate(lines, 1))


def one_line_signature(fn, ported):
    """Compact one-line faithful Rust signature via the fnskel rules table."""
    parts = []
    for p in fn["params"]:
        pname = fnskel.safe_param_name(p["name"])
        t, un = fnskel.map_ctype(p["type"], ported)
        parts.append(f"{pname}: {t}" + (f" /*TODO: Port {un}*/" if un else ""))
    ret, un = fnskel.map_ctype(fn["ret_type"], ported, as_return=True)
    tail = f" -> {ret}" if ret is not None else ""
    if un:
        tail += f" /*TODO: Port {un}*/"
    if fn.get("variadic"):
        parts.append("/* ... C varargs */")
    return f"pub fn {fn['name']}({', '.join(parts)}){tail}"


def callee_file(entry):
    """Defining file basename from a manifest callee cite."""
    if not entry.get("cite"):
        return None
    return Path(entry["cite"].rsplit(":", 1)[0]).name


def render_fn_row(f):
    o = [f"### `{f['name']}` — {f['file']}:{f['line']}-{f['end_line']} "
         f"({f['loc']} LOC, wave {f['wave']}"
         + (", file-static" if f["static"] else "") + ")"]
    sig = f"{f['ret_type']} {f['name']}(" + ", ".join(
        f"{p['type']} {p['name']}".strip() for p in f["params"]) + \
        (", ..." if f.get("variadic") else "") + ")"
    o.append(f"- C: `{sig}`")
    for bucket, label in (("in-module", "in-module callees"),
                          ("bg", "bg_ callees"), ("syscall", "traps"),
                          ("libc/other", "libc/other")):
        names = [c["name"] for c in f["callees"][bucket]]
        if names:
            o.append(f"- {label}: " + ", ".join(f"`{n}`" for n in names))
    if f["globals_read"]:
        o.append("- globals read: " + ", ".join(
            f"`{g['name']}`" for g in f["globals_read"]))
    if f["globals_write"]:
        o.append("- globals written: " + ", ".join(
            f"`{g['name']}`" for g in f["globals_write"]))
    if f["statics"]:
        o.append("- fn-scope statics (ruling 5): " + ", ".join(
            f"`{s['type']} {s['name']}`" for s in f["statics"]))
    if f["fnptr_writes"]:
        o.append("- fn-ptr writes (ruling 2): " + ", ".join(
            f"`{w['field']} = {w['target']}`" for w in f["fnptr_writes"]))
    if f["stored_as_fnptr"]:
        o.append("- **stored as a fn pointer** — needs an EntXxx enum variant "
                 "(see out/gen/ent_fn_enums.rs)")
    return "\n".join(o)


def build_packet(cfile, fns, ctx):
    """ctx: dict with rulings, byname (name,file)->fn, byname_any name->fn,
    ported, trap_tokens, trap_sigs, scc_names."""
    src_path = GAME / cfile
    total_loc = sum(f["loc"] for f in fns)
    waves = sorted({f["wave"] for f in fns})
    o = []

    # ------------------------------------------------------------ (a) header
    o.append(f"# PORT PACKET (per-file): `{cfile}` — jampgame (mp-game)")
    o.append("")
    o.append("Self-contained work order: this packet + your skeleton module "
             "(`out/skel/" + cfile[:-2] + ".rs`) is everything you read. "
             "NEVER open oracle/ or other crates — every needed symbol is "
             "resolved below; anything marked UNPORTED is parked with a "
             "`//TODO: Port <subject>` marker per protocol.")
    o.append("")
    o.append(f"- functions: **{len(fns)}**, LOC: **{total_loc}**")
    o.append(f"- waves (SCC-condensed, deps-first): "
             f"{waves[0]}–{waves[-1]}" if len(waves) > 1 else
             f"- wave: {waves[0]}")
    scc_lines = []
    for f in fns:
        names = ctx["scc_names"].get(f["scc"])
        if names and len(names) > 1:
            outside = [n for n in names if n != f["name"]]
            scc_lines.append(f"  - `{f['name']}` ↔ {', '.join(f'`{n}`' for n in outside)}")
    if scc_lines:
        o.append("- **mutual-recursion SCC memberships** (port together / "
                 "stub-first):")
        o.extend(sorted(set(scc_lines)))
    else:
        o.append("- SCCs: all singletons (no mutual recursion in this file)")
    o.append("")

    # ------------------------------------------------- (b) rulings digest
    o.append("## RULINGS DIGEST (verbatim, docs/handoffs/jampgame-fork-discovery.md)")
    o.append("")
    o.append(ctx["rulings"])
    o.append("")

    # ------------------------------------------------------ (c) full source
    o.append(f"## COMPLETE ORACLE SOURCE — `oracle/oracle/codemp/game/{cfile}`")
    o.append("")
    o.append("```c")
    o.append(numbered_source(src_path))
    o.append("```")
    o.append("")

    # ------------------------------------------------- (d) manifest rows
    o.append(f"## MANIFEST ROWS ({len(fns)} functions, source order)")
    o.append("")
    for f in fns:
        o.append(render_fn_row(f))
        o.append("")

    # --------------------------------------- (e) out-of-file call surface
    in_mod, bg, hdr, traps, libc = {}, {}, {}, set(), set()
    for f in fns:
        for c in f["callees"]["in-module"]:
            cf = callee_file(c)
            if cf and cf != cfile:
                # header-defined `static ID_INLINE` helpers (VectorLength,
                # BG_GiveMeVectorFromMatrix, …) — not UNPORTED; include source
                if cf.endswith(".h"):
                    hdr[c["name"]] = c
                else:
                    in_mod[(c["name"], cf)] = c
        for c in f["callees"]["bg"]:
            cf = callee_file(c)
            if cf and cf != cfile:
                if cf.endswith(".h"):
                    hdr[c["name"]] = c
                else:
                    bg[(c["name"], cf)] = c
        for c in f["callees"]["syscall"]:
            traps.add(c["name"])
        for c in f["callees"]["libc/other"]:
            libc.add(c["name"])

    o.append("## OUT-OF-FILE CALL SURFACE (resolved — do not explore)")
    o.append("")

    def sig_block(title, entries, note):
        o.append(f"### {title} ({len(entries)})")
        o.append(note)
        o.append("")
        if not entries:
            o.append("_None._")
            o.append("")
            return
        o.append("```rust")
        for (name, cf) in sorted(entries):
            fn = ctx["byname"].get((name, cf))
            if fn is None:
                o.append(f"// UNPORTED `{name}` ({cf}) — park per protocol:")
                o.append(f"//TODO: Port {name}")
                o.append(f"// Source: oracle/oracle/codemp/game/{cf}")
                continue
            o.append(f"// {cf}:{fn['line']}  (skeleton: out/skel/{cf[:-2]}.rs)")
            o.append(one_line_signature(fn, ctx["ported"]) + ";")
        o.append("```")
        o.append("")

    sig_block("In-module functions defined in OTHER game files", in_mod,
              "Faithful skeleton signatures (fnskel rules table). Call them; "
              "their bodies are other porters' packets.")
    sig_block("bg_ functions (shared bg tier)", bg,
              "bg_*.c is ported with this module (hard co-dependency). "
              "Staged skeleton signatures below; UNPORTED rows park.")

    o.append(f"### Header-inline helpers ({len(hdr)})")
    o.append("`static ID_INLINE` functions defined in shared headers — port "
             "inline per the rulings (ignore any `#ifdef _XBOX` asm branch; "
             "the plain-C branch is the compiled one). Verbatim source:")
    o.append("")
    if not hdr:
        o.append("_None._")
        o.append("")
    for name in sorted(hdr):
        c = hdr[name]
        path, span = c["cite"].rsplit(":", 1)
        a, b = (int(x) for x in span.split("-")) if "-" in span else (
            int(span), int(span))
        lines = (REPO / path).read_text(errors="replace").splitlines()
        o.append(f"#### `{name}` — `{c['cite']}`")
        o.append("```c")
        o.append("\n".join(f"{n:>5} | {lines[n - 1]}"
                           for n in range(a, min(b, len(lines)) + 1)))
        o.append("```")
        o.append("")

    o.append(f"### Engine traps ({len(traps)})")
    o.append("Call via the pre-generated SEAM-D13 wrappers "
             "(`out/gen/trap_wrappers.rs`): `trap::<Name>(engine, args)`. "
             "C signatures from `docs/abi-traps.md`:")
    o.append("")
    for t in sorted(traps):
        sig = ctx["trap_sigs"].get(t)
        tok = ctx["trap_tokens"].get(t)
        wrapper = t[len("trap_"):]
        if sig:
            o.append(f"- `{sig['return']} {t}({sig['params']})` → wrapper "
                     f"`trap::{wrapper}`"
                     + (f" (`{tok[0]}`)" if tok else ""))
        elif tok:
            o.append(f"- `{t}` → wrapper `trap::{wrapper}` (`{tok[0]}`) — not "
                     f"in abi-traps.md")
        else:
            o.append(f"- `{t}` — MODULE-LOCAL helper (defined in game/*.c, "
                     f"see its own packet), not a seam wrapper")
    o.append("")

    o.append(f"### libc / SDK ({len(libc)})")
    o.append("Standard C library — use the Rust std equivalent per the house "
             "rules; no signature needed.")
    o.append("")
    if libc:
        o.append(", ".join(f"`{n}`" for n in sorted(libc)))
    else:
        o.append("_None._")
    o.append("")
    return "\n".join(o)


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    here = Path(__file__).resolve().parent
    ap.add_argument("--manifest", default=str(here / "out" / "jampgame-fn-manifest.json"))
    ap.add_argument("--out", default=str(here / "out" / "packets"))
    ap.add_argument("--only", help="emit a single file's packet (basename.c)")
    ap.add_argument("--refresh-stats",
                    default=str(here / "out" / "jampgame-fn-stats.md"))
    args = ap.parse_args()

    manifest = json.loads(Path(args.manifest).read_text())
    funcs = manifest["functions"]
    by_file = defaultdict(list)
    for f in funcs:
        by_file[f["file"]].append(f)
    for fns in by_file.values():
        fns.sort(key=lambda f: f["line"])

    scc_names = defaultdict(list)
    for f in funcs:
        scc_names[f["scc"]].append(f["name"])

    ctx = {
        "rulings": extract_rulings(),
        "byname": {(f["name"], f["file"]): f for f in funcs},
        "ported": C.scan_ported("mp", "game")[0],
        "trap_tokens": parse_trap_tokens(),
        "trap_sigs": parse_abi_traps(),
        "scc_names": scc_names,
    }

    outdir = Path(args.out)
    outdir.mkdir(parents=True, exist_ok=True)
    sizes = {}
    targets = sorted(by_file) if not args.only else [args.only]
    for cfile in targets:
        pkt = build_packet(cfile, by_file[cfile], ctx)
        p = outdir / f"{cfile[:-2]}.md"
        p.write_text(pkt)
        sizes[cfile] = len(pkt.encode())
    tot = sum(sizes.values())
    ordered = sorted(sizes.values())
    print(f"[fnpacket] wrote {len(sizes)} packets to {outdir} "
          f"({tot / 1e6:.1f} MB total)")
    if ordered:
        print(f"[fnpacket] bytes min={ordered[0]:,} "
              f"median={ordered[len(ordered) // 2]:,} max={ordered[-1]:,} "
              f"(~tokens/4: {ordered[0] // 4:,} / "
              f"{ordered[len(ordered) // 2] // 4:,} / {ordered[-1] // 4:,})")

    # ---- refresh the packets section of the stats doc
    sp = Path(args.refresh_stats)
    if not args.only and sp.exists() and ordered:
        md = sp.read_text()
        marker = "\n## Per-file packets & pre-generated boilerplate"
        if marker in md:
            md = md[:md.index(marker)].rstrip() + "\n"
        big = sorted(sizes.items(), key=lambda kv: -kv[1])[:5]
        add = [
            "", "## Per-file packets & pre-generated boilerplate", "",
            f"`fnpacket.py` wrote {len(sizes)} self-contained per-file packets "
            f"to `out/packets/` ({tot / 1e6:.1f} MB total); `fngen.py` staged "
            "deterministic boilerplate in `out/gen/` (game_cvars.rs, "
            "ent_fn_enums.rs, trap_wrappers.rs).", "",
            f"- packet bytes: min {ordered[0]:,} / median "
            f"{ordered[len(ordered) // 2]:,} / max {ordered[-1]:,} "
            f"(~tokens ≈ bytes/4)",
            "- largest packets: " + ", ".join(
                f"`{k}` ({v / 1e3:.0f} KB)" for k, v in big), "",
            "### Known self-containment gaps (porter protocol: park, never "
            "explore)", "",
            "- **Out-of-file macros/constants** (`MAX_GENTITIES`, damage "
            "flags, …) are not resolved into packets; the type port already "
            "landed them in crates/, and unresolved ones park as "
            "`//TODO: Port <NAME>`.",
            "- **Struct field layouts** (`gentity_t` members, …) are not in "
            "packets — ported types in crates/ are the ambient reference; the "
            "packet resolves *call* surface only.",
            "- **Non-cvar globals defined in other files** (`level`, "
            "`g_entities`, …) appear by name in the manifest rows; their "
            "GameWorld placement is ruling 1 (digest included), and cvar "
            "handles map via `out/gen/game_cvars.rs`.",
            "- **fn-ptr census delta**: the discovery doc counted ~213 targets "
            "(incl. compile-time initializer refs); the manifest census (201) "
            "covers runtime body assignments only — initializer-stored "
            "targets surface at Land when the owning tables are ported.",
            "- `trap_Cvar_VariableValue` is a module-local helper "
            "(g_bot.c:36), not a seam trap — no wrapper generated; ported as "
            "a normal function.",
        ]
        sp.write_text(md.rstrip() + "\n" + "\n".join(add) + "\n")
        print(f"[fnpacket] refreshed packets section in {sp}")


if __name__ == "__main__":
    main()
