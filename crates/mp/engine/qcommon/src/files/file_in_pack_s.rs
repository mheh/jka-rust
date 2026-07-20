#![allow(non_camel_case_types, non_snake_case)]
use core::ffi::c_ulong;

/// Raven `fileInPack_t` — a single hashed file entry inside a loaded pack.
///
/// Engine-internal only (lives in `pack_t.buildBuffer`, never crosses the
/// module ABI), so the C layout is dropped (string-data migration, DEC-32):
/// the name is owned and the intrusive hash chain is an index into the owning
/// `pack_t.buildBuffer`.
/// Type definition source: `oracle/codemp/qcommon/files.h:36-40`
pub struct fileInPack_t {
    /// name of the file
    pub name: String,
    /// file info position in zip
    pub pos: c_ulong,
    /// next file in the hash (`buildBuffer` index; `None` = Raven's null)
    pub next: Option<u32>,
}
