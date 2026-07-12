#![allow(non_camel_case_types, non_snake_case)]
use super::directory_t::directory_t;
use super::pack_t::pack_t;

/// Raven `searchpath_t` — one entry in the search-path linked list: either a
/// loaded pak (`pack`) or a loose directory (`dir`).
///
/// Type definition source: `oracle/codemp/qcommon/files.h:63-68`
#[repr(C)]
pub struct searchpath_t {
    pub next: *mut searchpath_t,

    /// only one of pack / dir will be non NULL
    pub pack: *mut pack_t,
    pub dir: *mut directory_t,
}

/// Raven's C tag name for `searchpath_t`.
pub type searchpath_s = searchpath_t;

const _: () = assert!(core::mem::offset_of!(searchpath_t, next) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<searchpath_t>() == 24);
    assert!(core::mem::offset_of!(searchpath_t, pack) == 8);
    assert!(core::mem::offset_of!(searchpath_t, dir) == 16);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<searchpath_t>() == 12);
    assert!(core::mem::offset_of!(searchpath_t, pack) == 4);
    assert!(core::mem::offset_of!(searchpath_t, dir) == 8);
};
