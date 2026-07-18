#!/usr/bin/env python3
"""qmath-census: classify oracle q_math functions as SP/MP-identical or per-mode.

Drives native_math membership (2026-07-17 centralization ruling): a function
whose MP (`oracle/codemp/game/q_math.c`) and SP (`oracle/code/game/q_math.cpp`)
bodies are behaviorally identical is single-sourced in `crates/native/math`;
a behaviorally divergent one is defined per-mode and re-exported. Run from the
repo root; review any classification change before moving code.

Known-equivalent surface noise (kept in the IDENTICAL bucket after manual
verification, see the audit trail in git history 2026-07-17):
  - debug-only asserts (Q_rsqrt, ProjectPointOnPlane)
  - `-32768` vs `(short)0x8000` (ClampShort), `360` vs `360.0` literals
  - `fabs` vs `Q_fabs` (identical for all float inputs)
  - declaration-position shuffles (G_Find*/G_PointDist*)
  - lazy vs eager sin/cos of unused axes (AngleVectors)
Genuinely divergent at last audit:
  - PerpendicularVector (axis scan order: MP x->z, SP z->x with z bias)
  - ClearBounds (MP 99999 sentinel, SP WORLD_SIZE = 128k)
"""
import re, pathlib, sys

MP = "oracle/codemp/game/q_math.c"
SP = "oracle/code/game/q_math.cpp"

def fns(path):
    s = pathlib.Path(path).read_text(errors="replace")
    out = {}
    for m in re.finditer(r"^[A-Za-z_][A-Za-z_0-9 \t\*]*?[ \t\*]([A-Za-z_][A-Za-z_0-9]*)\s*\([^;{]*\)\s*\{", s, re.M):
        i = s.index("{", m.start()); depth = 0; j = i
        while True:
            if s[j] == "{": depth += 1
            elif s[j] == "}":
                depth -= 1
                if depth == 0: break
            j += 1
        out.setdefault(m.group(1), []).append(re.sub(r"\s+", " ", s[m.start():j+1]).strip())
    return out

mp, sp = fns(MP), fns(SP)
both = sorted(set(mp) & set(sp))
ident = [n for n in both if mp[n] == sp[n]]
diff = [n for n in both if mp[n] != sp[n]]
print(f"MP fns: {len(mp)}  SP fns: {len(sp)}  shared names: {len(both)}")
print(f"textually identical: {len(ident)}")
print(f"textually differing (audit each): {diff}")
print(f"MP-only: {sorted(set(mp) - set(sp))}")
print(f"SP-only: {sorted(set(sp) - set(mp))}")
