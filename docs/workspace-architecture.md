# Workspace Architecture

## Scope

`jka-rust` is a full, idiomatic Rust reimplementation of *Jedi Academy* — the
entire Raven build graph, not just the MP game module: both SP and MP trees,
their `game` / `cgame` / `ui` modules, the engine subsystems (`qcommon`,
`server`, `client`, `botlib`, `ghoul2`, `icarus`, `rmg`), the renderer, and the
host binaries. The end goal is a drop-in Rust replacement that a real engine can
load and, eventually, a fully native Rust engine.

Behavior is verified against the Raven source under `oracle/` by differential
testing (golden fixtures + the A/B referee).

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
    sort/                    #   native_sort: canonical qsort (bg_lib body;
                             #   bg_lib retired, DEC-34)
    string/                  #   native_string: cross-mode string ops (&str/char
                             #   surface; the CString-removal seam, #13)
    build-date/              #   native_build_date: build-stamp helper
    types/                   #   native_types: cross-mode Raven scalar/handle
                             #   primitives byte-identical across SP/MP
                             #   q_shared.h (qboolean + lowercase qtrue/qfalse,
                             #   fileHandle_t/clipHandle_t/qhandle_t/etc.,
                             #   byte/word/ulong, mdxaBone_t, MAX_QPATH),
                             #   re-exported by each mode's qshared umbrella

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
                             #   Depends on abi-transport + sp/game (logic)
                             #   + sp/abi + sp/qshared (table + member types).

  mp/
    qshared/                 # Tier 0: codemp/game/q_shared.{h,c}  (re-exports native/math)
    bg/                      # Tier 1: codemp/game/bg_*
    uishared/                # Tier 2: codemp/ui/ui_shared
    abi/                     # MP engine<->module seam (dllEntry/vmMain surfaces)
    engine-select/           # mp_engine_select binding leaf: the one cfg'd
                             #   `pub type Engine` alias (Static by feature
                             #   "static"; default CEngine/NativeDll). Logic
                             #   crates import it so `mod trap` stays
                             #   non-generic and cfg-free (SEAM-D13).
    host-interface/          # mp_host_interface: EngineHost/PlatformHost traits
                             #   + MockHost fixture; owns the parsed-once mdx
                             #   views (MdxaRef/MdxmRef) the engine hands the
                             #   game (ghoul2 block ownership, DEC-35 / task #17)
    game/                    # mp_game logic (transport-agnostic; jampgame shell wraps it)
    cgame/                   # mp_cgame logic (transport-agnostic; cgame shell wraps it)
    ui/                      # mp_ui logic (transport-agnostic; ui shell wraps it)
    engine/
      core/                  # mp_engine_core facade: aggregate `Engine`,
                             #   com_init/com_frame/com_shutdown (+ com_error
                             #   *recovery*; com_error itself is defined one tier
                             #   lower in qcommon, state-ownership.md STATE-D7)
      qcommon/  server/  client/  botlib/  ghoul2/  icarus/  rmg/
    renderer/                # mp_renderer: codemp/renderer CPU frontend
                             #   (per-mode, split for authenticity)
    renderer-gpu/            # mp_renderer_gpu: the wgpu backend the render
                             #   thread owns, plus the world/ui_host harness bins
    app/                     # mp_app: the jampded dedicated-server bin; target
                             #   shape is a thin shell over mp_engine_core
                             #   (legacy leaf edges remain, see its Cargo.toml)
    client-app/              # mp_client_app: the jamp client bin (DEC-56).
                             #   Main thread = winit loop, "jamp-sim" = com
                             #   loop, "jamp-render" = the wgpu device

  sp/
    qshared/  bg/  uishared/
    abi/                     # SP: GetGameAPI table (game) + dllEntry/vmMain (cgame/ui)
    game/                    # sp_game logic (transport-agnostic; jagame shell wraps it)
    cgame/  ui/               # statically linked into sp/app via vmachine shim (DEC-07)
    engine/                  # no sp_engine_core exists yet; the facade that
                             #   mirrors mp_engine_core is future work
      qcommon/  server/  client/  ghoul2/  icarus/  rmg/
    renderer/                # code/renderer (per-mode)
    app/                     # sp_app (client); bin shell over
                             #   sp_engine_{qcommon,server,client} +
                             #   sp_renderer + sp_abi (no core facade yet)

  testkit/                   # shared test fixtures; dev-dependency only

tools/cgame-referee/         # probe + shim recorder packages; standalone
                             #   (empty [workspace] table), NOT workspace members
```

## Tier definitions (mapped to Raven compile-lists)

| Tier | Crate(s) | Raven source | Compiled/used by |
| --- | --- | --- | --- |
| -1 native | `native/{math,platform,containers,sort,string,build-date,types}` | q_math (math only), platform dirs, Ra* template libs, `bg_lib` qsort (DEC-34), cross-mode string ops + `q_shared.h` scalar/handle primitives | everything, cross-mode |
| transport | `abi-transport` | QVM word ABI + `GetGameAPI` table shape | both `*/abi`, both engines |
| 0 qshared | `mp/qshared`, `sp/qshared` | `q_shared.{h,c}` | engine + game + cgame + ui (per mode) |
| 1 bg | `mp/bg`, `sp/bg` | `bg_*` (pmove, weapons, saber, panimate, saga, vehicles) | game + cgame + ui (per mode) |
| 2 uishared | `mp/uishared`, `sp/uishared` | `ui_shared` | cgame + ui (per mode) |
| 3 module (logic) | `mp/game`, `mp/cgame`, `mp/ui`, `sp/game` (+ `sp/cgame`, `sp/ui`, statically linked) | `g_*` / `cg_*`+`fx_*` / `ui_*` | transport-agnostic; wrapped by the shell crates below (or statically linked into `sp/app`) |
| shell | `jampgame`, `cgame`, `ui`, `jagame` | `dllEntry`/`vmMain`/`GetGameAPI` export shape | thin cdylib shells: `ENGINE` `OnceLock` + entrypoints + `Dispatch` match (MP shells only — `jagame` fills the `game_export_t` table directly, no command dispatch; settled SP mapping 2026-07-03, state-ownership.md) |
| engine | `*/engine/*`, `*/renderer`, `mp/renderer-gpu` | `qcommon/server/client/botlib/ghoul2/icarus/RMG/renderer` | host binaries; `mp/engine/core` is the aggregate facade (no SP twin exists yet) (`Engine` + `com_init`/`com_frame`/`com_shutdown`, plus `com_error`'s *recovery* — `com_error` itself is defined one tier lower in `*/engine/qcommon`, state-ownership.md STATE-D7) depended on by `*/app` |

## Dependency edges

Module logic crates (per mode; transport-agnostic — no ABI/cdylib concerns):

The rows list the tier and seam edges. Most per-mode crates also take the `native/*` runtime crates (`native_math`, `native_string`, `native_types`, and `native_sort` where qsort is used) as plain leaf edges. The tables do not repeat them.

| Crate | Depends on |
| --- | --- |
| `mp/game` | `mp/qshared`, `mp/bg`, `mp/abi`, `mp/engine-select`, `native/platform` (`zeroed_box`/`ZeroValid` impls — STATE-D9; skeleton-verified 2026-07-03) |
| `sp/game` | `sp/qshared`, `sp/bg`, `sp/abi`, `native/platform` (SP dual of the row above; `mod gi` binds `game_import_t` directly, no select crate — settled SP mapping 2026-07-03) |
| `mp/cgame` | `mp/qshared`, `mp/bg`, `mp/uishared`, `mp/abi`, `mp/engine-select` |
| `mp/ui` | `mp/qshared`, `mp/bg`, `mp/uishared`, `mp/abi`, `mp/engine-select` |
| `mp/engine-select` | `abi-transport` (concrete `CEngine`/`Static` backends) |
| `mp/bg` | `mp/qshared` |
| `mp/uishared` | `mp/qshared`, `mp/bg` (dev-only: `mp/engine/botlib`, driving the real tokenizer in the menu-parse golden test — not a shipping edge) |
| `mp/abi` | `abi-transport`, `mp/qshared` |
| `mp/qshared` | `native/math`, `native/platform`, `native/string`, `native/types` |
| `abi-transport` | `native/platform` (re-exports its `RawSyscall`/`RawVmMain` fn-pointer aliases) |

`mp/abi` **re-exports the four `abi-transport` seam traits**
(`Dispatch`/`InboundVmCall`/`Execute`/`OutboundSysCall`): it already depends on
`abi-transport` and is the seam crate by definition, so module logic crates name them
as `use mp_abi::…` through their existing `mp/abi` edge — **no logic crate gains an
`abi-transport` edge** (state-ownership STATE-Q12 resolution, 2026-07-03).

Module cdylib shells (per mode) — thin, hosting only the `ENGINE` `OnceLock`,
live entrypoint exports, and the `Dispatch` match:

| Crate | Depends on |
| --- | --- |
| `jampgame` | `abi-transport`, `mp/game` (`mp/game` re-exports `MpGameExport` at its crate root — `pub use mp_abi::game::exports::MpGameExport;` — so the shell's frozen two-edge set reaches the seam enum through the logic crate; state-ownership STATE-Q13 resolution, 2026-07-03) |
| `cgame` | `abi-transport`, `mp/cgame` |
| `ui` | `abi-transport`, `mp/ui` |
| `jagame` | `abi-transport`, `sp/game`, `sp/abi`, `sp/qshared` (table + member-signature types — skeleton-verified 2026-07-03) |

SP `cgame`/`ui` have no separate shell — they are statically linked into
`sp/app` behind the vmachine shim (DEC-07).

> **Two types named `Engine` exist, on opposite islands, never co-scoped**
> (disambiguation, 2026-07-03): `mp_engine_select::Engine` is the module-side
> cfg'd transport-backend alias (CEngine/Static) that `mod trap` wrappers
> take as `engine: &Engine`; `mp_engine_core::Engine` is the engine-island
> aggregate struct (`{common, sv, cl, cm, snd, icarus, rmg, render_models, re, fx, nav, roff, bot}`, state-ownership.md STATE-D5).
> Module crates cannot reach core; engine crates do not import select. Doc text
> must always crate-qualify the two.

Per-build transport selection (SEAM-D13): `mp/engine-select` owns the single
cfg'd `pub type Engine` alias — Cargo feature `static` picks `Static`, default
is `CEngine` (`NativeDll`); shells select it (`jampgame` et al. take the default, a
static-linking engine build enables `static`). Known cost: NativeDll and Static
shells on the same host triple cannot share one feature-unified
`cargo build --workspace` graph — those builds go per-package. SP needs no
select crate: `sp/game`'s `mod gi` binds the `game_import_t` table directly
(SEAM-D2, always native) and SP `cgame`/`ui` are always `Static` — their
aliases are fixed.

Engine (per mode): every `mp/engine/*` crate depends on `mp/qshared` and `native/*`. Only `mp/engine/qcommon` holds the `abi-transport` edge. Six engine crates (`qcommon`, `server`, `client`, `ghoul2`, `icarus`, `rmg`) also take `mp/host-interface`, the Stage-0 `EngineHost` services trait (ruling 56c). `ghoul2` is depended on by `server`, `client`, `core`, `mp/renderer`, and `mp/renderer-gpu`; `cgame` reaches it only through traps, not a crate edge. Three engine-to-module edges exist for shared helpers and the listen server: `botlib -> mp/game`, `client -> mp/game` + `mp/ui`. `server` and `client` also link `mp/renderer` (in `server` under the alias `mp_engine_renderer`). `mp/renderer` itself depends on `mp/qshared`, `mp/engine/qcommon`, `mp/host-interface`, `mp/engine/ghoul2`, and `native/*`. `mp/renderer-gpu` stacks the wgpu backend on `mp/renderer` and hosts the `ui_host` harness, whose extra edges (`mp/ui`, `mp/uishared`, `mp/bg`, `mp/engine/botlib`, `mp/engine/core`, `mp/engine/server`) belong to the harness bins, not the renderer. `mp/engine/core` (package `mp_engine_core`) is the aggregate
facade: it depends on the other `mp/engine/*` subcrates, defines the aggregate
`pub struct Engine`, and hosts `com_init`/`com_frame`/`com_shutdown` (plus
`com_error`'s recovery; `com_error` itself is defined one tier lower in
`mp/engine/qcommon` — state-ownership.md STATE-D7).
`mp/app` is the `jampded` dedicated-server bin. Its target shape is a thin shell over `mp/engine/core`; legacy leaf edges (`qcommon`, `server`, `client`, `renderer`, `abi`, `native/platform`) remain until their consumers migrate.
`mp/client-app` (package `mp_client_app`) is its client twin, the `jamp`
platform shell (DEC-56): the main thread runs the winit event loop, the sim
thread runs the com loop, and the render thread owns the wgpu device it takes
from `mp/renderer-gpu`. It is the only crate that turns on `mp_engine_client`'s
`sound_device` feature, so the cpal edge never reaches the `-p mp_app`
dedicated-server or ILP32 lanes.
SP has no core facade yet; `sp/app` links `sp/engine/{qcommon,server,client}` + `sp/renderer` + `sp/abi` directly. SP `game` uses the `GetGameAPI` table half of `abi-transport` instead of `dllEntry`/`vmMain`.

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

Two crates carry a `prelude` module: `mp/game` and `mp/bg`. Each prelude is pure re-exports, routed through the crate's frozen dependency set, so the skeleton-landed files open with `use crate::prelude::*;` and resolve Raven spellings without per-file import ceremony (see `crates/mp/game/src/prelude.rs`). The other module crates import the real package names directly (`use mp_qshared::…`, `use mp_bg::…`) at file top, per the import rules in `docs/porting-rules.md`. The crate graph enforces the tier boundaries either way.

## Open items

- **Renderer:** per-mode split (chosen, for authenticity) and landed for MP as `mp/renderer` (frontend) + `mp/renderer-gpu` (wgpu backend). Revisit an OpenJK-style unified `rd-common` + backends only if a real need appears.
- **`native/` may stay small:** most "shared" types trace to `q_shared.h` and are
  therefore per-mode. `native/math` is the main genuine cross-mode crate.
- **SP `cgame`/`ui` transport:** ~~confirm whether SP builds these as QVM modules
  or links them differently~~ — resolved per `decisions.md` DEC-07: statically
  linked into `sp/app` behind the vmachine shim, matching shipped `jasp`
  (design in `docs/architecture/module-loading.md`).
