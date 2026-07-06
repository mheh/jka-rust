# jampgame-oracle — differential harness for the `q_math` + `bg_lib` ports

Compiles the **unmodified** Raven `codemp/game/q_math.c` and `codemp/game/bg_lib.c`
into standalone dumper binaries, runs them over `fixtures/`, and stores the
canonical bit-exact dumps under `golden/`. The Rust ports
(`crates/mp/game/src/q_math.rs`, `bg_lib.rs`, `bg_channel/rng.rs`) must reproduce
the goldens byte-for-byte via `crates/mp/game/tests/jampgame_parity.rs`.

Every float is dumped as its IEEE-754 **bit pattern** (`%08x` for `f32`,
`%016llx` for `f64`) — no textual float rounding is ever involved on either side;
the Rust test uses `f32::to_bits()` / `f64::to_bits()`.

## Usage

```sh
sh run.sh            # build dumpers, diff current output against golden/
sh run.sh --regen    # rebuild golden/ (after changing fixtures/dumpers)
cargo test -p mp_game --test jampgame_parity
```

`run.sh` copies the oracle `.c` files and their real header chain (`q_shared.h`
→ `teams.h`, `bg_lib.h`, `surfaceflags.h`, `../qcommon/{disablewarnings,tags}.h`)
into `build/` so their relative `#include`s resolve; `oracle/` is never edited.
The goldens are committed, so `cargo test` needs no C toolchain — `run.sh` is
only needed to regenerate or spot-check.

## Slices

Slices added 2026-07-06 are self-contained: each has its own `run_<slice>.sh`
(own `build-<slice>/` dir, gitignored), `main_<slice>.c`, `fixtures/<slice>/`,
`golden/<slice>.txt`, and `crates/mp/game/tests/<slice>_parity.rs` — the build
model and rules below apply unchanged. Current set:

- **qshared** (`run_qshared.sh`) — `q_shared.c` tokenizer (`COM_Parse`/
  `COM_ParseExt`/`COM_Compress`/`SkipBracedSection`), path/string helpers
  (`Q_str*`, `Q_CleanStr`, `va`), and the `Info_*` family incl. `_Big`
  variants. Reconciliation fixed 4 port bugs (see the slice commit).
- **bgmisc** (`run_bgmisc.sh`) — `BG_EvaluateTrajectory`/`Delta` over every
  `trType_t`, the full `bg_itemlist`/`weaponData`/`ammoData` tables,
  `BG_FindItem*`, `BG_CanItemBeGrabbed` (every branch), and
  `BG_PlayerStateToEntityState(ExtraPolate)`. Fixed the port's f32-vs-f64
  trajectory evaluation. `snap=1` excluded (§19 — the `-D__linux__` macro
  truncates where retail x87 rounds; platform-ifdef not arbitrable here).
- **pmove_saber** (`run_pmove_saber.sh`) — the pmove single-step model
  (spec/world/RNG tripwire unchanged) re-based on `WP_SABER`: stance/gait
  anims, standing/running/strafing attack arcs (`saberMove` chains), jump.
  Dump line appends `sm sb shl sen sal sac`. Zeroed `g_entities` makes
  `BG_MySaber` NULL on both sides — custom-saber override paths stay
  deliberately out of scope.

Original slices:

- **qmath** — RNG streams (`Rand_Init`/`flrand`/`Q_flrand`/`irand`/`Q_irand`
  over 4 seeds, ~800 interleaved draws each; `Q_rand`/`Q_random`/`Q_crandom`),
  `ClampChar`/`ClampShort`/`Q_log2`/`Q_rsqrt`/`Q_fabs`/`powf`/`ByteToDir`/
  `ColorBytes*`/`NormalizeColor`, `NormalToLatLong`, `SetPlaneSignbits`/
  `BoxOnPlaneSide`, and the vector/angle/matrix headline functions driven over a
  30-entry vec3 fixture table (zeros, negatives, denormals, 90/180/270°,
  non-normalized, large).
- **bglib** — `srand`/`rand` streams (5 seeds), `atoi`/`atof`/`_atof` over a
  string fixture table, `qsort` over an int array and a struct array (deterministic
  comparator), and `memmove` overlap cases.
- **saberload** — `bg_saberLoad.c`'s `WP_SaberParseParms` `.sab` parser. Drives
  the real load path (`WP_SaberLoadParms` fills the `SaberParms` buffer from
  `fixtures/sabers/*.sab`, then `WP_SaberParseParms` per saber name) and dumps
  every `saberInfo_t` field in declaration order (floats as IEEE-754 bit-hex,
  strings quoted, `blade[]`/sound arrays indexed) plus the per-saber
  sound/skin registration logs. Six saber names exercise: a realistic single
  (`Kyle`, doubling as the `DEFAULT_SABER` fallback target), a staff (two blade
  styles, secondary `*2` fields, per-index `saberColor2`/`saberLength2`,
  swing/hit2 sound lists, `SFL_TWO_HANDED`), an edge saber (below-cap
  length/radius clamped to 4.0/0.25, unknown tokens skipped, `SABER_ARC`,
  negative/large numerics, force-restrict + style bitfields, `animTable`
  lookups, a `customSkin` registration, a value-less trailing keyword), a
  truncated block (`broken_saber` → unexpected-EOF → `qfalse`), a not-found
  name (poisoned by the unclosed block → `qfalse`), and the empty name
  (immediate `DEFAULT_SABER`).

## Build model (no `-DQ3_VM`)

The dumpers compile for the **native** target, exactly like the shipping `jamp`
game module — `Q3_VM` is **not** defined, so `q_shared.h` pulls the system libc
and `M_PI` is math.h's **double**. Compiler flags of note:

- `-D__linux__` — selects `q_shared.h`'s clean C branches (the default Win32
  path has MSVC inline `__asm` in `SnapVector` that clang can't compile). It does
  not affect the tested math.
- `-ffp-contract=off` — disables FMA contraction so `a*b+c` matches Rust's
  default (non-contracted) evaluation bit-for-bit.
- `-D_FORTIFY_SOURCE=0` — macOS wraps `memmove` in a fortify macro that collides
  with `bg_lib.c`'s own `memmove` definition.
- `shim.h` (force-included) does `#include <math.h>` then `#define powf raven_powf`
  so Raven's 2-arg `powf(float,int)` doesn't clash with libm's `powf(float,float)`.

## Three verbatim extractions (unreachable / UB on a native LP64 build)

`run.sh` extracts three functions **verbatim** from the copied oracle sources
into their own `build/` files, because a native, no-`Q3_VM` compile on this LP64
host cannot faithfully reach them otherwise. The extractions edit nothing in
`oracle/`; they only rename symbols and normalize an integer width.

- **`raven_atoi.c`** — Raven's `atoi` (`bg_lib.c:915-958`) is guarded by
  `#if defined(Q3_VM)`. Without `Q3_VM` the linker would bind libc's `atoi`,
  which does **not** do Raven's signed-char `> 0x7f` whitespace skip. Extracted
  and renamed `raven_atoi`.
- **`raven_rng.c`** — Raven's `holdrand` LCG (`q_math.c:1432-1474`). `holdrand`
  is `unsigned long`; on the shipping 32-bit i686 target that is 32-bit, which is
  exactly what the port models (`Rng`'s `u32` + a static-assert). On this LP64
  host `unsigned long` is 64-bit and the `>> 17` masking would diverge, so the
  extraction normalizes `unsigned long` → `unsigned int` and renames the
  functions `r_*`.
- **`raven_rng.c` (appended) `r_Q_rsqrt`** — `Q_rsqrt` (`q_math.c:616-636`) reads
  a `float` through `*(long*)&y`; on LP64 that reads 4 bytes past the float (UB)
  and `>>1` diverges from the 32-bit target. Normalized `long` → `int` (the
  port's i32 model) with the `__linux__` `isnan` assert dropped.

## The saberload slice: build model, stubs, and entry sequence

Unlike the qmath/bglib slices (which compile for the plain native target), the
saberload dumper is compiled with **`-DQAGAME`** — jampgame *is* Raven's QAGAME
build, so the `#elif defined CGAME` branches are dead (effect tokens
`SkipRestOfLine` instead of registering; `BG_SoundIndex` routes to
`G_SoundIndex`). It links the **unmodified** `bg_saberLoad.c` + `q_shared.c`
(the `COM_*` parser + `GetIDForString`/string tables) plus `animtable_def.c`
(compiles `cgame/animtable.h` standalone to define the `animTable` symbol,
byte-identical to the port's generated `anim_table.rs`). `run.sh` copies the
full `game/` + `qcommon/` header closure, the two `namespace_*.h` shims, and
`animtable.h` into `build/` so the copied sources' relative includes resolve;
`oracle/` is never touched.

**Entry sequence** (mirrors how the game loads sabers, read out of
`WP_SaberLoadParms` → `WP_SaberParseParms`): the dumper first calls
`WP_SaberLoadParms()`, which `trap_FS_GetFileList("ext_data/sabers", ".sab", …)`,
then for each file `trap_FS_FOpenFile` / `trap_FS_Read` / `COM_Compress` /
`Q_strcat`s the contents (plus a trailing `\n`) into the `SaberParms` buffer.
It then calls `WP_SaberParseParms(name, &saber)` per saber name over a
zeroed `saberInfo_t`. The Rust parity test drives the identical two-step over
the ported `crate::bg_saberLoad` (`BgState` owns `SaberParms`; a fixtures-backed
`TestTraps: BgTraps` serves the FS calls).

**Stub definitions provided in `main_saberload.c`** (the TU extern-declares
these; the linker demands them):

| symbol | faithfulness |
| --- | --- |
| `trap_FS_GetFileList` / `FOpenFile` / `Read` / `FCloseFile` | backed by `fixtures/sabers/`; vpaths (`ext_data/sabers[/name]`) mapped by stripping `ext_data/` and prefixing the fixture dir. Listing is **sorted** (byte-lexicographic) since `readdir` is unordered — the Rust `TestTraps` sorts identically, so `SaberParms` is byte-identical on both sides. `FS_Write` is a no-op (never on the load path). |
| `trap_R_RegisterSkin` | name-logging counter (per-saber, from 1). Skins genuinely cross the observable `BgTraps` seam, so both sides mint the same deterministic handle — the `customSkin` field carries it and the name is logged (`regskin`). |
| `G_SoundIndex` (behind `BG_SoundIndex`) | name-logging observer that returns **0** — see normalization below. |
| `FPTable` | the force-power name/id table, written with Raven's `ENUM2STRING` macro (oracle `bg_saga.c:100-121`); matches the port's `bg_saga::FPTable`. |
| `animTable` | supplied by `animtable_def.c` (the real `cgame/animtable.h`). |
| `Com_Printf` / `Com_Error` | routed to **stderr** so parser diagnostics never enter the golden (stdout). `Com_Error` additionally `exit(3)`s (mirrors the port's `panic!`); fixtures never trigger it (`numBlades` kept valid, buffer small). |
| `Q_irand` | link-satisfying stub, **never called** — only `TranslateSaberColor`'s `"random"` color reaches it, and fixtures avoid `"random"` (it would need a seed-matched RNG on both sides). |

## Normalizations (documented divergences — porting-rules §19)

- **Sound-index return values are 0 on both sides; only the registration
  *names* are observable.** The port's `G_SoundIndex` is a documented
  placeholder returning 0 (the configstring architecture is unwired), so every
  `saberInfo_t` sound field (`soundOn`, `swingSound[]`, `hitSound[]`, …) is 0.
  The oracle dumper's `G_SoundIndex` stub matches (returns 0). What *is*
  pinned is the sequence of names `BG_SoundIndex` is called with (the `regsound`
  log) — real parser behavior, identical on both sides. The port exposes this
  order through a dormant thread-local observation seam (`saber_snd_tape_*` in
  `bg_saberLoad.rs`); it only *observes* (return value unchanged), so production
  behavior is byte-identical whether or not the tape is installed. Skins, by
  contrast, cross the real `BgTraps` seam and so carry a genuine per-saber
  counter (`skin` field + `regskin` log). This asymmetry — skins observable via
  the wired seam, sounds only name-observable pending `G_SoundIndex` — is itself
  the surfaced divergence.
- **The unclosed `broken_saber` block poisons not-found searches.** In the
  concatenated `SaberParms` buffer the truncated block has no closing `}`, so
  any full traversal (`SkipBracedSection`) runs `p` to `NULL` and the search
  loop exits `qfalse` via `if(!p)` before the `DEFAULT_SABER` fall-through can
  fire. This is Raven's real behavior (pinned by the `broken_saber` and
  `nonexistent_xyz` cases); it is not a divergence, just a consequence noted so
  the golden's `ret 0` values read correctly.

## Normalizations (qmath/bglib)

- **NaN bit patterns are canonicalized to the positive quiet NaN** (`7fc00000`
  f32 / `7ff8000000000000` f64) by `dumpcommon.h`'s `f2b`/`d2b` and the Rust
  tests' `cbits()` mirror. NaN sign/payload is platform-defined — ARM's default
  qNaN is positive, x86 SSE's is negative — so the raw bits diverge across
  hosts even when both sides agree per-platform (first seen as CI-on-Linux vs
  goldens-from-macOS on `ProjectPointOnPlane` degenerate cases). Value
  computation is untouched; only the dump encoding is canonical.
- **`ColorBytes3`** writes only bytes `[0..2]` of an uninitialized `unsigned i`;
  byte `[3]` is indeterminate stack garbage. The dumper masks it (`& 0x00ffffff`)
  so the golden is deterministic and matches the port (which zeroes byte 3).
  `ColorBytes4` sets all four bytes and is compared unmasked.
- **`ColorBytes*` inputs are kept in `[0,1]`.** A `float → byte` cast of an
  out-of-range value (`r*255 < 0` or `> 255`) is C UB and diverges from the
  port's saturating cast; such inputs are kept out of the fixtures.
- **`PlaneFromPoints`** leaves `plane[3]` untouched on the degenerate path (as
  does the port), so the dumper zero-inits the `vec4_t` for a defined value.
- **`ProjectPointOnPlane` / `PerpendicularVector` / `RotatePointAroundVector`**
  divide by `dot(normal,normal)` and assert against a zero divisor (mirrored by
  the port's `debug_assert`). Degenerate normals (zero, or denormals whose
  squared sum underflows to 0) are skipped (`... SKIP`) on both sides.
- **`NormalToLatLong`** is fed only unit-ish normals (`|z| ≤ 1`); `acos` of an
  out-of-range `z` is NaN, and C's `(int)NaN` vs Rust's saturating `as i32`
  diverge.

## Port fixes made during reconciliation

Reconciling the port against the oracle surfaced (and fixed, in the crate) these
real port bugs — the divergence is the product:

- **f32-vs-f64 math.** The port evaluated many functions entirely in `f32`, but
  Raven calls the **double** libm (`sin`/`cos`/`atan2`/`sqrt`/`acos`) and, on the
  native build, `M_PI` is math.h's **double**. Fixed to evaluate the trig/sqrt
  and the `M_PI`/`DEG2RAD`/`RAD2DEG`/`AngleMod` constant chains in `f64`, rounding
  to the `f32` result: `VectorLength`, `VectorNormalize`, `VectorNormalize2`,
  `DistanceHorizontal`, `vectoangles`, `AngleVectors`, `RotatePointAroundVector`,
  `NormalToLatLong`, `AngleMod`, `AngleNormalize360`,
  `G_FindClosestPointOnLineSegment`, `G_PointDistFromLineSegment`.
- **`bg_lib::atoi` / `Q_rsqrt` integer overflow.** C `int` arithmetic wraps;
  Rust `*`/`+`/`-` panic in debug. Switched to `wrapping_*` so huge `atoi`
  inputs and the `Q_rsqrt` magic-constant subtraction match Raven.
- **`bg_lib::_atof` leading-dot.** Raven inits `int c = '0'` and checks the sign
  on `*string` directly; a leading `.` with no digits leaves `c == '0'` so the
  fractional block is skipped and `_atof(".5")` returns 0 advancing 0. The port
  had seeded `c` from the sign char and wrongly consumed the `.`.
- **saberload: none.** The pass-3 port of `WP_SaberParseParms` /
  `WP_SaberLoadParms` reproduced the golden byte-for-byte on first reconciliation
  across all field categories (clamps, per-blade `saberColor`/`saberLength`
  indices, style/force bitfields, `animTable` lookups, flag composition, the
  registration logs, and the `DEFAULT_SABER` / EOF / not-found control paths).
  The only port-side addition was the dormant `saber_snd_tape_*` observation
  seam described above, which changes no behavior.

## Single-threading

The oracle keeps its RNG in file statics, so each family is dumped by a fresh
process. The Rust side mirrors that constraint: one `#[test]` per family, one
`Rng` per family, sub-checks sequential inside — never parallel draws against a
shared generator.

---

# The pmove single-step slice

The frozen contract for this slice lives in [`pmove-spec.md`](pmove-spec.md)
(committed verbatim next to this README so the repo is self-contained). It is
the binding source of truth; this section is the operator's summary. Two
dumpers, two goldens:

- **`main_trace.c` → `golden/pmove_trace.txt`** — proves the axial-brush trace
  stub (`pmworld.h`) *in isolation*, before any pmove logic. If a pmove golden
  ever mismatches, this proves the collision layer is not the suspect.
- **`main_pmove.c` → `golden/pmove.txt`** — drives the UNMODIFIED oracle `Pmove`
  over six on-foot scenarios, dumping every playerState_t / pmove_t field that
  changes during basic movement as IEEE-754 bit-hex.

The Rust parity test (`crates/mp/game/tests/pmove_parity.rs`, Agent B) drives
`mp_game::bg_pmove::Pmove` over the **same** fixture files + the **same**
synthetic `animation.cfg` and must reproduce both goldens byte-for-byte.

## Build model (`-DQAGAME`, `-fgnu89-inline`, RNG rename)

`bg_pmove.c` under `-DQAGAME` (jampgame *is* Raven's QAGAME build) pulls in
`g_local.h` + `ghoul2/G2.h`, so the pmove dumper links a much larger closure
than the saberload slice and needs the ghoul2/cgame/icarus header trees copied
into `build/`. The linked TUs (all UNMODIFIED oracle):

`bg_pmove.c bg_slidemove.c bg_panimate.c bg_saber.c bg_saberLoad.c bg_misc.c
bg_weapons.c q_shared.c q_math.c` + `main_pmove.c`.

Flags of note beyond the shared set (`-D__linux__ -ffp-contract=off -include
shim.h`):

- **`-fgnu89-inline`** — Raven's PM_* helpers are non-static `inline`
  (`qboolean PM_INLINE PM_IsRocketTrooper(void)`, `PM_INLINE == ID_INLINE ==
  inline`). Under C99 semantics clang emits *no* out-of-line symbol at `-O0`, so
  the intra-TU call goes unresolved at link. gnu89 inline semantics (Raven's own
  compilation model) emit the external definition. This changes only symbol
  emission — never the IEEE math.
- **q_math.c is recompiled with its holdrand RNG functions renamed**
  (`-DQ_irand=o_Q_irand -Dirand=o_irand -Dflrand=o_flrand -DQ_flrand=o_Q_flrand
  -DRand_Init=o_Rand_Init`). See "RNG tripwire" below.
- No `animtable_def.c` — `bg_panimate.c` `#include`s `cgame/animtable.h`
  directly, so it already defines `animTable`; linking `animtable_def.c` (as the
  saberload slice does) would duplicate the symbol.

The trace dumper is self-contained (only `pmworld.h` + a fixture parser) and
compiles for the plain native target — no QAGAME, links nothing.

## Stub table (`main_pmove.c`)

The TU provides everything the closure extern-references from unlinked game TUs
or the engine. Every entry that must NOT be reached on the basic on-foot path is
an `abort()`ing stub — a firing stub means a fixture leaked off the path, and
the abort makes it loud + greppable (the Rust `TestCallbacks`/`TestTraps`
`panic!` for the same reason).

| symbol(s) | role |
| --- | --- |
| `g_entities[]`, `level`, `g_gametype`, `bg_fighterAltControl`, `g_vehWeaponInfo` | zeroed data globals the closure reads |
| `FPTable` | force-power name/id table (bg_saga form), matches the port |
| `pmove_t.trace` / `.pointcontents` | the `pmworld.h` axial-brush world (see below) |
| `trap_SnapVector` | `rintf` per component (see snap_vector pin) |
| `trap_FS_FOpenFile/Read/FCloseFile/Write/GetFileList` | fixtures-backed; only the `animation.cfg` load path uses them. Any request is mapped to `<fixdir>/<basename>`; a missing file returns −1 so optional loads (animevents) skip |
| `Q_irand` (+`irand`/`flrand`/`Q_flrand`/`Rand_Init`) | 32-bit holdrand LCG mirror + draw counter — the RNG tripwire |
| `Com_Printf` → stderr; `Com_Error` → `exit(3)` | diagnostics never enter stdout; a triggered `Com_Error` is a fixture bug |
| `Client_CheckImpactBBrush`, `G_CheapWeaponFire`, `G_Damage`, `G_DamageFromKiller`, `G_AddEvent`, `G_PlayEffect(ID)`, `G_CanBeEnemy`, `G_FlyVehicleSurfaceDestruction`, `G_NewString`, `G_SoundIndex`, `NPC_SetAnim`, `Q3_SetParm`, `TryGrapple`, `WP_GetVehicleCamPos`, `FighterIsLanded`, `trap_Trace`, `trap_FX_PlayEffect`, `trap_R_RegisterSkin`, `trap_G2API_*`, `strap_G2API_*` | **`abort()` stubs** — vehicle / entity-impact / ghoul2 / effects surface, all unreachable on the basic on-foot MELEE path |

The two `GameCallbacks` that *are* reachable — the QAGAME anim restart-check —
are served directly by the anim mirror (below), not by a stub.

## Anim mirror rule

`bg_panimate`'s QAGAME `BG_Start{Legs,Torso}Anim` restart-check reads
`g_entities[clientNum].s.legsAnim/torsoAnim` — the value
`BG_PlayerStateToEntityState` writes live at the *end* of each server frame,
i.e. the *previous* frame's `ps` anim. The dumper reproduces this by copying
`ps.legsAnim/torsoAnim` into `g_entities[0].s` **after** every `Pmove` (and
seeding it from the initial `ps` at `start`), so the next step's restart-check
sees the prior value. The Rust `TestCallbacks::entity_legs/torso_anim` returns
the identical mirror.

## Synthetic `animation.cfg`

`Pmove` needs a loaded animation set (`pm->animations`): `PM_SetAnim` →
`BG_SetAnimFinal` **asserts** `firstFrame > 0 || numFrames > 0` on any anim it
plays, and both sides must parse the same frame data or their `legsTimer` /
`torsoTimer` / `bobCycle` diverge. There is no real `animation.cfg` asset in the
repo, so `fixtures/pmove/animation.cfg` is **synthetic** — generated by
`gen_animcfg.py` from the oracle `animTable` (`cgame/animtable.h`). It emits
*every* animTable token so no assert can fire regardless of path; the ~50
movement anims the on-foot MELEE path plays (`BOTH_STAND1`, `BOTH_WALK1`,
`BOTH_RUN1`, `BOTH_JUMP1`, `BOTH_INAIR1`, `BOTH_LAND1`, roll/land/turn/crouch
variants, `TORSO_WEAPONREADY*`) carry hand-tuned, distinct, plausible frame
blocks so the timer goldens are non-degenerate; every other token carries a
uniform filler `1 1 -1 20` purely to satisfy the assert and is never played.
The specific numbers are irrelevant to parity because **both** dumper and Rust
port parse this exact committed file. The grammar is Raven's
`BG_ParseAnimationFile` (`bg_panimate.c:2442-2520`): `<TOKEN> <firstFrame>
<numFrames> <loopFrames> <fps>`, `//` and `/* */` comments honoured by
`COM_Parse`; `frameLerp = ceil(1000/fps)`. Both sides load it via
`BG_ParseAnimationFile("models/players/_humanoid/animation.cfg",
bgHumanoidAnimations, qtrue)` (the dumper redirects the vpath to the fixture
file through the stubbed FS traps).

## snap_vector pin

`trap_SnapVector(v)` is pinned to `v[i] = rintf(v[i])` ↔ Rust
`f32::round_ties_even` (round-to-nearest, ties-to-even). **This may differ from
the real jamp engine's `SnapVector`** (an x87/SSE `cvtss2si`-style round in the
shipping build). It is fine for the differential slice — both sides use `rintf`
— but is flagged here for the eventual live-engine seam revisit.

## The trace stub (`pmworld.h`) — verbatim algorithm

The world is a set of axis-aligned box brushes (`brush x0 y0 z0 x1 y1 z1
surf=<hex>`). The trace is Q3's `CM_ClipBoxToBrush` restricted to those axial
brushes — which reproduces exactly the semantics pmove depends on. **Bit-identity
rules**: (a) every float literal carries the `f` suffix (a bare `0.125` promotes
the subexpression to `double`; the Rust f32 side would diverge); (b) only f32
`+ - * /` and compares — no libm/fabs/sqrt/macros, because axial normals are
exact `(0,±1)` and need no normalization. With `-ffp-contract=off` the result is
IEEE-deterministic on both sides. Pseudocode (per brush; `SURFACE_CLIP_EPSILON =
0.125f`):

```
enterFrac = -1; leaveFrac = 1; startout = getout = 0; clip = none
for each of the 6 outward axial faces (normal exactly one of ±x,±y,±z; dist = world face dist):
    ofs[i]  = normal[i] < 0 ? boxMaxs[i] : boxMins[i]           # Minkowski expand
    dist    = faceDist - dot(ofs, normal)
    d1 = dot(start, normal) - dist                             # signed dist, start
    d2 = dot(end,   normal) - dist                             # signed dist, end
    if d2 > 0: getout   = 1                                     # end not in solid
    if d1 > 0: startout = 1                                     # start not in solid
    if d1 > 0 and (d2 >= 0.125f or d2 >= d1): return           # wholly in front -> no hit
    if d1 <= 0 and d2 <= 0: continue                            # never crosses this plane
    if d1 > d2:                                                 # entering
        f = (d1 - 0.125f) / (d1 - d2); if f < 0: f = 0
        if f > enterFrac: enterFrac = f; clip = this plane
    else:                                                       # leaving
        f = (d1 + 0.125f) / (d1 - d2); if f > 1: f = 1
        if f < leaveFrac: leaveFrac = f
if not startout:                                               # start was inside brush
    startsolid = 1; if not getout: allsolid = 1; fraction = 0
    return
if enterFrac < leaveFrac and enterFrac > -1 and enterFrac < fraction:
    if enterFrac < 0: enterFrac = 0
    fraction = enterFrac; plane = clip; surfaceFlags = brush.surf; contents = CONTENTS_SOLID
```

Driver outputs pmove consumes: `allsolid`, `startsolid`, `fraction`,
`endpos[i] = start[i] + fraction*(end[i]-start[i])`, axial `plane.normal` +
`plane.dist` + `plane.type`/`signbits`, per-brush `surfaceFlags`,
`contents = CONTENTS_SOLID`, and `entityNum = fraction < 1 ? ENTITYNUM_WORLD
(1022) : ENTITYNUM_NONE (1023)`. `pointcontents` = point inside any brush AABB
→ `CONTENTS_SOLID` else 0.

## RNG tripwire

The only mid-pmove RNG draws are `PM_HoverTrace`/jetpack (`Q_irand`), which the
fixtures never reach. To make "no draw" *observable*, `main_pmove.c` defines its
own `Q_irand` (+`irand`/`flrand`/…) mirroring Raven's holdrand LCG **normalized
to 32-bit** (the port's `u32` `Rng` model — Raven's `unsigned long holdrand` is
64-bit on this LP64 host and would diverge) plus a draw counter; `run.sh` renames
q_math.c's own copies to `o_*` to avoid a duplicate symbol. `holdrand` is seeded
to `0x89abcdef` and dumped as `rng=%08x` every step. On the basic path it stays
`0x89abcdef` in all six scenarios — any draw would move it and the diff would
catch it. (Deviation from spec §4, which named the file-static `holdrand`
directly: that static is inaccessible in the unmodified q_math.c, so the tripwire
is this reproducible 32-bit mirror instead — strictly observable on both sides.)

## Fixture grammar (as implemented — Agent B must match)

Both dumpers read line-oriented text; `#` starts a comment. Floats are either a
plain (possibly negative) integer — parsed exactly as `(float)atol` — or an
`0x????????` f32 **bit pattern**; there are no decimal-point tokens (they would
double-round differently on the two sides).

**`fixtures/pmove/trace.txt`** (trace dumper):
```
brush  <x0> <y0> <z0> <x1> <y1> <z1> surf=<hex>
sweep  <sx> <sy> <sz> <ex> <ey> <ez> <mnx> <mny> <mnz> <mxx> <mxy> <mxz>
reset                          # clear the brush set
```

**`fixtures/pmove/<scenario>.txt`** (pmove dumper):
```
brush <x0> <y0> <z0> <x1> <y1> <z1> surf=<hex>     # world geometry
ps    <field> <value...>                            # override a baseline pin
start                                               # freeze anim mirror + emit step 0 (pre-move)
cmd   <dt> <fwd> <right> <up> <buttons> <yaw> <pitch> <roll> [xN [yawinc]]
```
`ps` fields: `origin`/`velocity`/`viewangles` (3 f32), `delta_angles` (3 int),
and scalars `groundEntityNum` `pm_flags` `pm_type` `legsAnim` `torsoAnim`
`weapon` `gravity` `speed` (f32) `basespeed` `fallingToDeath` `clientNum`. A
`cmd` row's `angles` are raw `int16` BAM (no `ANGLE2SHORT` float math in the
parser); `buttons` accepts hex. `xN` repeats the row N times; an optional
`yawinc` adds to the yaw short each repeat (used by strafe-turn). (This is the
spec §5 grammar with `ps <field> <value>` spelled space-separated rather than
`key=value`, and `yawinc` added for the turning scenario.)

The **baseline pins** every scenario starts from (spec §2), before `ps`
overrides: `pm_type=PM_NORMAL`, `weapon=cmd.weapon=WP_MELEE`,
`weaponstate=WEAPON_READY`, `stats[STAT_HEALTH]=100`, `gravity=800`,
`speed=250`, `basespeed=250`, `standheight=40`, `crouchheight=16`,
`viewheight=DEFAULT_VIEWHEIGHT`, `groundEntityNum=ENTITYNUM_NONE`,
`clientNum=0`, `m_iVehicleNum=0`, everything else (fd, saberMove, zoomMode,
heldByClient, emplacedIndex, legsAnim/torsoAnim, …) zero; `tracemask =
MASK_PLAYERSOLID`, `baseEnt = g_entities`, `entSize = sizeof(gentity_t)`.

## Dump line format

One line per step (`s=N`; step 0 is the pre-move baseline emitted at `start`):
```
s=N t=<commandTime> org=<3×f32hex> vel=<3×f32hex> va=<3×f32hex> da=<3×int>
gnd=<groundEntityNum> pmf=<pm_flags hex> pmt=<pm_time> la=<legsAnim>:<legsTimer>
ta=<torsoAnim>:<torsoTimer> fl=<legsFlip><torsoFlip> bob=<bobCycle>
vh=<viewheight> ef=<eFlags hex> seq=<eventSequence>
ev=<events[0]>:<eventParms[0]>,<events[1]>:<eventParms[1]> wt=<weaponTime>
ws=<weaponstate> spd=<speed f32hex> wl=<waterlevel> wtp=<watertype>
nt=<numtouch> mn=<mins[2] f32hex> mx=<maxs[2] f32hex> xy=<xyspeed f32hex>
air=<inAirAnim> f2d=<fallingToDeath> fjz=<fd.forceJumpZStart f32hex>
ntr=<trace calls this Pmove> rng=<holdrand hex>
```
`ntr` resets per `Pmove` (a per-step trace-count tripwire; a `dt>66` row
aggregates the chop-loop's PmoveSingle calls). `rng` is the holdrand tripwire.

## The six scenarios (`golden/pmove.txt`, concatenated with `-- scenario X --`)

| # | scenario | exercises | steps |
| --- | --- | --- | --- |
| 1 | `idle` | ground snap, stand anim, resting stability | 21 |
| 2 | `walk-fwd` | forward accel, bobCycle, run/walk gait, BUTTON_WALKING | 61 |
| 3 | `strafe-turn` | diagonal move + per-cmd yaw turn, forward/right basis, friction | 31 |
| 4 | `jump-land` | PM_CheckJump (EV_JUMP=16), PMF_JUMP_HELD upmove=20 refeed, ballistic arc, PM_CrashLand (EV_FOOTSTEP=2) | 38 |
| 5 | `fall-onto-box` | free-fall, ground reacquire on a box top, PM_CrashLand quadratic (EV_FALL=11, material parm) | 41 |
| 6 | `wall-step` | PM_StepSlideMove step-up over a 16-tall ledge, clip-slide + corner crease on a 128-tall wall, one `dt=200` chop row | 34 |

Sanity anchors baked into the goldens: idle settles `gnd=1022` (ENTITYNUM_WORLD)
by step 1 with zero velocity; jump shows `EV_JUMP` + a ~19-step airborne arc
(`gnd=1023`, `pmf=2` PMF_JUMP_HELD, `la=1138` BOTH_JUMP1) + landing; fall parks
the box-top at `88.125`; wall-step steps up onto the ledge (`origin z 40.125`)
then stops at the wall; no `abort()` fires; `rng` stays `89abcdef` across all
six.
