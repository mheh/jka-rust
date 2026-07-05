#!/usr/bin/env python3
"""PROTOTYPE — pass-2 retrofit: inject `ctx: GameContext<'_>` into game-tier
needs-ctx fns in the worktree, thread `ctx` through their call sites, ctx-ify
the ent_fn_enums dispatch. Idempotent; verify with `cargo check`.

Tier discipline (fork 8a/3/11): bg/qshared fns get NO GameContext. Game fns
that are called from a bg/qshared body (the cross-tier boundary set) are ALSO
held ctx-free so the boundary stays clean — their state channel is a pass-2
design (bg dispatch / PmoveContext / trap re-tier)."""
import re, json
from pathlib import Path
import pass2lib as L

m = L.load_manifest(); F = m['functions']
need, seed = L.compute_needs_ctx(F)
byusr = {f['usr']: f for f in F}
byname = {}
for f in F:
    byname.setdefault(f['name'], f)  # first def wins for name->fn lookup

# worktree ported/parked status by name (a name is "ported" if any def has a
# real body). Parked fns have empty (todo!()) bodies -> excluding is free.
_recs = L.scan_worktree()
_ported = set()
for r in _recs:
    if not r['parked']:
        _ported.add(r['name'])

# ---- boundary seed: (a) game needs-ctx fns called from a bg/qshared body
#      (fork 8a), (b) game fns stored as RAW Rust fn pointers (fork 7 vehicle
#      vtable) — both must stay ctx-free.
seed_excl = set()
for f in F:
    if L.tier(f['file']) in ('bg', 'qshared'):
        for u in f['callee_usrs']:
            g = byusr.get(u)
            if g and L.tier(g['file']) == 'game' and g['usr'] in need:
                seed_excl.add(g['name'])
import re as _re, subprocess as _sp
_gs = str(L.GAME_SRC)
_stored = _sp.run(['grep', '-rhoE', r'= Some\([A-Za-z_][A-Za-z0-9_]*\)', _gs],
                  capture_output=True, text=True).stdout
for mm in _re.finditer(r'= Some\(([A-Za-z_]\w*)\)', _stored):
    seed_excl.add(mm.group(1))

# ---- downward closure: if an EXCLUDED fn is PORTED, every game needs-ctx fn it
#      calls must also be excluded (its live body would call the ctx-form). Uses
#      manifest callees (oracle superset — safe over-exclusion). Parked fns have
#      no live calls, so they terminate the closure.
def game_ctx_callees(fn):
    out = set()
    for u in fn['callee_usrs']:
        g = byusr.get(u)
        if g and L.tier(g['file']) == 'game' and g['usr'] in need:
            out.add(g['name'])
    return out

EXCLUDE = set(seed_excl)
work = list(seed_excl)
while work:
    nm = work.pop()
    if nm not in _ported:
        continue
    fn = byname.get(nm)
    if not fn:
        continue
    for c in game_ctx_callees(fn):
        if c not in EXCLUDE:
            EXCLUDE.add(c); work.append(c)

inject_names = {f['name'] for f in F
                if f['usr'] in need and L.tier(f['file']) == 'game'} - EXCLUDE
CTX_PARAM = "ctx: GameContext<'_>"
CALL_RE = re.compile(r'(?<![\w.])([A-Za-z_]\w*)\s*\(')

def process(path):
    text = path.read_text()
    fns, _ = L.scan_rs_file(path)
    edits = []
    n_inj = n_cs = 0
    def is_ctx_fn(r):
        return (r['name'] in inject_names) or \
               (r['name'].startswith('dispatch_') and path.name == 'ent_fn_enums.rs')
    # 1) signature injections
    for r in fns:
        if not is_ctx_fn(r) or r['has_ctx']:
            continue
        popen = r['popen']
        if r['params'].strip() == '':
            edits.append((popen, popen + 1, f"({CTX_PARAM}"))
        else:
            edits.append((popen, popen + 1, f"(\n    {CTX_PARAM},"))
        n_inj += 1
    # 2) call-site threading inside ctx-bearing bodies
    for r in fns:
        if not (r['has_ctx'] or is_ctx_fn(r)):
            continue
        b0, b1 = r['bopen'] + 1, r['bend']
        body = text[b0:b1]
        for mm in CALL_RE.finditer(body):
            if mm.group(1) not in inject_names:
                continue
            paren = b0 + mm.end() - 1
            after = text[paren + 1: paren + 48]
            if re.match(r'\s*ctx\b', after):        # already threaded
                continue
            if re.match(r'\s*\)', after):           # empty arg list
                edits.append((paren + 1, paren + 1, 'ctx'))
            else:
                edits.append((paren + 1, paren + 1, 'ctx, '))
            n_cs += 1
    edits.sort(key=lambda e: e[0], reverse=True)
    for s, e, rep in edits:
        text = text[:s] + rep + text[e:]
    return text, n_inj, n_cs

def main():
    recs = L.scan_worktree()
    by_path = {}
    for r in recs:
        by_path.setdefault(r['path'], []).append(r)
    tot_inj = tot_cs = touched = 0
    for path, fns in sorted(by_path.items()):
        if not ((set(r['name'] for r in fns) & inject_names)
                or path.name == 'ent_fn_enums.rs'
                or any(r['has_ctx'] for r in fns)):
            continue
        new, ni, nc = process(path)
        if new != path.read_text():
            path.write_text(new); touched += 1
        tot_inj += ni; tot_cs += nc
    print(f"[retrofit] exclude(cross-tier)={len(EXCLUDE)} inject_names={len(inject_names)}")
    print(f"[retrofit] files touched {touched}, injected {tot_inj}, callsites {tot_cs}")

if __name__ == '__main__':
    main()
