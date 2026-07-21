//! Builds the C-variadic shim half of the SEAM-D11 inbound syscall trampoline
//! (`src/vm/game_syscall_trampoline.c`) — stable Rust cannot define a
//! C-variadic fn (skeleton-findings resolution 1, 2026-07-03). Also emits
//! `BUILD_DATE`, standing in for C's `__DATE__` (no `env!`-visible builtin in
//! stable Rust), consumed by `common_fns.rs`'s banner prints.

use native_build_date::build_date;

fn main() {
    println!("cargo:rerun-if-changed=src/vm/game_syscall_trampoline.c");
    cc::Build::new()
        .file("src/vm/game_syscall_trampoline.c")
        .compile("game_syscall_trampoline");

    println!("cargo:rustc-env=BUILD_DATE={}", build_date());
}
