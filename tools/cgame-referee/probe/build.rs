//! Compiles the C variadic forward the probe sends its traps through.

fn main() {
    println!("cargo:rerun-if-changed=src/syscall.c");
    cc::Build::new()
        .file("src/syscall.c")
        .warnings(true)
        .compile("cgame_probe_syscall");
}
