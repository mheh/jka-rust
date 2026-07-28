//! MP renderer export-table types (`tr_public.h`).
//!
//! `refexport_t` was deleted at R2 (DEC-37): the renderer export table
//! dissolves at the Rust boundary rather than staying a `#[repr(C)]` vtable
//! struct — see `crates/mp/renderer/src/tr_init.rs`'s `GetRefAPI` DEFERRED
//! note.
//!
//! What the header still owns for this crate are its flag masks: `ref_flags`
//! is the canonical home for the `RF_*`/`RDF_*` bit constants.

pub mod ref_flags;
