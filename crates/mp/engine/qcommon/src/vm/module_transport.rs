//! `ModuleTransport` — the engine-side runtime per-module transport (SEAM-D1).

/// The jka-rust engine-side transport layer chosen per loaded module at runtime
/// (DEC-05) so one session can mix transports. A field of `ModuleRegistry`
/// (engine-seam § Engine-side dispatchers). `NativeDll` runs through the C
/// syscall pointer, `Static` is linked into our Rust engine.
///
/// Source: `docs/architecture/engine-seam.md` § Engine-side runtime transport (SEAM-D1).
pub enum ModuleTransport {
    NativeDll,
    Static,
}
