#![allow(non_camel_case_types, non_snake_case)]
use core::ffi::{c_char, c_ulong};

/// Raven `fileInPack_t` — a single hashed file entry inside a loaded pack.
///
/// Type definition source: `oracle/codemp/qcommon/files.h:36-40`
#[repr(C)]
pub struct fileInPack_t {
    /// name of the file
    pub name: *mut c_char,
    /// file info position in zip
    pub pos: c_ulong,
    /// next file in the hash
    pub next: *mut fileInPack_t,
}

/// Raven's C tag name for `fileInPack_t`.
pub type fileInPack_s = fileInPack_t;

const _: () = assert!(core::mem::size_of::<fileInPack_t>() == 24);
const _: () = assert!(core::mem::offset_of!(fileInPack_t, name) == 0);
const _: () = assert!(core::mem::offset_of!(fileInPack_t, pos) == 8);
const _: () = assert!(core::mem::offset_of!(fileInPack_t, next) == 16);
