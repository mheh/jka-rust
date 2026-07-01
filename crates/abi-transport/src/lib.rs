//! `abi_transport` — cross-mode ABI transport: syscall/vmMain word packing,
//! the `OutboundSysCall`/`InboundVmCall` traits, function-table shapes, raw
//! `dllEntry`/`vmMain`/`GetGameAPI` entrypoint types, and Raven `PASSFLOAT`.
//!
//! No Raven game types cross here — only the wire transport.

pub mod entrypoints;
pub mod generic;

/// Reinterpret an `f32` as an integer-width syscall argument, mirroring Raven's
/// module-local `PASSFLOAT` helper.
///
/// `PASSFLOAT` is transport behavior, not a game-domain API. Raven uses it in
/// module syscall wrappers to carry an `f32` through an integer varargs syscall
/// slot by preserving the float's raw 32-bit representation.
#[inline]
pub fn pass_float(f: f32) -> isize {
    f.to_bits() as i32 as isize
}
