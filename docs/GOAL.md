# Goal

Build a Rust implementation of the Jedi Academy MP game module that can speak
the same engine/module ABI as Raven's original `jampgamex86.dll`.

The current MP boundary work gives the Rust code the same import/export
vocabulary as the Raven MP game module: `MpGameImport` models module-to-engine
syscall numbers, and `MpGameExport` models engine-to-module `vmMain` command
numbers. Those integer values must match the original Raven values exactly.

Matching those enum values is necessary, but it is not the whole drop-in DLL ABI.
A Rust replacement must also match:

- exported C symbols: `dllEntry` and `vmMain`
- calling convention and `vmMain` argument/return word behavior
- syscall callback storage handed in through `dllEntry`
- argument packing for ints, pointers, and floats via the original `PASSFLOAT`
  convention
- struct layout for every crossed type, including entities, player state, traces,
  cvars, commands, botlib data, ICARUS data, nav data, and Ghoul2 data
- shared memory/global expectations visible to the engine
- all engine-observable side effects for init, frames, shutdown, clients, botlib,
  ICARUS, nav, Ghoul2, and related systems

So the boundary target is:

> The MP Rust game module should become ABI-compatible with Raven's MP game
> module, such that the engine can load it through the same `dllEntry`/`vmMain`
> contract and observe equivalent behavior.

The present state is not yet a drop-in `jampgamex86.dll` replacement. It is the
scaffold for making that replacement possible without losing the original ABI
numbers or mixing MP game, MP cgame, MP UI, and SP surfaces into one global enum.
