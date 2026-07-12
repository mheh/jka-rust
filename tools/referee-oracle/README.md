# referee-oracle

Builds Raven's **unmodified** `jampgame` (the MP `QAGAME` game module) from the
oracle source into a loadable dylib — `build/liboraclejampgame.dylib` — and proves
our test harness can drive its full engine lifecycle.

This is Stage-R **phase 1**. This DLL is the reference module for the upcoming
referee differential harness: the oracle DLL and our Rust `jampgame` cdylib will
later be driven over identical inputs and byte-diffed. Phase 1 only establishes
(a) the oracle builds as a working loadable module and (b) our mock-engine harness
loads and drives it through `dllEntry`/`vmMain` exactly as it drives our port.

The oracle tree (`oracle/**`) is **never** edited. Everything is compiled
from a throwaway copy under `build/src/` (gitignored).

## What's built

- **`build.sh`** compiles all **89** `oracle/codemp/game/*.c` translation
  units — the complete QAGAME module (`g_*.c`, `bg_*.c`, `ai_*.c`, `NPC_*.c`,
  `w_*.c`, the vehicle NPCs `FighterNPC/SpeederNPC/WalkerNPC/AnimalNPC.c`,
  `q_math.c`, `q_shared.c`, `g_syscalls.c`, `tri_coll_test.c`, …) — and links them
  into `build/liboraclejampgame.dylib`. The module is self-contained: its only
  undefined symbols are standard libc/libm (`memcpy`, `sin`, `sqrt`, `atoi`,
  `strcpy`, …), resolved from libSystem at `dlopen`. `dllEntry` and `vmMain` are
  exported (C linkage, unmangled) for `dlsym`.

  All 89 codemp/game TUs are the module: there is no separate project file in the
  oracle drop, but every `.c` there compiles and links cleanly into one DLL, and
  none carries a `main()` or a rival entrypoint. The `bg_*` files are the
  shared-with-cgame sources compiled into the game DLL under `QAGAME` (their
  `_CGAME` branches are dead here).

## Toolchain: real GCC required (not Apple clang)

The unmodified source is 32-bit-era C++ (JKA codemp is C++: `.c` files with
`extern "C"`, ghoul2/icarus C++ class headers). Two things break a modern **clang**
64-bit build with no clang escape hatch:

- `FOFS(x) ((int)&(((gentity_t *)0)->x))` casts a 64-bit pointer to `int` — a hard
  error in C++ (`cast to smaller type loses information`). The values are small
  field offsets, so the truncation is numerically harmless; GCC's **`-fpermissive`**
  downgrades it to a warning. Apple clang ignores `-fpermissive` (silent no-op).

A 32-bit build (where `int` is pointer-width and these are non-issues) is not an
option: modern arm64 macOS cannot `dlopen` a 32-bit dylib.

So the build uses **Homebrew GCC**:

```sh
brew install gcc        # provides g++-16 (or -15/-14/-13)
```

`build.sh` auto-detects `g++-1x` (in `PATH` or `/opt/homebrew/bin`); override with
`CXX=/path/to/g++`. It refuses Apple clang with a clear message.

## Defines

| define | why |
| --- | --- |
| `QAGAME` | this is the game DLL (vs cgame/ui); selects the game code paths |
| `_JK2MP` | multiplayer tree — routes the vehicle/NPC TUs past their SP-only `#ifndef _JK2MP` includes (`g_functions.h`, `Ratl/string_vs.h`) to the MP `bg_vehicles.h` path |
| `__linux__` | wraps `dllEntry`/`vmMain` in `extern "C"` so the exports are unmangled and `dlsym`-findable |
| `_FORTIFY_SOURCE=0` | no `_FORTIFY_SOURCE` wrappers |

## FP regime flags (this defines parity)

```
-fno-fast-math -ffp-contract=off -O2
```

- **`-ffp-contract=off`** — no fused multiply-add. This is the one that matters:
  rustc/LLVM do **not** contract `a*b+c` into an FMA by default, so the oracle must
  not either, or the two would disagree in the low bits. This is why it is called
  out as parity-defining.
- **`-fno-fast-math`** — strict IEEE (no reassociation, no `-ffast-math` reciprocal
  tricks); matches rustc's default.
- **`-O2`** — matches the Rust release profile's optimization level.

(`build.sh` also passes `-fexceptions -funwind-tables` so a harness `panic!` — e.g.
on a module `G_ERROR` — can unwind cleanly back through the oracle's C++ frames;
the C++ compile has these on by default, they are stated for clarity.)

## The one source normalization (a copy, never the oracle)

`build.sh` copies `codemp/` + `ui/menudef.h` into `build/src/` and makes **exactly
one** change, to the copied `q_shared.h`: it activates Raven's **own**
`#define qboolean int` — the line Raven already ships under `#ifdef _XBOX` ("don't
want strict type checking on the qboolean", q_shared.h:353-355). On the normal PC
build `qboolean` is `typedef enum {qfalse, qtrue}`; C++11-and-later reject assigning
a `bool` (`==` result) to that enum, which the original MSVC build allowed. `enum
qboolean` and `int` are both 4 bytes with values 0/1, so this is **ABI-identical** —
it only restores the lax typing the code was written against. The oracle tree
itself is untouched.

## Compile shim

`shim/oracle_shim.h` is force-included (`-include`) before every TU. Its sole job:
defuse Raven's `float powf(float x, int y)` (q_shared.h:1242 / q_math.c:1476), whose
2-arg-with-`int` signature conflicts with libm's `float powf(float, float)`. It
`#include <math.h>` first (real `powf` declared behind its guard), then
`#define powf raven_powf` so Raven's declaration, definition, and callers all move
out of libm's way together. No other shims are needed — the full-module link
resolves every real definition.

## Smoke test — the acceptance proof

`crates/jampgame/tests/oracle_smoke.rs` loads `build/liboraclejampgame.dylib`
through the **real** ported module loader and drives it through the exact
engine/module contract against the **same** mock engine as `abi_smoke.rs` — the
whole mock, transport wiring, lifecycle and assertions are shared verbatim in
`crates/jampgame/tests/common/mod.rs`. Raven's DLL calling our mock through the
SEAM-D11 syscall trampoline is the referee acceptance proof.

Lifecycle driven: arm engine slot → `dllEntry(trampoline)` → `GAME_INIT` → 3
warm-up `GAME_RUN_FRAME`s → `CLIENT_CONNECT(0)` → `CLIENT_BEGIN(0)` → 10 connected
frames → `CLIENT_COMMAND(0,"say hello")` → 2 frames → `CLIENT_DISCONNECT(0)` → 2
frames → `GAME_SHUTDOWN`. The unmodified oracle survives all of it (43 distinct
syscalls exercised, 124 configstrings set).

The oracle DLL's `vmMain`/syscall use Raven's original 32-bit `int` widths while
the loader's `AbiWord` is `isize`; on arm64 this is benign for the small
non-negative lifecycle values (a write to a 32-bit result register zero-extends),
and the variadic syscall path is width-agnostic (pointers pass at natural width) —
so the identical trampoline drives both the oracle and our port.

### One mock enrichment the oracle forced

The mock's client-0 userinfo carries a full `forcepowers` config
(`7-1-032330000000001333`, Raven's own canonical `<rank>-<side>-<18 digits>`
fallback). This is **required**: with an empty `forcepowers`, Raven's
`WP_InitForcePowers` → `BG_LegalizedForcePowers` (w_force.c:277, bg_misc.c:439)
leaves its `int final_Powers[NUM_FORCE_POWERS]` (NUM_FORCE_POWERS = 18) stack array
uninitialized, then reads garbage as a force-power level and indexes
`bgForcePowerCost[i][countDown]` out of bounds → SIGSEGV in `CLIENT_BEGIN`. A real
client always sends `forcepowers`; our Rust port only survived an empty string
because Rust zero-inits the array. The enrichment is realistic mock data (like the
existing userinfo), not an assertion change, and `abi_smoke` still passes with it.

## How to run

```sh
# 1. build the oracle DLL from a clean tree (needs `brew install gcc`)
tools/referee-oracle/build.sh

# 2. drive it (ignored by default so CI needs no C++ toolchain)
cargo test -p jampgame --test oracle_smoke -- --ignored --test-threads=1 --nocapture

# the whole workspace stays green either way:
cargo build --workspace && cargo test --workspace -- --test-threads=1   # 54 passed
```

`build/` is gitignored.
