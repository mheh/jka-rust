//! Emits `BUILD_DATE`, standing in for C's `__DATE__` (no `env!`-visible
//! builtin in stable Rust), consumed by `g_init_game.rs`'s `gamedate` print
//! and `game_cvars.rs`'s `gamedate` cvar default. Copy of the qcommon
//! engine's `build.rs` logic (`crates/mp/engine/qcommon/build.rs`).

use native_build_date::build_date;

fn main() {
    println!("cargo:rustc-env=BUILD_DATE={}", build_date());
}
