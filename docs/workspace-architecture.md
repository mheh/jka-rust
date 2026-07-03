# Workspace Architecture

## Scope

`jka-rust` is a full, idiomatic Rust reimplementation of *Jedi Academy* — the
entire Raven build graph, not just the MP game module: both SP and MP trees,
their `game` / `cgame` / `ui` modules, the engine subsystems (`qcommon`,
`server`, `client`, `botlib`, `ghoul2`, `icarus`, `rmg`), the renderer, and the
host binaries. The end goal is a drop-in Rust replacement that a real engine can
load and, eventually, a fully native Rust engine.

Behavior is verified against the faithful 1:1 port under `oracle/` by
differential testing (`--features oracle`).

## Core idea: sharing is a dependency edge, not a folder

Raven has **no shared gameplay folder**. It expresses sharing by *compile-list
membership*: the same `.c` file is compiled into several module targets. The
`.q3asm` lists are the ground truth — e.g. `bg_*.c` physically live in
`codemp/game/` but `game.q3asm`, `cgame.q3asm`, and `ui.q3asm` each pull them in;
`q_shared`/`q_math` are pulled into all three modules *and* the engine;
`ui_shared` lives in `ui/` but `cgame.q3asm` also pulls it into cgame.

The idiomatic Rust encoding of "one file compiled into N targets" is **one crate
that N crates depend on.** So each Raven sharing tier becomes a crate, and the
dependency graph reproduces the `.q3asm` compile-lists exactly. The payoff:
`mp/cgame` depends on `mp/bg` but **not** on `mp/game`, so cgame physically
*cannot* reach `g_*` internals — the compiler enforces what Raven's build graph
enforced only by omission.

## Principles

- **Sharing = crate edge.** Reproduce Raven's compile-lists as `[dependencies]`.
- **Duplicate per-mode; do not unify.** SP and MP are near-duplicate but *not*
  identical (SP vs MP `q_shared.h` differ by ~3,466 lines). Each Raven-derived
  tier is per-mode and duplicated. Only genuinely Raven-free code (pure math,
  platform) is cross-mode, under `native/`.
- **Ravenish names.** Crate names track Raven subsystem names (`qshared`, `bg`,
  `qcommon`, `ghoul2`, …) for ABI traceability, even when not idiomatic Rust.
- **Nested directories, flat call sites.** A crate's directory location has no
  effect on how it is referenced. `crates/mp/bg/` is still `use bg::…`. Each
  module crate exposes a `prelude` re-exporting its shared crates under stable
  paths so call sites read like Raven's flat `#include` world.
- **Behavioral parity first, structural freedom always.** Match oracle behavior
  at the ABI seam exactly; internals are free to be idiomatic Rust.

## Crate graph

```
crates/
  native/                    # Tier -1: genuinely Raven-free, cross-mode
    math/                    #   vec3/matrix/angles (q_math is identical math)
    platform/                #   OS/threads/paths (replaces win32/unix/mac + sys)
    containers/              #   Rust-native stand-ins for Ratl/Ravl/Rufl/Ragl

  abi-transport/             # cross-mode ABI transport: OutboundSysCall,
                             #   InboundVmCall, Encode/Decode, vmMain word packing,
                             #   function-table shapes. Depends on native/platform
                             #   for the raw ABI fn-pointer aliases (RawSyscall,
                             #   RawVmMain), which it re-exports. No Raven types.

  jampgame/                  # MP game cdylib shell: ENGINE OnceLock, live
                             #   entrypoint exports, Dispatch match. Depends on
                             #   abi-transport + mp/game (logic).
  cgame/                     # MP cgame cdylib shell (same shape). SP cgame is
                             #   statically linked (DEC-07); no separate shell.
  ui/                        # MP ui cdylib shell (same shape). SP ui is
                             #   statically linked (DEC-07); no separate shell.
  jagame/                    # SP game cdylib shell (GetGameAPI table ABI).
                             #   Depends on abi-transport + sp/game (logic).

  mp/
    qshared/                 # Tier 0: codemp/game/q_shared.{h,c}  (re-exports native/math)
    bg/                      # Tier 1: codemp/game/bg_*
    uishared/                # Tier 2: codemp/ui/ui_shared
    abi/                     # MP engine<->module seam (dllEntry/vmMain surfaces)
    engine-select/           # mp_engine_select binding leaf: the one cfg'd
                             #   `pub type Engine` alias (wasm32 by target_arch;
                             #   Static by feature "static"; default CEngine/
                             #   NativeDll). Logic crates import it so `mod trap`
                             #   stays non-generic and cfg-free (SEAM-D13).
    game/                    # mp_game logic (transport-agnostic; jampgame shell wraps it)
    cgame/                   # mp_cgame logic (transport-agnostic; cgame shell wraps it)
    ui/                      # mp_ui logic (transport-agnostic; ui shell wraps it)
    engine/
      core/                  # mp_engine_core facade: aggregate `Engine`,
                             #   com_init/com_frame/com_shutdown/com_error
      qcommon/  server/  client/  botlib/  ghoul2/  icarus/  rmg/
    renderer/                # codemp/renderer (per-mode, split for authenticity)
    app/                     # openjk (client) + openjkded (dedicated); thin
                             #   bin shell depending on mp_engine_core

  sp/
    qshared/  bg/  uishared/
    abi/                     # SP: GetGameAPI table (game) + dllEntry/vmMain (cgame/ui)
    game/                    # sp_game logic (transport-agnostic; jagame shell wraps it)
    cgame/  ui/               # statically linked into sp/app via vmachine shim (DEC-07)
    engine/
      core/                  # sp_engine_core facade (mirrors mp_engine_core)
      qcommon/  server/  client/  ghoul2/  icarus/  rmg/
    renderer/                # code/renderer (per-mode)
    app/                     # openjk_sp (client); thin bin shell depending on
                             #   sp_engine_core
```

## Tier definitions (mapped to Raven compile-lists)

| Tier | Crate(s) | Raven source | Compiled/used by |
| --- | --- | --- | --- |
| -1 native | `native/{math,platform,containers}` | q_math (math only), platform dirs, Ra* template libs | everything, cross-mode |
| transport | `abi-transport` | QVM word ABI + `GetGameAPI` table shape | both `*/abi`, both engines |
| 0 qshared | `mp/qshared`, `sp/qshared` | `q_shared.{h,c}` | engine + game + cgame + ui (per mode) |
| 1 bg | `mp/bg`, `sp/bg` | `bg_*` (pmove, weapons, saber, panimate, saga, vehicles) | game + cgame + ui (per mode) |
| 2 uishared | `mp/uishared`, `sp/uishared` | `ui_shared` | cgame + ui (per mode) |
| 3 module (logic) | `mp/game`, `mp/cgame`, `mp/ui`, `sp/game` (+ `sp/cgame`, `sp/ui`, statically linked) | `g_*` / `cg_*`+`fx_*` / `ui_*` | transport-agnostic; wrapped by the shell crates below (or statically linked into `sp/app`) |
| shell | `jampgame`, `cgame`, `ui`, `jagame` | `dllEntry`/`vmMain`/`GetGameAPI` export shape | thin cdylib shells: `ENGINE` `OnceLock` + entrypoints + `Dispatch` match |
| engine | `*/engine/*`, `*/renderer` | `qcommon/server/client/botlib/ghoul2/icarus/RMG/renderer` | host binaries; `*/engine/core` is the aggregate facade (`Engine` + `com_init`/`com_frame`/`com_shutdown`/`com_error`) depended on by `*/app` |

> `MAX_GENTITIES` currently sits in `mp_engine_server` from the mechanical
> type-port but belongs in `mp_qshared` (oracle home
> `codemp/game/q_shared.h:1996,2004`); relocation is a slice-0 wiring task.

## Dependency edges

Module logic crates (per mode; transport-agnostic — no ABI/cdylib concerns):

| Crate | Depends on |
| --- | --- |
| `mp/game` | `mp/qshared`, `mp/bg`, `mp/abi`, `mp/engine-select` |
| `mp/cgame` | `mp/qshared`, `mp/bg`, `mp/uishared`, `mp/abi`, `mp/engine-select` |
| `mp/ui` | `mp/qshared`, `mp/bg`, `mp/uishared`, `mp/abi`, `mp/engine-select` |
| `mp/engine-select` | `abi-transport` (concrete `CEngine`/`Static` backends) |
| `mp/bg` | `mp/qshared` |
| `mp/uishared` | `mp/qshared`, `mp/bg` |
| `mp/abi` | `abi-transport`, `mp/qshared` |
| `mp/qshared` | `native/math`, `native/platform` |
| `abi-transport` | `native/platform` (re-exports its `RawSyscall`/`RawVmMain` fn-pointer aliases) |

Module cdylib shells (per mode) — thin, hosting only the `ENGINE` `OnceLock`,
live entrypoint exports, and the `Dispatch` match:

| Crate | Depends on |
| --- | --- |
| `jampgame` | `abi-transport`, `mp/game` |
| `cgame` | `abi-transport`, `mp/cgame` |
| `ui` | `abi-transport`, `mp/ui` |
| `jagame` | `abi-transport`, `sp/game` |

SP `cgame`/`ui` have no separate shell — they are statically linked into
`sp/app` behind the vmachine shim (DEC-07).

Per-build transport selection (SEAM-D13): `mp/engine-select` owns the single
cfg'd `pub type Engine` alias — `cfg(target_arch = "wasm32")` picks the wasm
backend, Cargo feature `static` picks `Static`, default is `CEngine`
(`NativeDll`); shells select it (`jampgame` et al. take the default, a
static-linking engine build enables `static`). Known cost: NativeDll and Static
shells on the same host triple cannot share one feature-unified
`cargo build --workspace` graph — those builds go per-package. SP needs no
select crate: `sp/game`'s `mod gi` binds the `game_import_t` table directly
(SEAM-D2, always native) and SP `cgame`/`ui` are always `Static` — their
aliases are fixed.

Engine (per mode): `mp/engine/*` depend on `mp/qshared`, `abi-transport`, and
`native/*`; `ghoul2` is depended on by both `engine/*` and `cgame` (Raven shares
it that way). `mp/engine/core` (package `mp_engine_core`) is the aggregate
facade: it depends on the other `mp/engine/*` subcrates, defines the aggregate
`pub struct Engine`, and hosts `com_init`/`com_frame`/`com_shutdown`/`com_error`.
`mp/app` is a thin bin shell depending only on `mp/engine/core` and hosts the
module cdylib shells. SP mirrors the same edges via `sp/engine/core` (package
`sp_engine_core`); SP `game` uses the `GetGameAPI` table half of
`abi-transport` instead of `dllEntry`/`vmMain`.

## Migration mapping (current `src/` -> target crate)

> **Migration complete** — `src/` has been dissolved into `crates/`; this table
> is kept as the historical record of where things went.

| Current | Target | Notes |
| --- | --- | --- |
| `src/lib.rs` | dissolves | monolith split into the crates above |
| `src/shared/vector.rs`, `platform.rs` | `native/math`, `native/platform` | genuinely Raven-free |
| `src/shared/{trajectory,collision,cvar,entity_shared,fsMode_t,pc_token_t,sharedIKMoveParams_t}.rs` | `mp/qshared` (dup to `sp/qshared` later) | these are `q_shared.h`-derived, not native |
| `src/common/mp/qcommon/*` | `mp/qshared` (or `mp/engine/qcommon` where engine-only) | classify per file |
| `src/common/mp/bg/*` | `mp/bg` | |
| `src/common/mp/{game,cgame,ui}/*` | `mp/{game,cgame,ui}` module commons | |
| `src/common/sp/*` | `sp/*` mirror | |
| `src/abi/generic/*` (transport, message, table, inbound, outbound, vm_main) | `abi-transport` | cross-mode |
| `src/abi/mp/*` (surfaces + token files) | `mp/abi` | |
| `src/abi/sp/*` | `sp/abi` | |
| `src/abi/entrypoints.rs` | each module cdylib `lib.rs` | `dllEntry`/`vmMain`/`GetModuleAPI`/`GetGameAPI` |
| `src/game/{entity,level,client}/*` (g_local.h structs) | `mp/game` (private) | keep Raven layout |
| `src/modules/{mp,sp}/*` | dissolve into `*/game,cgame,ui` | |
| `src/engine/*` (PLAN.md) | `mp/engine/*` design input | |
| `src/bg/*` | `mp/bg` | |
| `src/ffi/*` | `native/platform` or `abi-transport` | classify |
| `src/codemp/` | delete | empty |

## Call-site ergonomics

Each module crate has a `prelude` module re-exporting its dependency crates under
stable, Ravenish paths:

```rust
// mp/cgame/src/prelude.rs
pub use qshared::*;        // vec3_t, entityState_t, playerState_t, trace_t, ...
pub use bg::{self, *};     // pmove, weapon/saber defs
pub use uishared as ui_shared;
pub use abi::trap;         // trap::CG_* call sites
```

Call sites do `use crate::prelude::*;` and keep Raven's flat feel while the crate
graph enforces the tier boundaries underneath.

## Open items

- **Renderer:** per-mode split (chosen, for authenticity). Revisit an
  OpenJK-style unified `rd-common` + backends only if a real need appears.
- **`native/` may stay small:** most "shared" types trace to `q_shared.h` and are
  therefore per-mode. `native/math` is the main genuine cross-mode crate.
- **SP `cgame`/`ui` transport:** ~~confirm whether SP builds these as QVM modules
  or links them differently~~ — resolved per `decisions.md` DEC-07: statically
  linked into `sp/app` behind the vmachine shim, matching shipped `jasp`
  (design in `docs/architecture/module-loading.md`).
