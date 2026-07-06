//! Oracle ABI smoke test — drive Raven's UNMODIFIED jampgame (the referee
//! reference module) through the EXACT same mock engine, transport wiring and
//! lifecycle+assertions as `abi_smoke.rs`, proving our harness can load and drive
//! the oracle DLL the identical way it drives our Rust cdylib. This is Stage-R
//! phase 1: the oracle DLL calling our mock through the SEAM-D11 trampoline.
//!
//! The oracle DLL is NOT produced by cargo — build it first with
//! `tools/referee-oracle/build.sh` (needs Homebrew `gcc`; see that dir's
//! README.md). This test is therefore `#[ignore]`d so `cargo test` / CI need no
//! C++ toolchain. Run it explicitly:
//!
//! ```sh
//! tools/referee-oracle/build.sh
//! cargo test -p jampgame --test oracle_smoke -- --ignored --test-threads=1 --nocapture
//! ```
//!
//! Everything substantive is shared verbatim with `abi_smoke.rs` via
//! `tests/common/mod.rs`; this file only points the shared drive at the oracle
//! dylib instead of the cargo-built cdylib.

mod common;

use std::path::PathBuf;

/// Absolute path to the artifact `tools/referee-oracle/build.sh` produces.
/// `CARGO_MANIFEST_DIR` is `crates/jampgame`; the tool lives at the repo root.
fn oracle_dylib() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest
        .parent()
        .and_then(|p| p.parent())
        .expect("repo root (crates/jampgame/../..)");
    repo_root.join("tools/referee-oracle/build/liboraclejampgame.dylib")
}

/// Load and drive Raven's unmodified jampgame through the full lifecycle.
#[test]
#[ignore = "requires tools/referee-oracle/build.sh (Homebrew gcc); run with --ignored"]
fn oracle_smoke_init_frames_shutdown() {
    let dylib = oracle_dylib();
    assert!(
        dylib.exists(),
        "oracle dylib not found at {}. Build it first: `tools/referee-oracle/build.sh`.",
        dylib.display()
    );
    common::run_on_engine_thread(dylib);
}
