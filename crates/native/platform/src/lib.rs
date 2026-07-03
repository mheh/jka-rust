//! `native_platform` — Raven-free OS/platform primitives (cross-mode).
#![allow(non_camel_case_types)]

pub mod entrypoints;
pub mod mem;
pub mod module_loader;
pub mod platform;

pub use mem::zeroed_box;
