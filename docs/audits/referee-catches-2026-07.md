# Referee catches & C-semantics findings — July 2026

The divergences the differential machinery (golden fixtures → mock referee →
lockstep referee → live sessions) caught, and the C-language semantics each
one taught. Kept as a checklist of *classes* to watch for; F1/F2 sweeps in
`docs/GOAL.md` generalize two of them.

## C-semantics classes (module-side, transcription-era)

| Commit | Site | The C semantic that was lost |
|---|---|---|
| `6dd4345b` | `CheckSaberDamage` saberEnd | `vec3_t` param decays to `float*` — callee writes reach the caller's array. By-value port dropped the write-back. |
| `69e1075a` | `BG_G2PlayerAngles` lastHeadAngles | Same decay class, worse: the write-back is cross-frame state (head-lerp memory persisted into `client->lastHeadAngles`). Killed the frame-1806 family. |
| `f51f89e9` | `BG_G2ClientNeckAngles` | Unsuffixed double literals: C computes `x * 0.4` in f64 and narrows on store; port flattened to all-f32. One-ULP cranium divergence. **Class F1.** |
| `435f7d57` | `G_BounceMissile` | Same class via macro: `VectorScale(v, 0.65, o)` inlines a double multiply per component; also `normal[2] > 0.7` promotes to a double compare. |
| `956101f7` | `g_combat` nullable `point` | C's nullable `vec3_t*` guards (`if (!point)`) dropped by a batch ruling; callers encode NULL as the zero vector. Lost fallbacks (`pos1 = ps.origin` on point-less deaths) diverged the death-anim pick. **Class F2.** |
| `bffc4ccb` | `FireWeapon` → `CalcMuzzlePoint` | C reads live file-scope `forward/vright/up` at the call; port snapshotted them into locals at fn entry — stale aim vectors (one Bryar shot fired with forward = 0). |
| `7936d037` | 14 SnapVector sites | The native module compiles the SnapVector *macro*; only QVM builds trap. Trap-ported sites diverged the syscall stream. |
| `7936d037` | `Rng::rand` | `bg_lib.c` is QVM-only (`ExcludedFromBuild` + `#ifdef Q3_VM`): retail native links the **MSVC CRT LCG** (`holdrand*214013+2531011`), not bg_lib's 69069. Any bg_lib-variant libc transcription is suspect. |
| `7936d037` | `chatTime` | C `strlen()*45 + int` runs in `size_t`: a negative int wraps to ~2^64 ("never chat"); the i32 port went negative ("chat now"). Audit mixed strlen/int arithmetic. |
| `c3aa37d8` | five hoist sites | Hoisting subexpressions out of conditionals changed C's short-circuit **evaluation order** (side-effecting operands). |
| `0b41dc4e` | `holdrand` | `c_ulong` is platform-width, not `u32` (t2_wedge NPC-type pick). Predates the CRT-rand finding above. |
| `dfcfa240` | MD4 | Raven's `UINT4` truncates to 32 bits; wider arithmetic broke sv_pure checksums. |
| `855b73ef` / `4138f7d0` | NPC class ids / `G_LogWeaponPowerup` | Raven UB sites (§19): out-of-range enum reads and a mis-sized stats array — pick one defined behavior, note it at the site. |
| *(fix in flight, task #1)* | `Rng::crandom` → `Drop_Item` toss velocity | Translation bug, **class F1**: the `crandom()` macro (`q_shared.h:1592`, no `#ifdef` variants) is `2.0 * (random() - 0.5)` — double arithmetic, double result — and `velocity[2] += 200 + crandom() * 50` stays double until the f32 store. The port translated `crandom` as an all-f32 fn. Brute force over all 32,768 `rand()` draws: 7.1% narrow to a different f32 (1 ULP) — rare enough that 12k-frame soaks stayed clean; the 27k-frame human session hit one (ent115 `weapon_thermal` toss, frame 14282). Diagnosed from the tape alone, no live repro. |

## Width/ABI catches (engine + harness plumbing)

| Commit | Finding |
|---|---|
| `059157c4` | Oracle `vmMain(int, int…)` truncates pointer-carrying words on LP64 — widened to intptr_t in the harness build; engine clientNum width fixed alongside. |
| `2603f697` | VM words carried at `AbiWord` width; `SV_Trace` NULL mins/maxs guard restored at the syscall arms. |
| `fa2da4f3` | `ConvertedEntity` pointer fields must be word-width. |
| `cefd12b4` | Mock referee read 32-bit trap buffer sizes at 64-bit width. |
| `eaac8c75` | "Divergences" that were harness FP semantics: oracle build patched to retail-win32 behavior (SnapVector → `rint`, float libm calls forced to the double family MSVC's C compile used). The spawn-pitch and 1-ULP angle findings dissolved. |
| `a86bc811` (supersedes `926cccb5`) | playerState netfield tables: the **retail wire is 137/140/69 rows** — the oracle source drop's 152/152/80 tables never shipped. Never regenerate from oracle msg.cpp. |
| `d1130ae5` | `CM_ModelBounds` out-params were dropped — brush entities had no bounds. |
| `45b800ed` | `com_printf` cleared the rcon redirect buffer after every no-op flush — every rcon reply engine-wide was empty. |

## Live-session catches (not referee-visible until humans played)

| Commit | Finding |
|---|---|
| `4ac85cf8` | Three stacked ghoul2 bolt defects, incl. a **silent, marker-less always-fail `G2_Add_Bolt` stub** — sabers whiffed. Lesson: sweep for unmarked always-fail stubs. |
| `2158aba6` | Follower wiped human packet-loop events buffered between server ticks (SV_Frame runs ~3×/game frame) — ClientBegin and ~half the usercmds never reached the mirror. Bot-only runs can't expose it. |
| `d6902d8b` | Ping is an *input*: `SV_CalcPings` writes network-computed ping into digested `ps.ping` → taped as `P` records. Replicas also phantom-timed-out at `sv_timeout` (no packets refresh their clock) — injected events now count as packets. |

## Open

- `ent115.pos.trDelta+8` (live frame 14282): **root cause found 2026-07-14**
  — the `Rng::crandom` translation bug above; fix + tape-replay verification
  in flight. Task #1.
- Replica-connect syscall-count blips (equal digests). Task #2.
