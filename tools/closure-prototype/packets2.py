#!/usr/bin/env python3
"""PROTOTYPE — pass-2 parked-only port packets. One work order per oracle game
.c file containing ONLY its still-parked fns: the forks-8-11 rulings digest, the
parked fns' cited oracle source, their POST-RETROFIT Rust signatures, the
post-retrofit signatures of everything they call, the GameWorld/GameCvars/
GameGlobals field names they touch, their trap surface, and the fork-9 vec3
out-param target shapes. Files with >60 parked fns shard by contiguous fn range.
Emits out/pass2/packets/ + out/pass2/manifest.json."""
import re, json
from pathlib import Path
import pass2lib as L
import closure as C

REPO = L.REPO
GAME = C.ORACLE / "codemp" / "game"
RULINGS = REPO / "docs" / "handoffs" / "jampgame-fork-discovery.md"
OUT = L.HERE / "out" / "pass2"
SHARD_MAX = 60

def extract_rulings():
    t = RULINGS.read_text()
    return t[t.index("## Fork classes"):].rstrip()

def numbered_slice(path, a, b):
    lines = path.read_text(errors="replace").splitlines()
    a = max(1, a); b = min(len(lines), b)
    return "\n".join(f"{n:>5} | {lines[n-1]}" for n in range(a, b + 1))

VEC = {'vec3_t', 'vec5_t', 'vec4_t', 'vec2_t'}

def vec3_notes(fn):
    outs = []
    for p in fn['params']:
        t = p['type']
        base = t.replace('const', '').strip()
        if base in VEC:
            ro = 'const' in t
            outs.append((p['name'], t, ro))
    if not outs:
        return None
    lines = []
    for n, t, ro in outs:
        if ro:
            lines.append(f"  - `{n}: {t}` — read-only input, keep by-value `[f32;3]`.")
        else:
            lines.append(f"  - `{n}: {t}` — **fork-9 out-param**: `&mut [f32;3]` if written "
                         f"through; `Option<&mut [f32;3]>` if any oracle caller passes NULL "
                         f"(AngleVectors idiom); mutate+scalar-return (VectorNormalize) → "
                         f"`&mut` + scalar. Keep by-value only if never written.")
    return "\n".join(lines)

def main():
    m = L.load_manifest(); F = m['functions']
    need, _ = L.compute_needs_ctx(F)
    rulings = extract_rulings()
    # post-retrofit worktree signatures: name -> (rs_file, params, ret, parked)
    recs = L.scan_worktree()
    wt = {}
    for r in recs:
        # prefer the record whose file matches (mp game). first wins; parked flag kept
        wt.setdefault(r['name'], r)
    # manifest fn by name+file and by name
    byfile = {}
    for f in F:
        byfile.setdefault(f['file'], []).append(f)
    for v in byfile.values():
        v.sort(key=lambda f: f['line'])
    byname = {}
    for f in F:
        byname.setdefault(f['name'], f)
    # cvar + globals field membership
    gc = (L.GAME_SRC / 'game_cvars.rs').read_text()
    cvar_fields = set(re.findall(r'pub (\w+): vmCvar_t', gc))
    gg = (L.GAME_SRC / 'game_globals.rs').read_text()
    glob_fields = set(re.findall(r'pub (?:r#)?(\w+):', gg))
    MASTER = {'level', 'g_entities', 'g_clients'}

    def wt_sig(name):
        r = wt.get(name)
        if r is None:
            return None
        params = re.sub(r'\s+', ' ', r['params']).strip()
        ret = r['ret'].strip()
        tail = f" {ret}" if ret and ret != '{' else ''
        return f"pub fn {name}({params}){(' ' + ret) if ret else ''}".rstrip()

    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / 'packets').mkdir(exist_ok=True)
    manifest_out = []

    for cfile in sorted(byfile):
        if L.tier(cfile) not in ('game', 'bg', 'qshared'):
            continue
        fns = byfile[cfile]
        # parked subset = manifest fns whose worktree rec is parked
        parked = [f for f in fns if (wt.get(f['name']) and wt[f['name']]['parked'])]
        if not parked:
            continue
        loc_parked = sum(f['loc'] for f in parked)
        # shard by contiguous fn ranges
        n_shards = (len(parked) + SHARD_MAX - 1) // SHARD_MAX
        for si in range(n_shards):
            chunk = parked[si * SHARD_MAX:(si + 1) * SHARD_MAX]
            shard = (si + 1) if n_shards > 1 else None
            o = []
            base = cfile[:-2]
            title = f"{cfile}" + (f" — shard {shard}/{n_shards}" if shard else "")
            o.append(f"# PARKED PORT PACKET: `{title}` — jampgame pass 2")
            o.append("")
            o.append(f"Fill ONLY the {len(chunk)} parked fns below (bodies are `todo!()`). "
                     "Their POST-RETROFIT signatures already carry `ctx: GameContext<'_>` "
                     "(fork 8) where needed — do NOT change the signature; write the body. "
                     "Every symbol they call is resolved below with its CURRENT worktree "
                     "signature. Reach state through `ctx.world` (STATE-D6 leaf reborrows) "
                     "and traps through `trap::X(ctx.engine, …)`.")
            o.append("")
            o.append(f"- parked fns in this shard: **{len(chunk)}**, LOC (oracle extents): "
                     f"**{sum(f['loc'] for f in chunk)}**")
            o.append("")
            o.append("## RULINGS DIGEST (verbatim — forks 1-11 + riders)")
            o.append("")
            o.append(rulings)
            o.append("")
            # ---- parked fns: source + post-retrofit signature + vec3
            o.append(f"## PARKED FUNCTIONS ({len(chunk)}) — oracle source + target signature")
            o.append("")
            traps = set(); glob_used = {}; callees = set()
            for f in chunk:
                o.append(f"### `{f['name']}` — {cfile}:{f['line']}-{f['end_line']} "
                         f"({f['loc']} LOC, wave {f['wave']})")
                sig = wt_sig(f['name'])
                if sig:
                    o.append("Post-retrofit Rust signature (fill this body):")
                    o.append("```rust")
                    o.append(sig + " { /* todo!() — port here */ }")
                    o.append("```")
                vn = vec3_notes(f)
                if vn:
                    o.append("fork-9 vec3 params:")
                    o.append(vn)
                o.append("Oracle source:")
                o.append("```c")
                o.append(numbered_slice(GAME / cfile, f['line'], f['end_line']))
                o.append("```")
                o.append("")
                for c in f['callees']['syscall']:
                    traps.add(c['name'])
                for c in f['callees']['in-module'] + f['callees']['bg']:
                    callees.add(c['name'])
                for g in f['globals_read'] + f['globals_write']:
                    glob_used.setdefault(g['name'], g['cite'])
            # ---- resolved call surface (post-retrofit signatures)
            o.append("## CALL SURFACE — post-retrofit signatures (do not explore)")
            o.append("")
            o.append("```rust")
            for name in sorted(callees):
                if name == f['name']:
                    continue
                sig = wt_sig(name)
                if sig:
                    r = wt.get(name)
                    tag = "PARKED" if r['parked'] else "ported"
                    o.append(f"// {tag}: {r['path'].name}")
                    o.append(sig + ";")
                else:
                    o.append(f"//TODO: Port {name}  // Source: oracle/codemp/game/ (unresolved)")
            o.append("```")
            o.append("")
            # ---- state fields
            o.append("## STATE FIELDS TOUCHED (reach via `ctx.world`)")
            o.append("")
            cv = sorted(n for n in glob_used if n in cvar_fields)
            gl = sorted(n for n in glob_used if n in glob_fields)
            ms = sorted(n for n in glob_used if n in MASTER)
            other = sorted(n for n in glob_used if n not in cvar_fields and n not in glob_fields and n not in MASTER)
            o.append(f"- `ctx.world.level` / `.entities` / `.clients`: {', '.join('`'+n+'`' for n in ms) or '_none_'}")
            o.append(f"- `ctx.world.cvars.*` (GameCvars): {', '.join('`'+n+'`' for n in cv) or '_none_'}")
            o.append(f"- `ctx.world.globals.*` (GameGlobals): {', '.join('`'+n+'`' for n in gl) or '_none_'}")
            if other:
                o.append(f"- other file-scope globals (bg-owned / const table — see rulings 5/8a/11): "
                         + ', '.join('`'+n+'`' for n in other))
            o.append("")
            # ---- traps
            o.append(f"## TRAP SURFACE ({len(traps)}) — call `trap::<Name>(ctx.engine, args)`")
            o.append("")
            for t in sorted(traps):
                o.append(f"- `{t}` → `trap::{t[len('trap_'):]}`")
            o.append("")
            fname = f"{base}" + (f".shard{shard}" if shard else "") + ".md"
            (OUT / 'packets' / fname).write_text("\n".join(o))
            manifest_out.append({
                "file": cfile, "packet": f"packets/{fname}",
                "fns_parked": len(chunk), "loc_parked": sum(f['loc'] for f in chunk),
                **({"shard": shard, "shards_total": n_shards} if shard else {}),
            })
    (OUT / 'manifest.json').write_text(json.dumps(manifest_out, indent=1))
    n_files = len({e['file'] for e in manifest_out})
    n_shards = sum(1 for e in manifest_out if 'shard' in e)
    print(f"[packets2] {len(manifest_out)} packets across {n_files} files "
          f"({n_shards} from sharded files) -> out/pass2/packets/")
    print(f"[packets2] wrote out/pass2/manifest.json")

if __name__ == '__main__':
    main()
