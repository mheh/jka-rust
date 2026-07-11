//! Builds the C-variadic half of the botlib import `Print` callback
//! (`src/botlib_import_trampoline.c`) — stable Rust cannot define a C-variadic
//! fn, so the `botlib_import_t.Print` slot (`fn(int, char*, ...)`) needs a C
//! shim, mirroring qcommon's `game_syscall_trampoline.c` (skeleton-findings
//! resolution 1, 2026-07-03).

fn main() {
    println!("cargo:rerun-if-changed=src/botlib_import_trampoline.c");
    cc::Build::new()
        .file("src/botlib_import_trampoline.c")
        .compile("botlib_import_trampoline");
}
