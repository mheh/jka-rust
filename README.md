# jka-rust

An idiomatic, opinionated Rust reimplementation of Star Wars Jedi Knight:
Jedi Academy — MP first (game module, then cgame/ui/engine), SP behind it —
built toward drop-in replacements for Raven's shipped binaries.

## Relationship to `jedi-academy-rust` (the oracle)

`jedi-academy-rust` is a faithful, 1:1 C-mirror Rust port of Raven's source.
**This project treats it as an oracle**, not a dependency. It is vendored under
[`oracle/`](oracle) (git submodule), which also carries Raven's original C/C++
under `oracle/oracle/` (SP `code/`, MP `codemp/`) — never edited. Every port is
verified against it: clang-verified layout static-asserts for types,
differential parity tests (`--features oracle`) for behavior. No FFI, no
extracted C.

This repo does not reuse the mirror port's layout — it takes its own structure:
per-module logic crates (`crates/mp/*`, `crates/sp/*`) under thin cdylib shells
(`crates/jampgame`, `crates/cgame`, `crates/ui`, `crates/jagame`) that export
the exact symbols the engines load. See
[`docs/workspace-architecture.md`](docs/workspace-architecture.md) for the
crate graph and [`docs/porting-rules.md`](docs/porting-rules.md) for how code
is ported.

## Status (2026-07-08)

- **Type port: complete** (Waves 0–7, both trees). Every ABI-crossing struct
  carries `size_of`/`offset_of!` static-asserts — a green build is the layout
  test.
- **MP game module (`jampgame`): transcribed and integrated.** `mp_game`
  compiles green with zero `todo!()` stubs and zero open `TODO: Port` markers;
  all `vmMain` dispatch arms are wired. The built cdylib exports
  `dllEntry`/`vmMain`/`GetModuleAPI`.
- **CI**: every push to `master` runs a full-workspace compile gate, then
  builds `jampgame` for Windows/Linux × release/debug and publishes the zips to
  the rolling [`latest` release](../../releases/tag/latest) under the exact
  filenames the engine loads (`jampgamex86_64.dll`, `jampgamex86_64.so`, …).
  32-bit lanes are allowed failures pending an ILP32 layout-assert pass.
- **Not yet verified**: compiling green is not parity. The next phase is the
  referee swap — oracle differential tests (single-threaded, replay-based)
  become the ground truth, followed by the safe-state migration that retires
  the transcription's raw-pointer scaffolding.
- Remaining port surface is tracked in
  [`docs/audits/marker-inventory-2026-07-08.md`](docs/audits/marker-inventory-2026-07-08.md),
  [`docs/audits/2026-07-07-marker-triage.md`](docs/audits/2026-07-07-marker-triage.md) (open-work
  enumeration), [`docs/audits/const-sweep-2026-07-08.md`](docs/audits/const-sweep-2026-07-08.md),
  [`docs/audits/gatesweep-2026-07-08.md`](docs/audits/gatesweep-2026-07-08.md), and
  [`docs/audits/per-file-oracle-audit.md`](docs/audits/per-file-oracle-audit.md); architectural
  decisions live in [`docs/decisions.md`](docs/decisions.md).

## Ship targets

- **MP** (`jamp` engine): 3 loadable modules — `jampgame`, `cgame`, `ui`.
- **SP** (`jasp` engine): `jagame` only (SP cgame/ui are statically linked into
  the engine).
- Eventually the engines themselves; the renderer is deferred by decision
  (DEC ledger).
