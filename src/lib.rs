//! jka-rust — an idiomatic, opinionated, per-module Rust reimplementation of the
//! *Jedi Academy* MP game/engine.
//!
//! INDEPENDENT of the faithful 1:1 port: it does not depend on the port's C-mirror
//! layout. The port (`jedi-academy-rust`) is carried under `oracle/` purely as a
//! behavioral reference for differential parity testing.
//!
//! Modules are filed by true sharing scope (`shared` < `bg` < {`game`, `engine`}), bridged by the `boundary` seam; see README.md.

pub mod bg;
pub mod common;
pub mod game;
pub mod modules;
pub mod shared;

// `boundary/` (the typed seam) and `engine/` (its backend) not declared yet — pending type-sourcing.
