//! ABI smoke test — drive the BUILT `jampgame` cdylib through the exact
//! engine/module contract (`dllEntry(syscall)` handshake, then
//! `vmMain(command, arg0..arg11)`) against a mock engine, asserting the module
//! survives `GAME_INIT` → several `GAME_RUN_FRAME`s → `GAME_SHUTDOWN` and
//! produces the structural side effects the engine relies on.
//!
//! This is the live realization of the GOAL.md checklist item "Add an ABI smoke
//! test that loads the Rust module through the same `dllEntry`/`vmMain` contract
//! as the engine".
//!
//! The mock engine, transport wiring, the whole `GAME_INIT` → frames → connect →
//! begin → command → disconnect → `GAME_SHUTDOWN` drive, and every structural
//! assertion live in `tests/common/mod.rs` — shared verbatim with
//! `oracle_smoke.rs`, which runs the SAME lifecycle against Raven's unmodified
//! jampgame (the referee reference module). This file only locates the built Rust
//! cdylib and kicks off that shared drive.
//!
//! Single-shot: the module's `ENGINE`/`WORLD` globals and the engine slot are
//! process singletons, so the whole lifecycle runs in ONE `#[test]` fn. The
//! suite is invoked `--test-threads=1`.

mod common;

/// Locate the built cdylib next to the test binary and drive the full lifecycle
/// on a generous-stack engine thread (see `common::run_on_engine_thread`).
#[test]
fn abi_smoke_init_frames_shutdown() {
    let dylib = common::locate_cargo_cdylib(&common::dylib_filename("jampgame"));
    common::run_on_engine_thread(dylib);
}
