#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use mp_qshared::shared::{qboolean, vec3_t};

/// Raven `leafList_s` — accumulator used while walking the BSP tree collecting leafs.
///
/// Type definition source: `oracle/codemp/qcommon/cm_local.h:266-274`
#[repr(C)]
pub struct leafList_s {
    pub count: c_int,
    pub maxcount: c_int,
    pub overflowed: qboolean,
    pub list: *mut c_int,
    pub bounds: [vec3_t; 2],
    /// for overflows where each leaf can't be stored individually
    pub lastLeaf: c_int,
    pub storeLeafs: Option<unsafe extern "C" fn(ll: *mut leafList_s, nodenum: c_int)>,
}

pub type leafList_t = leafList_s;

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<leafList_t>() == 64);
const _: () = assert!(core::mem::offset_of!(leafList_t, count) == 0);
const _: () = assert!(core::mem::offset_of!(leafList_t, maxcount) == 4);
const _: () = assert!(core::mem::offset_of!(leafList_t, overflowed) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(leafList_t, list) == 16);
const _: () = assert!(core::mem::offset_of!(leafList_t, bounds) == 24);
const _: () = assert!(core::mem::offset_of!(leafList_t, lastLeaf) == 48);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(leafList_t, storeLeafs) == 56);
