//! ABI definitions for Raven-compatible module/engine boundaries.
//!
//! This module owns the raw call surfaces and their transport rules:
//! `dllEntry`, `vmMain`, `GetGameAPI`, syscall/vmcall tokens, argument
//! packing, pointer word conversion, and Raven-style `PASSFLOAT` float
//! bit transport.
//!
//! `PASSFLOAT` is transport behavior, not a game-domain API. Raven uses it in
//! module syscall wrappers to carry an `f32` through an integer varargs syscall
//! slot by preserving the float's raw 32-bit representation.

/// Reinterpret an `f32` as an integer-width syscall argument, mirroring Raven's
/// module-local `PASSFLOAT` helper.
#[inline]
pub fn pass_float(f: f32) -> isize {
    f.to_bits() as i32 as isize
}

pub mod entrypoints;
pub mod generic;
pub mod mp;
pub mod sp;
