//! `mp_engine_select` — the SEAM-D13 binding leaf: the one cfg'd `pub type
//! Engine` alias each MP logic crate imports so its `mod trap` stays non-generic
//! and cfg-free.
//!
//! Cargo feature `static` picks `Static`, default is `CEngine` (`NativeDll`).
//!
//! NOTE (disambiguation, 2026-07-03): this `mp_engine_select::Engine` is the
//! **module-side transport-backend alias** — a *different* type from
//! `mp_engine_core::Engine`, the engine-island aggregate. Opposite islands,
//! never co-scoped (workspace-architecture § canonical disambiguation block).

/// `Static` under Cargo feature `static` (a module linked into our Rust engine).
#[cfg(feature = "static")]
pub type Engine = abi_transport::generic::engine::Static;

/// Default: `CEngine` wrapping the raw syscall pointer (`NativeDll`).
#[cfg(not(feature = "static"))]
pub type Engine = abi_transport::generic::engine::CEngine;
