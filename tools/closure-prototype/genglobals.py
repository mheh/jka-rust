#!/usr/bin/env python3
"""PROTOTYPE — pass-2 deliverable 4: emit `GameGlobals`, the remaining game-tier
mutable file-scope globals/statics as one GameWorld sub-struct (fork ruling 1),
so pass-2 porters never add a field. Scalar decls resolve to their Rust type;
non-scalar (pointer/struct/array) decls land as `()` + `//TODO: Port <ctype>`
per house style (green-safe; the porter fills the real type when porting that
body). bg/qshared-owned globals (fork 8a) and read-only const tables (ruling 5)
are intentionally EXCLUDED — they are not GameWorld state."""
import re, json
from pathlib import Path
import pass2lib as L

RUST_KW = {'move','type','ref','box','final','override','become','yield','macro','virtual','use','fn','let','loop','match','self','crate','super','mod','impl','dyn','async','await','where','while','const','static'}

def fld(n):
    return f'r#{n}' if n in RUST_KW else n

SCALARS = {
    'int': 'c_int', 'unsigned int': 'c_uint', 'unsigned': 'c_uint',
    'short': 'c_short', 'char': 'c_char', 'byte': 'byte',
    'float': 'f32', 'double': 'f64', 'qboolean': 'qboolean',
    'long': 'c_long', 'vec_t': 'vec_t',
}

def decl_line(cite):
    p, ln = cite.rsplit(':', 1)
    ln = int(ln.split('-')[0])
    lines = (L.REPO / p).read_text(errors='replace').splitlines()
    return lines[ln - 1].strip()

def resolve(name, cite):
    """Return (rust_type, init, todo_ctype_or_None)."""
    line = decl_line(cite)
    # strip leading storage/qualifier keywords
    body = re.sub(r'^\s*(static|const|extern|register|volatile)\s+', '', line)
    body = re.sub(r'^\s*(static|const|extern|register|volatile)\s+', '', body)
    # form: <type...> <name> [array]? [= init]? ;
    m = re.search(rf'([\w\s\*]+?)\b{re.escape(name)}\b\s*(\[[^;]*\])?\s*(=|;)', body)
    if not m:
        return '()', '()', line  # unparsed -> placeholder
    ctype = re.sub(r'\s+', ' ', m.group(1)).strip()
    arr = m.group(2)
    if arr is None and '*' not in ctype and ctype in SCALARS:
        rt = SCALARS[ctype]
        return rt, ('QFALSE' if rt == 'qboolean' else '0 as _' if rt in ('f32','f64') else '0'), None
    # non-scalar / pointer / array -> placeholder
    return '()', '()', (ctype + (arr or '') + ('*' if '*' in ctype else ''))

def main():
    m = L.load_manifest(); F = m['functions']
    g_written = set()
    g_cite = {}
    for f in F:
        for g in f['globals_write']:
            g_written.add(g['name'])
        for g in f['globals_read'] + f['globals_write']:
            g_cite.setdefault(g['name'], g['cite'])
    gw = (L.GAME_SRC / 'world' / 'game_world.rs').read_text()
    gc = (L.GAME_SRC / 'game_cvars.rs').read_text()
    existing = set(re.findall(r'pub (\w+):', gw)) | set(re.findall(r'pub (\w+): vmCvar_t', gc))
    MASTER = {'level', 'g_entities', 'g_clients'}
    # game-owned mutable globals
    cands = []
    for n, cite in sorted(g_cite.items()):
        if n in existing or n in MASTER or n not in g_written:
            continue
        cf = Path(cite.rsplit(':', 1)[0]).name
        if L.tier(cf) != 'game':
            continue
        cands.append((n, cite, cf))
    # group by owning file
    by_file = {}
    for n, cite, cf in cands:
        by_file.setdefault(cf, []).append((n, cite))
    o = []
    o.append("//! `GameGlobals` — the remaining game-tier mutable file-scope globals")
    o.append("//! and file-statics as one owned GameWorld sub-struct (fork ruling 1:")
    o.append("//! file-scope mutable globals become GameWorld fields, grouped by owning")
    o.append("//! `.c` file). Pass-2 porters read/write these through `ctx.world`; they")
    o.append("//! never add a field. Scalar decls carry their Rust type; non-scalar")
    o.append("//! decls (pointers/structs/arrays) are `()` placeholders with a")
    o.append("//! `//TODO: Port <type>` marker — the porter fills the real type when")
    o.append("//! porting that body (bg/qshared-owned globals and const tables are")
    o.append("//! intentionally excluded — not GameWorld state).")
    o.append("#![allow(non_snake_case, non_camel_case_types, unused)]")
    o.append("")
    o.append("use crate::prelude::*;")
    o.append("")
    o.append("/// Raven game-tier mutable file-scope globals (fork ruling 1).")
    o.append("#[derive(Default)]")
    o.append("pub struct GameGlobals {")
    n_scalar = n_todo = 0
    for cf in sorted(by_file):
        o.append(f"    // --- `{cf}` file-scope globals ---")
        for n, cite in by_file[cf]:
            rt, init, todo = resolve(n, cite)
            if todo:
                n_todo += 1
                o.append(f"    //TODO: Port {todo}")
                o.append(f"    // Source: {cite}")
                o.append(f"    pub {fld(n)}: (),")
            else:
                n_scalar += 1
                o.append(f"    /// `{n}`. Source: `{cite}`")
                o.append(f"    pub {fld(n)}: {rt},")
    o.append("}")
    (L.GAME_SRC / 'game_globals.rs').write_text("\n".join(o) + "\n")
    print(f"[genglobals] {len(cands)} fields ({n_scalar} scalar, {n_todo} placeholder) -> game_globals.rs")
    return len(cands), n_scalar, n_todo

if __name__ == '__main__':
    main()
