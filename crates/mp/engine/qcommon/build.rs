//! Builds the C-variadic shim half of the SEAM-D11 inbound syscall trampoline
//! (`src/vm/game_syscall_trampoline.c`) — stable Rust cannot define a
//! C-variadic fn (skeleton-findings resolution 1, 2026-07-03).

fn main() {
    println!("cargo:rerun-if-changed=src/vm/game_syscall_trampoline.c");
    cc::Build::new()
        .file("src/vm/game_syscall_trampoline.c")
        .compile("game_syscall_trampoline");
}
