# Goal

Build a Rust implementation of the Jedi Academy MP game module that can speak
the same engine/module ABI as Raven's original `jampgamex86.dll`.

The current MP ABI work gives the Rust code the same import/export
vocabulary as the Raven MP game module: `MpGameImport` models module-to-engine
syscall numbers, and `MpGameExport` models engine-to-module `vmMain` command
numbers. Those integer values must match the original Raven values exactly.

Matching those enum values is necessary, but it is not the whole drop-in DLL ABI.
A Rust replacement must work through this checklist before it can hot-swap for
Raven's `jampgamex86.dll`:

- [x] Scope the MP game ABI separately from MP cgame, MP UI, and SP surfaces.
- [x] Preserve the original MP game import/export integer vocabulary:
  `MpGameImport` for module-to-engine syscalls and `MpGameExport` for
  engine-to-module `vmMain` commands.
- [x] Generate or verify every MP game import/export value directly against
  Raven `codemp/game/g_public.h`, including explicit reset points like `100`,
  `200`, `250`, `300`, `400`, and `500`.
- [x] Port or explicitly classify every generated ABI arg/output TODO for the
  current MP/SP game, cgame, and UI surfaces.
- [x] Clear the ABI type inventory audit so there are no active
  `FIXME: create type` markers or unresolved referenced Rust type names.
- [x] Document enum-only/unused ABI tokens instead of leaving them as port TODOs
  when Raven exposes no wrapper, callsite, or transport behavior.
- [x] Expose exported C symbols with the original names: `dllEntry` and
  `vmMain` (`crates/jampgame`, the thin cdylib shell).
- [x] Match Raven's calling convention and `vmMain` argument/return word
  behavior. (The 12-word signature and `-1` fall-through are live; all
  command arms are wired to `mp_game` entrypoints.)
- [x] Store and call the engine syscall callback handed in through `dllEntry`
  (`ENGINE: OnceLock<CEngine>`, called by every `trap::*` wrapper).
- [x] Match argument packing for ints, pointers, and floats, including the
  original `PASSFLOAT` convention (`abi_transport::pass_float`).
- [x] Source or define ABI-correct layouts for every crossed type, including
  entities, player state, traces, cvars, commands, botlib data, ICARUS data, nav
  data, and Ghoul2 data (type port complete — Waves 0–7).
- [x] Verify struct sizes, alignments, and field offsets against Raven headers
  for all ABI-visible types (`size_of`/`offset_of!` static-asserts, badge-verified
  against clang ground truth).
- [ ] Model shared memory and global expectations visible to the engine.
- [ ] Implement engine-observable side effects for `GAME_INIT`,
  `GAME_RUN_FRAME`, `GAME_SHUTDOWN`, client lifecycle calls, botlib, ICARUS, nav,
  Ghoul2, and related systems.
- [x] Build the Rust game module as a native dynamic library with the filename
  and platform conventions expected by the engine (CI packages engine-named
  modules — `jampgamex86_64.dll`/`.so` — on every master push; the 32-bit
  `jampgamex86.dll`/`jampgamei386.so` lanes await the ILP32 assert pass).
- [x] Add an ABI smoke test that loads the Rust module through the same
  `dllEntry`/`vmMain` contract as the engine
  (`crates/jampgame/tests/abi_smoke.rs`: loads the built cdylib via the ported
  `native_platform` loader + real inbound syscall trampoline, drives
  `GAME_INIT` → 10 `GAME_RUN_FRAME`s → `GAME_SHUTDOWN` against a mock engine,
  asserting survival and structural side effects).
- [ ] Add differential tests against Raven/oracle behavior for representative
  imports, exports, and frame/client flows.
- [ ] Prove hot-swap behavior by replacing `jampgamex86.dll`/the platform
  equivalent with the Rust build and running the MP engine through init, map
  load, frame loop, client connect, and shutdown.

So the ABI target is:

> The MP Rust game module should become ABI-compatible with Raven's MP game
> module, such that the engine can load it through the same `dllEntry`/`vmMain`
> contract and observe equivalent behavior.

The present state is not yet a drop-in `jampgamex86.dll` replacement, but it is
no longer just the scaffold: the full jampgame logic port is transcribed and
integrated (`mp_game` compiles green, merged 2026-07-05; `todo!()` stubs and
open `TODO: Port` markers both at zero, all `vmMain` arms wired, CI publishing
engine-named modules), without losing the original ABI numbers or mixing MP
game, MP cgame, MP UI, and SP surfaces into one global enum. What remains
before hot-swap is oracle differential verification (the referee swap) and a
live-engine smoke test — compiling green is not verified parity.

## Related ABI Track: SP `GetGameAPI`

Before continuing deeper into the MP `dllEntry` / `vmMain` work, model Raven's
other game-module boundary: the SP `GetGameAPI` function-table ABI.

This is related to the syscall/vmMain work because it is another engine/module
ABI surface, but it is not the same transport:

- MP game, MP cgame, MP UI, SP cgame, and SP UI use the QVM-style shape:
  `dllEntry(syscall_callback)`, `vmMain(command, arg0..arg11)`, and typed
  syscall/vmMain wrappers over raw integer words.
- SP game uses `GetGameAPI(game_import_t *import) -> game_export_t *`, where
  the engine passes a `game_import_t` function-pointer table into the module and
  the module returns a `game_export_t` function-pointer table back to the engine.

The SP table ABI should be modeled beside the MP `vmMain` ABI, not forced into
the same enum transport.

- [x] Create a generic function-table ABI vocabulary alongside the existing
  syscall/vmMain vocabulary.
- [x] Port Raven SP `game_import_t` as a `#[repr(C)]` Rust import table with
  Raven comments and source line references
  (`crates/sp/abi/src/game/public/game_import_t.rs`; a handful of member
  types remain opaque behind `TODO: Port` markers — CGhoul2Info,
  IGhoul2InfoArray, CRagDollUpdateParams, the variadic `Printf`/`Error`/
  `SendServerCommand` args — see the marker inventory).
  - [ ] Create the SP game ABI type foundation for table fields:
    `qboolean`, `fileHandle_t`, `fsMode_t`, `cvar_t`, `gentity_t`,
    `usercmd_t`, `trace_t`, `vec3_t`, `qhandle_t`, `memtag_t`,
    `SavedGameJustLoaded_e`, and related crossed types.
  - [ ] Define the opaque-type policy for Raven C++ classes, pointers, and
    references used by `game_import_t`, including Ghoul2, ragdoll, gore,
    collision, and weather types.
  - [ ] Define the function-pointer convention for table fields, including how
    nullable callbacks and variadic callbacks like `Printf`, `Error`, and
    `SendServerCommand` are represented.
  - [ ] Preserve Raven field names and order for ABI traceability, even when
    names do not match Rust style.
  - [ ] Create a field manifest for every `game_import_t` entry with field name,
    C signature, Raven source line, Rust type translation, and notes.
  - [ ] Record the default-argument rule: C++ default arguments are not ABI
    fields, so Rust signatures model the full parameter list only.
  - [ ] Add a layout verification plan for `game_import_t`, including size and
    representative field offsets against Raven headers.
- [x] Port Raven SP `game_export_t` as a `#[repr(C)]` Rust export table with
  Raven comments and source line references
  (`crates/sp/abi/src/game/public/game_export_t.rs`).
  - [ ] Reuse the SP game ABI type foundation for export callbacks and shared
    variables such as `gentity_t`, `usercmd_t`, `qboolean`, and
    `SavedGameJustLoaded_e`.
  - [ ] Define the function-pointer convention for exported game callbacks and
    decide where unported callback behavior is stubbed.
  - [ ] Preserve Raven field names and order for ABI traceability, including
    global shared fields such as `gentities`, `gentitySize`, and
    `num_entities`.
  - [ ] Create a field manifest for every `game_export_t` entry with field name,
    C signature, Raven source line, Rust type translation, and notes.
  - [ ] Add a layout verification plan for `game_export_t`, including size and
    representative field offsets against Raven headers.
- [x] Add the SP `GetGameAPI` exported symbol only after the import/export
  tables exist (`crates/jagame`; `GI_Init` wiring still marked).
- [ ] Store or expose the imported SP engine table in a way that SP game code
  can call without every callsite handling raw unsafe pointers directly.
- [ ] Stub exported SP game table functions where behavior is not ported yet,
  while keeping ABI signatures and source references exact.
- [ ] Keep SP `GetGameAPI` separate from the MP `dllEntry` / `vmMain` hot-swap
  path, but reuse shared ABI primitives where the representation truly overlaps.
