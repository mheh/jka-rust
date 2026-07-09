#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::shared::{qboolean, vec3_t};

/// Raven `leafList_t` — accumulates BSP leaf indices touched by a box/sphere
/// traversal, overflowing into a callback when too many to store individually.
///
/// Type definition source: `oracle/code/qcommon/cm_local.h:265-273`
#[repr(C)]
pub struct leafList_s {
    pub count: i32,
    pub maxcount: i32,
    pub overflowed: qboolean,
    pub list: *mut i32,
    pub bounds: [vec3_t; 2],
    /// for overflows where each leaf can't be stored individually
    pub lastLeaf: i32,
    pub storeLeafs: Option<unsafe extern "C" fn(ll: *mut leafList_s, nodenum: i32)>,
}

pub type leafList_t = leafList_s;

const _: () = assert!(core::mem::size_of::<leafList_t>() == 64);
const _: () = assert!(core::mem::offset_of!(leafList_t, count) == 0);
const _: () = assert!(core::mem::offset_of!(leafList_t, maxcount) == 4);
const _: () = assert!(core::mem::offset_of!(leafList_t, overflowed) == 8);
const _: () = assert!(core::mem::offset_of!(leafList_t, list) == 16);
const _: () = assert!(core::mem::offset_of!(leafList_t, bounds) == 24);
const _: () = assert!(core::mem::offset_of!(leafList_t, lastLeaf) == 48);
const _: () = assert!(core::mem::offset_of!(leafList_t, storeLeafs) == 56);
