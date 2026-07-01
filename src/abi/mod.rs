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

// Migration bridge: transport now lives in `abi_transport`; the MP/SP ABI
// surfaces now live in the `mp_abi` / `sp_abi` crates.
pub use abi_transport::{entrypoints, generic, pass_float};
