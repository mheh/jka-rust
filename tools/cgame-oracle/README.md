# cgame-oracle

Builds Raven's **unmodified** MP `cgame` module from the oracle source into a
loadable dylib - `build/liboraclecgame.dylib` (arm64) - and proves a mock engine
can drive its `dllEntry`/`vmMain` handshake.

This is the reference module for the **C6b demo-referee** (docs/decisions.md
DEC-48 ruling 2): the recorded engine->module input stream from one demo playback
is replayed in lockstep to BOTH this oracle dylib and our Rust `cgame` cdylib, and
their outgoing trap streams are byte-diffed. This script + `smoke.c` only
establish (a) the oracle builds as a working loadable module and (b) a mock engine
loads and drives it through `dllEntry`/`vmMain`.

The oracle tree (`oracle/**`) is **never** edited. Everything is compiled from a
throwaway copy under `build/src/` (gitignored). This is the `tools/referee-oracle`
recipe retargeted from the QAGAME game module to the CGAME client module.

## What's built

`build.sh` compiles **51 TUs** and links them into `build/liboraclecgame.dylib`.
`dllEntry` and `vmMain` are exported with C linkage (unmangled) for `dlsym`. The
module is self-contained: its only undefined symbols are standard libc/libm
(`memcpy`, `sin`, `sqrtf`, `atoi`, `strcpy`, `sprintf`, ...), all resolved from
libSystem at `dlopen` (the smoke's `RTLD_NOW` binds every one).

### TU census (authoritative: `oracle/codemp/cgame/JK2_cgame.vcproj`, Release)

The vcproj Source Files filter lists **52** `.c` TUs; **51** are built (bg_lib.c is
excluded - see below).

- **25 cgame-local** (`codemp/cgame/`): `cg_consolecmds`, `cg_draw`,
  `cg_drawtools`, `cg_effects`, `cg_ents`, `cg_event`, `cg_info`, `cg_light`,
  `cg_localents`, `cg_main`, `cg_marks`, `cg_newDraw`, `cg_players`,
  `cg_playerstate`, `cg_predict`, `cg_saga`, `cg_scoreboard`, `cg_servercmds`,
  `cg_snapshot`, `cg_strap`, `cg_syscalls`, `cg_turret`, `cg_view`,
  `cg_weaponinit`, `cg_weapons`.
- **9 effects** (`codemp/cgame/`): `fx_blaster`, `fx_bowcaster`, `fx_bryarpistol`,
  `fx_demp2`, `fx_disruptor`, `fx_flechette`, `fx_force`, `fx_heavyrepeater`,
  `fx_rocketlauncher`.
- **14 shared-with-game** (`codemp/game/`): the four vehicle NPCs `AnimalNPC`,
  `FighterNPC`, `SpeederNPC`, `WalkerNPC`; `bg_g2_utils`, `bg_misc`, `bg_panimate`,
  `bg_pmove`, `bg_saber`, `bg_saberLoad`, `bg_saga`, `bg_slidemove`,
  `bg_vehicleLoad`, `bg_weapons`; `q_math`, `q_shared`.
- **1 shared-with-ui** (`codemp/ui/`): `ui_shared`.

`cg_playeranimate.c` exists in `codemp/cgame/` but is **not** a vcproj TU, so it is
not built (it carries no `main`/entrypoint and nothing `#include`s it).

### The one vcproj TU excluded

`..\game\bg_lib.c` is the only listed source marked `ExcludedFromBuild="TRUE"` in
**every** config - Release, Final, Debug, Debug(SH)
(`oracle/codemp/cgame/JK2_cgame.vcproj:309-333`). Its `rand`/`srand`/`qsort`
bodies sit under `#ifdef Q3_VM` (QVM-only), so the native module never links them.
Dropped, matching retail.

## Toolchain: real GCC required (not Apple clang)

Same as referee-oracle: the unmodified 32-bit-era source has `FOFS`/`CGFOFS`
`((int)&(((T*)0)->x))` casts that narrow a 64-bit pointer to `int` - GCC's
**`-fpermissive`** downgrades that to a warning; Apple clang ignores `-fpermissive`
(silent no-op). The values are small field offsets, so the truncation is
numerically harmless. A 32-bit build is not an option (arm64 macOS cannot `dlopen`
a 32-bit dylib).

```sh
brew install gcc        # provides g++-16 (or -15/-14/-13)
```

`build.sh` auto-detects `g++-1x` (in `PATH` or `/opt/homebrew/bin`); override with
`CXX=/path/to/g++`. It refuses Apple clang with a clear message.

## Compiled as C, not C++

The vcproj's `CompileAs="0"` is compile-by-extension, so retail built these `.c`
files as **C**. cgame's headers (`q_shared.h`, `tr_types.h`, `bg_public.h`,
`cg_public.h`, `ghoul2/G2.h`) are all `#define`/struct-only - no C++ classes,
unlike the game module's ghoul2/icarus class headers that force referee-oracle to
compile as C++. So `build.sh` uses `-x c -std=gnu99`. Two payoffs:

- **retail-faithful** - matches how MSVC compiled the module.
- **free unmangled exports** - cgame, unlike `game/g_syscalls.c` +
  `game/g_main.c`, never wraps `dllEntry`/`vmMain` in `#ifdef __linux__ extern "C"`
  (`cg_syscalls.c:10`, `cg_main.c:190`). As C there is no mangling, so the exports
  come out as plain `dllEntry`/`vmMain` with no source patch.

Compiling as C also drops two referee-oracle C++-only patches: no `qboolean=int`
activation (`bool` assigns to the enum fine in C) and no libm double-promotion
macros (`sin`/`sqrt`/... resolve to the double libm with no overload ambiguity,
exactly as retail's C compile did).

## Defines

From `JK2_cgame.vcproj:30` Release (`NDEBUG;WIN32;_WINDOWS;MISSIONPACK;_JK2;CGAME`),
with the win32 pair swapped for the host branch:

| define | why |
| --- | --- |
| `CGAME` | this is the client DLL (vs game/ui); selects the cgame code paths, and makes `ui_shared.c` skip its `ui_local.h`/`client.h` includes (`ui_shared.c:4-6`) |
| `_JK2` | JK2 tree. The four vehicle-NPC TUs bridge `_JK2` -> `_JK2MP` themselves (`WalkerNPC.c:25-28` etc.), `bg_vehicleLoad.c:5` self-`#define`s `_JK2MP`, and no other TU here reads `_JK2MP` - so `_JK2` (what the vcproj ships, **not** `_JK2MP`) is the faithful define |
| `MISSIONPACK` | mission-pack tree (matches the vcproj) |
| `NDEBUG` | retail was an MSVC Release build - asserts compiled away |
| `__linux__` | host branch: selects the **macro** `SnapVector` (past the x86 `__asm fld/fistp` one, `q_shared.h:1408`, which cannot assemble on arm64) and `ID_INLINE inline` |
| `_FORTIFY_SOURCE=0` | no fortify wrappers |

## FP regime flags (this defines parity)

```
-fno-fast-math -ffp-contract=off -O2 -fsigned-char
```

- **`-ffp-contract=off`** - no fused multiply-add. rustc/LLVM do not contract
  `a*b+c` by default, so the oracle must not either or the two disagree in the low
  bits. This is the parity-defining flag.
- **`-fno-fast-math`** - strict IEEE; matches rustc's default.
- **`-O2`** - matches the Rust release profile.
- **`-fsigned-char`** - pins retail `char` semantics on platforms defaulting
  unsigned.

## Flags beyond the referee-oracle set

Two host quirks the game module never hit surface in cgame; both are handled with
flags/copy-normalizations, **never** an oracle edit:

- **`min`/`max` from MSVC `<stdlib.h>`** - `cg_players.c:10574` calls bare
  `max(...)`; MSVC's `<stdlib.h>` defines `min`/`max` macros on the retail PC build
  and Raven only declares its own under `Q3_VM`/`_XBOX` (`q_shared.h:76-77,94-95`),
  so on the normal compile it resolves to the CRT macro. gcc's stdlib has none, so
  `shim/oracle_shim.h` supplies the MSVC-identical `#ifndef`-guarded pair.
- **backslash include separators** - some cgame sources spell includes with MSVC
  backslashes (`cg_ents.c:9` `#include "..\game\q_shared.h"`); MSVC resolves them,
  unix gcc reads them literally. A copy-tree pass flips `\` -> `/` inside
  `#include "..."` directives only (a path-separator normalization, ABI-neutral).

## Compile shim

`shim/oracle_shim.h` is force-included (`-include`) before every TU. It (1)
defuses Raven's `float powf(float x, int y)` (`q_shared.h:1242`) - a conflicting
redeclaration of libm's `powf` - by `#include <math.h>` first, then
`#define powf raven_powf` so Raven's decl/def/callers all move out of libm's way
together; and (2) supplies the `min`/`max` macros described above.

## Source normalizations (a copy, never the oracle)

Same parity class as referee-oracle - retail-win32 rounding/RNG/64-bit width, none
touching cgame program logic:

- **`SnapVector`** (`q_shared.h:1404`): the `__linux__` macro **truncates** via
  `(int)` casts; retargeted to `rint()` (round-to-nearest-even under the default FP
  env - fistp's semantics, the port's parity bar).
- **MSVC-CRT `rand`/`srand`** (after `q_shared.h:89` `#include <limits.h>`): six
  cgame TUs (`cg_ents`, `cg_localents`, `cg_event`, `cg_marks`, `cg_players`,
  `cg_weapons`) call `rand()`/`srand()` directly, so effect randomization diverges
  from retail unless routed to a retail-exact MSVC-semantics LCG
  (`holdrand*214013+2531011`, `(holdrand>>16)&0x7fff`). State lives in the
  `q_math.c` copy.
- **`Q_rsqrt` `long i`** (`q_math.c:618`) -> `int` - the type-pun reads 8 bytes
  from a 4-byte float at LP64 width (UB); retail i386 `long` is 32-bit.
- **`holdrand`** (`q_math.c:1432`) `unsigned long` -> `unsigned int` - at LP64
  width `holdrand >> 17` spans the full register and `flrand`/`irand` blow past
  `[0, 32767]`; force retail 32-bit state.
- **`vmMain` word width** (`cg_main.c:190`): Raven's `int vmMain(int command, int
  arg0..arg11)` is 32-bit-era; our engine's `VM_Call` passes 12 pointer-width
  words. On LP64 the `int` params truncate every pointer-carrying arg
  (`CG_GET_ORIGIN`'s `(float*)arg1`, `CG_ROFF_NOTETRACK_CALLBACK`'s
  `(const char*)arg1`) and the four `return (int)ptr` arms
  (`CG_GET_GHOUL2`/`CG_GET_MODEL_LIST`/`CG_GET_ORIGIN_TRAJECTORY`/
  `CG_GET_ANGLE_TRAJECTORY`, `cg_main.c:232,237,294,297`) truncate handles the
  engine reads back. Params, return, and those four casts are widened to GCC's
  builtin `__INTPTR_TYPE__` (no include). Other arms narrow `intptr_t -> int`
  implicitly - the 32-bit behavior for the small values they carry. Mirrors
  referee-oracle's `g_main.c` vmMain widening (G1).

## Smoke test - the acceptance proof

`smoke.c` (built and run by `build.sh`) `dlopen`s the module, `dlsym`s
`dllEntry` + `vmMain`, installs a stub syscall through `dllEntry`, then calls
`vmMain` with a command word no export carries (`0x7fff`). Raven's `vmMain` default
arm (`cg_main.c:354-358`) routes `CG_Error("vmMain: unknown command %i")` back out
as a `CG_ERROR` (=1) syscall and then returns `-1`. The stub records the `CG_ERROR`
and verifies the message; the harness asserts `vmMain` returned `-1`. One drive
exercises dlopen, the `dllEntry` handshake, the outbound syscall path, and the
dispatch fall-through - the same shape as `crates/cgame/tests/abi_smoke.rs` and
referee-oracle's phase-1 `oracle_smoke`. (The real engine longjmps out of
`CG_ERROR`; the stub returns, which lets `vmMain` run on to `return -1`.)

## How to run

```sh
tools/cgame-oracle/build.sh     # needs `brew install gcc`; builds + runs the smoke
```

`build/` is gitignored.
