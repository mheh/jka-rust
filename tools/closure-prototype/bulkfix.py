#!/usr/bin/env python3
"""Bulk mechanical integration fixes, driven by cargo's JSON spans.

Modes (combine freely; --apply actually writes, default is dry-run):

  bulkfix.py --rename-entities g_weapon.rs [...]
      Scoped textual rename of the GameWorld arena access spelling
      `...world).entities` / `world.entities` -> `.g_entities`.
      Regex is anchored on the `world` receiver so unrelated `entities`
      identifiers (locals, params) are never touched.

  bulkfix.py --cast g_weapon.rs [...]
      Span-driven casts for E0308 integer-width / int-enum mismatches:
      runs `cargo check --message-format=json`, collects primary spans in
      the target files whose message is `expected \x60T\x60, found \x60U\x60` where
      T is a primitive int and U is a primitive int or a #[repr(int)] enum
      (`*_t` name), and rewrites the exact span to `(<expr>) as T`.
      Byte-precise (rustc's own span extents), applied bottom-up per file.

  bulkfix.py --overlay
      Span-driven overlay casts for the `*mut c_void` field family
      (E0609 `no field X on type ...c_void...`). Per the settled design
      deferral, `gentity_t.m_pVehicle` / `.client` / `.NPC` stay
      `*mut c_void` (crate cycle); the sanctioned idiom casts at the use
      site: `((*ent).m_pVehicle as *mut Vehicle_t)`. This mode finds each
      E0609-on-c_void primary span, back-parses the receiver, and either
      (a) rewrites an inline deref group `(*EXPR.client).f` ->
          `(*(EXPR.client as *mut gclient_t)).f`, or
      (b) chases a plain-identifier receiver to its `let NAME = ...;`
          binding and appends the cast there (one edit fixes all uses).
      Only the three known origin fields are rewritten; every other
      shape prints SKIP. Runs tree-wide (no file args). Iterate: casts
      can unlock cascading sites, so re-run until it reports 0 edits.

Always verify with cargo afterwards; the tool never touches anything
without a matching rustc diagnostic (--cast, --overlay) or the anchored
pattern (--rename-entities).
"""

import json
import re
import subprocess
import sys
from pathlib import Path

WT = Path("/Users/milohehmsoth/Developer/Milo/jka-rust/.claude/worktrees/agent-a43cc53200d2fdf54")
SRC = WT / "crates/mp/game/src"

# `(*ctx.world).entities`, `world).entities`, `world.entities` — receiver-anchored.
ENTITIES_PAT = re.compile(r"(world\)?)\.entities\b")

PRIM_INTS = {"i8", "u8", "i16", "u16", "i32", "u32", "i64", "u64", "isize", "usize", "c_int"}
EXPECTED_FOUND = re.compile(r"expected `([\w:]+)`, found `([\w:]+)`")


def resolve(names):
    out = []
    for n in names:
        p = Path(n)
        if not p.is_absolute():
            p = SRC / n
        if not p.exists():
            sys.exit(f"no such file: {p}")
        out.append(p)
    return out


def rename_entities(paths, apply):
    for p in paths:
        s = p.read_text()
        hits = ENTITIES_PAT.findall(s)
        if apply and hits:
            p.write_text(ENTITIES_PAT.sub(r"\1.g_entities", s))
        print(f"rename-entities {p.name}: {len(hits)} site(s){'' if apply else ' (dry-run)'}")


def cast_targets(target_paths):
    """cargo check json -> {abs_file: [(byte_start, byte_end, cast_to)]}"""
    targets = {str(p) for p in target_paths}
    res = subprocess.run(
        ["cargo", "check", "-p", "mp_game", "--message-format=json"],
        cwd=WT, capture_output=True, text=True,
    )
    edits = {}
    for line in res.stdout.splitlines():
        try:
            m = json.loads(line)
        except json.JSONDecodeError:
            continue
        if m.get("reason") != "compiler-message":
            continue
        msg = m["message"]
        code = (msg.get("code") or {}).get("code")
        if code != "E0308":
            continue
        for sp in msg.get("spans", []):
            if not sp.get("is_primary"):
                continue
            # rustc puts the `expected T, found U` text in the primary span's
            # label, not the top-level message (which is just "mismatched types").
            mm = EXPECTED_FOUND.search(sp.get("label") or "") or EXPECTED_FOUND.search(msg.get("message", ""))
            if not mm:
                continue
            expected, found = mm.group(1).split("::")[-1], mm.group(2).split("::")[-1]
            # only primitive-int targets; sources: primitive ints or #[repr(int)] enums
            if expected not in PRIM_INTS:
                continue
            if found not in PRIM_INTS and not found.endswith("_t"):
                continue
            f = str(WT / sp["file_name"])
            if f in targets:
                edits.setdefault(f, set()).add((sp["byte_start"], sp["byte_end"], expected))
    return {f: sorted(v) for f, v in edits.items()}


def cast_fixes(paths, apply):
    edits = cast_targets(paths)
    if not edits:
        print("cast: no matching E0308 spans in target files")
        return
    for f, spans in edits.items():
        data = Path(f).read_bytes()
        n = 0
        for start, end, to in sorted(spans, reverse=True):
            expr = data[start:end]
            if b"\n" in expr:  # multi-line span: skip, needs eyes
                print(f"  SKIP multi-line span at byte {start} in {Path(f).name}")
                continue
            data = data[:start] + b"(" + expr + b") as " + to.encode() + data[end:]
            n += 1
        if apply:
            Path(f).write_bytes(data)
        print(f"cast {Path(f).name}: {n} span(s) rewritten{'' if apply else ' (dry-run)'}")


# Overlay-cast origin fields: the only gentity_t members deliberately left
# `*mut c_void` (crate cycle), mapped to their sanctioned concrete cast target.
OVERLAY_ORIGINS = {
    b".m_pVehicle": "mp_bg::vehicles::Vehicle_t",
    b".client": "crate::client::gclient_t",
    b".NPC": "crate::npc::g_npc_t::gNPC_t",
}
NO_FIELD_CVOID = re.compile(r"no field `(\w+)` on type `[^`]*c_void[^`]*`")
IDENT_BYTE = re.compile(rb"[A-Za-z0-9_]")


def overlay_diags():
    """cargo check json -> [(abs_file, line, byte_start, byte_end, field)]"""
    res = subprocess.run(
        ["cargo", "check", "-p", "mp_game", "--message-format=json"],
        cwd=WT, capture_output=True, text=True,
    )
    out = []
    for line in res.stdout.splitlines():
        try:
            m = json.loads(line)
        except json.JSONDecodeError:
            continue
        if m.get("reason") != "compiler-message":
            continue
        msg = m["message"]
        if (msg.get("code") or {}).get("code") != "E0609":
            continue
        if not NO_FIELD_CVOID.search(msg.get("message", "")):
            continue
        for sp in msg.get("spans", []):
            if not sp.get("is_primary") or sp.get("expansion"):
                continue
            f = str(WT / sp["file_name"])
            if "/oracle/" in f:  # never touch the oracle
                continue
            out.append((f, sp["line_start"], sp["byte_start"], sp["byte_end"]))
    return out


def origin_of(expr):
    """expr bytes -> cast type if it ends with a known c_void origin field."""
    for suffix, ty in OVERLAY_ORIGINS.items():
        if expr.endswith(suffix):
            return ty
    return None


def ident_before(data, pos):
    """Identifier ending at byte pos (exclusive), or None."""
    s = pos
    while s > 0 and IDENT_BYTE.match(data[s - 1:s]):
        s -= 1
    return (s, data[s:pos]) if s < pos else None


def paren_group_before(data, pos):
    """Balanced (...) group ending at byte pos (exclusive), or None."""
    if pos <= 0 or data[pos - 1:pos] != b")":
        return None
    depth = 0
    i = pos - 1
    while i >= 0:
        c = data[i:i + 1]
        if c == b")":
            depth += 1
        elif c == b"(":
            depth -= 1
            if depth == 0:
                return (i, data[i:pos])
        i -= 1
    return None


def binding_edit(data, name, use_start, depth=0):
    """Find `let [mut] NAME = RHS;` above use_start; return (start,end,repl) or None.

    Handles RHS shapes `EXPR`, `&*EXPR`, `&mut *EXPR` where EXPR ends with an
    overlay origin field, and chases one level of `let NAME = &*OTHER;`."""
    if depth > 2:
        return None
    pat = re.compile(rb"let\s+(?:mut\s+)?" + re.escape(name) + rb"\s*=\s*([^;\n]+);")
    best = None
    for m in pat.finditer(data, 0, use_start):
        best = m  # nearest binding above the use site
    if not best:
        return None
    rhs, rhs_start = best.group(1), best.start(1)
    if b" as *mut" in rhs or b" as * mut" in rhs:
        return None  # already cast
    stripped, off = rhs, 0
    for pre in (b"&mut *", b"&mut*", b"&*"):
        if rhs.startswith(pre):
            stripped, off = rhs[len(pre):], len(pre)
            break
    ty = origin_of(stripped.rstrip())
    if ty:
        core = stripped.rstrip()
        s = rhs_start + off
        e = s + len(core)
        if off:  # `&*ptr` needs parens: `&*(ptr as *mut T)`
            return (s, e, b"(" + core + b" as *mut " + ty.encode() + b")")
        return (s, e, core + b" as *mut " + ty.encode())
    # chase one hop: `let pVeh = &mut *pVeh;` -> the earlier binding of `pVeh`
    if IDENT_BYTE.fullmatch(stripped.rstrip()[:1] or b"") and re.fullmatch(rb"\w+", stripped.rstrip()):
        return binding_edit(data, stripped.rstrip(), best.start(), depth + 1)
    return None


def overlay_fixes(apply):
    diags = overlay_diags()
    if not diags:
        print("overlay: no E0609-on-c_void spans")
        return
    edits = {}   # file -> {(start,end): replacement}
    skips = []
    stats = {}   # file -> counts
    for f, line_no, bs, _be in diags:
        data = Path(f).read_bytes()
        st = stats.setdefault(Path(f).name, [0, 0])
        if data[bs - 1:bs] != b".":
            skips.append(f"{Path(f).name}:{line_no} field not preceded by `.`")
            continue
        ed = None
        grp = paren_group_before(data, bs - 1)
        if grp and grp[1].startswith(b"(*"):
            # inline deref group: `(*PTR).field` with PTR ending in an origin
            gstart, gtext = grp
            ptr = gtext[2:-1].strip()
            ty = origin_of(ptr)
            if ty:
                ed = (gstart, gstart + len(gtext),
                      b"(*(" + ptr + b" as *mut " + ty.encode() + b"))")
            elif re.fullmatch(rb"\w+", ptr):
                # `(*client).f` where `client` is a local: fix its binding
                ed = binding_edit(data, ptr, gstart)
                if not ed:
                    skips.append(f"{Path(f).name}:{line_no} no castable binding for `{ptr.decode()}`")
            else:
                skips.append(f"{Path(f).name}:{line_no} deref group not an origin: {gtext.decode(errors='replace')[:60]}")
        elif (iv := ident_before(data, bs - 1)):
            _, name = iv
            ed = binding_edit(data, name, bs - 1)
            if not ed:
                skips.append(f"{Path(f).name}:{line_no} no castable binding for `{name.decode()}`")
        else:
            ctx = data[max(0, bs - 40):bs].decode(errors="replace")
            skips.append(f"{Path(f).name}:{line_no} unrecognized receiver ...{ctx}")
        if ed:
            prev = edits.setdefault(f, {}).get((ed[0], ed[1]))
            if prev is None:
                edits[f][(ed[0], ed[1])] = ed[2]
                st[0] += 1
            elif prev == ed[2]:
                st[1] += 1  # dedup (same binding serves many sites)
            else:
                skips.append(f"{Path(f).name}:{line_no} conflicting edit at bytes {ed[0]}-{ed[1]}")
    for f, emap in edits.items():
        data = Path(f).read_bytes()
        for (s, e), repl in sorted(emap.items(), reverse=True):
            data = data[:s] + repl + data[e:]
        if apply:
            Path(f).write_bytes(data)
        u, d = stats[Path(f).name]
        print(f"overlay {Path(f).name}: {u} edit(s), {d} deduped site(s){'' if apply else ' (dry-run)'}")
    for s in skips:
        print(f"  SKIP {s}")


def main():
    args = sys.argv[1:]
    apply = "--apply" in args
    args = [a for a in args if a != "--apply"]
    mode = args[0] if args else ""
    files = resolve(args[1:])
    if mode == "--rename-entities":
        rename_entities(files, apply)
    elif mode == "--cast":
        cast_fixes(files, apply)
    elif mode == "--overlay":
        overlay_fixes(apply)
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
