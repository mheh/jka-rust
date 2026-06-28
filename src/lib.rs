//! jka-rust — an idiomatic, opinionated, per-module Rust reimplementation of the
//! *Jedi Academy* MP game/engine.
//!
//! INDEPENDENT of the faithful 1:1 port: it does not depend on the port's C-mirror
//! layout. The port (`jedi-academy-rust`) is carried under `oracle/` purely as a
//! behavioral reference for differential parity testing.
//!
//! Status: scaffold. The ported boundary definitions live under `src/state/`
//! (imported from the port prototype) and are NOT yet wired in — they reference the
//! port's types and must be re-sourced against this crate's own layout first.
//! See README.md.

// modules intentionally not declared yet — see src/state/ (pending type-sourcing).
