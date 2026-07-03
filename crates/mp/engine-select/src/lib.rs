//! `mp_engine_select` — the SEAM-D13 binding leaf: the one cfg'd `pub type
//! Engine` alias each MP logic crate imports so its `mod trap` stays non-generic
//! and cfg-free.
//!
//! `cfg(target_arch = "wasm32")` picks the wasm backend, Cargo feature `static`
//! picks `Static`, default is `CEngine` (`NativeDll`).
//!
//! NOTE (disambiguation, 2026-07-03): this `mp_engine_select::Engine` is the
//! **module-side transport-backend alias** — a *different* type from
//! `mp_engine_core::Engine`, the engine-island aggregate. Opposite islands,
//! never co-scoped (workspace-architecture § canonical disambiguation block).

/// The per-build outbound-transport backend a logic crate threads as
/// `engine: &Engine` into its `mod trap` wrappers (SEAM-D13). `Engine:
/// Execute<C>` holds for every outbound call `C`.
#[cfg(target_arch = "wasm32")]
//TODO: Port wasm32 outbound backend type (SEAM-Q11 — concrete type/file open)
// Source: docs/architecture/engine-seam.md § Call-site conventions (SEAM-Q11)
pub type Engine = abi_transport::generic::engine::Static;

/// `Static` under Cargo feature `static` (a module linked into our Rust engine).
#[cfg(all(not(target_arch = "wasm32"), feature = "static"))]
pub type Engine = abi_transport::generic::engine::Static;

/// Default: `CEngine` wrapping the raw syscall pointer (`NativeDll`).
#[cfg(all(not(target_arch = "wasm32"), not(feature = "static")))]
pub type Engine = abi_transport::generic::engine::CEngine;
