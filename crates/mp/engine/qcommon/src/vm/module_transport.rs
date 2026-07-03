//! `ModuleTransport` — the engine-side runtime per-module transport (SEAM-D1).

/// Chosen per loaded module at runtime (DEC-05) so one session can mix
/// transports. A field of `ModuleRegistry` (engine-seam § Engine-side
/// dispatchers). NativeDll through the C syscall pointer, `Static` linked into
/// our Rust engine, `Wasm` through wasm imports.
///
/// Source: `docs/architecture/engine-seam.md` § Engine-side runtime transport (SEAM-D1).
pub enum ModuleTransport {
    NativeDll,
    Static,
    Wasm,
}
