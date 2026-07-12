#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

/// Raven `cArea_t` — flood-fill state for a single collision-model area.
///
/// Type definition source: `oracle/codemp/qcommon/cm_local.h:99-102`
#[repr(C)]
pub struct cArea_t {
    pub floodnum: c_int,
    pub floodvalid: c_int,
}

const _: () = assert!(core::mem::size_of::<cArea_t>() == 8);
const _: () = assert!(core::mem::offset_of!(cArea_t, floodnum) == 0);
const _: () = assert!(core::mem::offset_of!(cArea_t, floodvalid) == 4);
