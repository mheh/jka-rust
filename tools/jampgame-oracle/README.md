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

## Normalizations (documented divergences — porting-rules §19)

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

## Single-threading

The oracle keeps its RNG in file statics, so each family is dumped by a fresh
process. The Rust side mirrors that constraint: one `#[test]` per family, one
`Rng` per family, sub-checks sequential inside — never parallel draws against a
shared generator.
