# jka-rust

An idiomatic, opinionated Rust reimplementation of Star Wars Jedi Knight:
Jedi Academy — MP first (game module, then cgame/ui/engine), SP behind it —
built toward drop-in replacements for Raven's shipped binaries.

## Relationship to `jedi-academy-rust` (the oracle)

`jedi-academy-rust` is a faithful, 1:1 C-mirror Rust port of Raven's source.
**This project treats it as an oracle**, not a dependency. It is vendored under
[`oracle/`](oracle) (git submodule), which also carries Raven's original C/C++
under `oracle/` (SP `code/`, MP `codemp/`) — never edited. Every port is
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

## Status (2026-07-12)

- **Type port: complete** (Waves 0–7, both trees). Every ABI-crossing struct
  carries `size_of`/`offset_of!` static-asserts — a green build is the layout
  test.
- **MP game module (`jampgame`): transcribed and integrated.** `mp_game`
  compiles green with zero `todo!()` stubs and zero open `TODO: Port` markers;
  all `vmMain` dispatch arms are wired. The built cdylib exports
  `dllEntry`/`vmMain`/`GetModuleAPI`.
- **MP dedicated-server engine: closure complete (2026-07-12).** The full
  `jampDed` link set is transcribed and the workspace is **green** —
  `cargo check --workspace` builds every crate and **343 tests pass**, including
  the §F oracle-parity golden suites (GP2, ghoul2 bone/bolt/collision, ICARUS,
  RMG, StringEd, ROFF, tr_model). `qcommon`, `botlib`, and `server` are
  **closed** — zero `todo!()` stubs, zero open `TODO: Port` markers, zero extern
  forward-declaration blocks. The seven C++ subsystems (ICARUS, ghoul2-server,
  RMG/terrain, NPC nav, StringEd, ROFF, headless model/skin registry) are
  golden-verified against unmodified oracle TUs; the platform layer
  (`Sys_*`/console/sockets) is implemented natively in Rust behind the
  `EngineHost`/`EngineHooks` seams; the client draw surface is out of the
  dedicated scope by ruling (DEC-01).
- **CI**: pushes to `skeleton` and `master` run the full-workspace
  compile+test gate (`cargo build --workspace` + the oracle parity suites,
  single-threaded); `master`
  pushes additionally build `jampgame` for Windows/Linux × release/debug and
  publish the zips to the rolling
  [`latest` release](../../releases/tag/latest) under the exact filenames the
  engine loads (`jampgamex86_64.dll`, `jampgamex86_64.so`, …). 32-bit lanes
  are allowed failures pending the queued ILP32 layout-assert pass.
- **Remaining roadmap** (compiling green is not parity): boot/lifecycle wiring
  (hook install + core lifecycle so the dedicated server actually runs); the
  **referee swap** — oracle differential tests (single-threaded, replay-based)
  become the ground truth; then warning-zero, the safe-state migration that
  retires the transcription's raw-pointer scaffolding, and the ILP32
  layout-assert pass.
- Architectural decisions live in
  [`docs/decisions.md`](docs/decisions.md); remaining port surface is
  machine-audited against the link-set manifest
  (`tools/closure-prototype/out/engine/engine-port-order.tsv`); older audits
  live under [`docs/audits/`](docs/audits/).

## Ship targets

- **MP** (`jamp` engine): 3 loadable modules — `jampgame`, `cgame`, `ui`.
- **SP** (`jasp` engine): `jagame` only (SP cgame/ui are statically linked into
  the engine).
- **The MP dedicated server engine (`jampDed` equivalent) — in progress** (see
  Status); client engine and renderer are deferred by decision (DEC ledger).
