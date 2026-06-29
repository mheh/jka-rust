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
- [ ] Generate or verify every MP game import/export value directly against
  Raven `codemp/game/g_public.h`, including explicit reset points like `100`,
  `200`, `250`, `300`, `400`, and `500`.
- [ ] Expose exported C symbols with the original names: `dllEntry` and
  `vmMain`.
- [ ] Match Raven's calling convention and `vmMain` argument/return word
  behavior.
- [ ] Store and call the engine syscall callback handed in through `dllEntry`.
- [ ] Match argument packing for ints, pointers, and floats, including the
  original `PASSFLOAT` convention.
- [ ] Source or define ABI-correct layouts for every crossed type, including
  entities, player state, traces, cvars, commands, botlib data, ICARUS data, nav
  data, and Ghoul2 data.
- [ ] Verify struct sizes, alignments, and field offsets against Raven headers
  for all ABI-visible types.
- [ ] Model shared memory and global expectations visible to the engine.
- [ ] Implement engine-observable side effects for `GAME_INIT`,
  `GAME_RUN_FRAME`, `GAME_SHUTDOWN`, client lifecycle calls, botlib, ICARUS, nav,
  Ghoul2, and related systems.
- [ ] Build the Rust game module as a native dynamic library with the filename
  and platform conventions expected by the engine.
- [ ] Add an ABI smoke test that loads the Rust module through the same
  `dllEntry`/`vmMain` contract as the engine.
- [ ] Add differential tests against Raven/oracle behavior for representative
  imports, exports, and frame/client flows.
- [ ] Prove hot-swap behavior by replacing `jampgamex86.dll`/the platform
  equivalent with the Rust build and running the MP engine through init, map
  load, frame loop, client connect, and shutdown.

So the ABI target is:

> The MP Rust game module should become ABI-compatible with Raven's MP game
> module, such that the engine can load it through the same `dllEntry`/`vmMain`
> contract and observe equivalent behavior.

The present state is not yet a drop-in `jampgamex86.dll` replacement. It is the
scaffold for making that replacement possible without losing the original ABI
numbers or mixing MP game, MP cgame, MP UI, and SP surfaces into one global enum.
