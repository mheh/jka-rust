#![allow(non_camel_case_types, non_snake_case)]

use super::directory_t::directory_t;
use super::pack_t::pack_t;

/// Raven `searchpath_t` — one entry in the search-path chain (a pak or a loose directory).
///
/// Raven: only one of pack / dir will be non NULL.
/// Type definition source: `oracle/code/qcommon/files.h:50-55`
#[repr(C)]
pub struct searchpath_t {
    pub next: *mut searchpath_t,

    pub pack: *mut pack_t,
    pub dir: *mut directory_t,
}

pub type searchpath_s = searchpath_t;

const _: () = assert!(core::mem::size_of::<searchpath_t>() == 24);
const _: () = assert!(core::mem::offset_of!(searchpath_t, next) == 0);
const _: () = assert!(core::mem::offset_of!(searchpath_t, pack) == 8);
const _: () = assert!(core::mem::offset_of!(searchpath_t, dir) == 16);
