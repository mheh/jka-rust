#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use mp_qshared::shared::{qboolean, vec3_t};

use crate::collision_world::CollisionWorld;

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
    // leafList_t is engine-INTERNAL (never crosses the module ABI), so the store
    // callback is retyped to a receiver-carrying Rust fn ptr: `CM_StoreLeafs`/
    // `CM_StoreBrushes` thread `&mut CollisionWorld` (they read `cm.cmg`) per
    // ruling 2 state threading. Same size/align as the C fn ptr; offset asserts hold.
    pub storeLeafs: Option<fn(&mut CollisionWorld, *mut leafList_s, c_int)>,
}

pub type leafList_t = leafList_s;

const _: () = assert!(core::mem::offset_of!(leafList_t, count) == 0);
const _: () = assert!(core::mem::offset_of!(leafList_t, maxcount) == 4);
const _: () = assert!(core::mem::offset_of!(leafList_t, overflowed) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<leafList_t>() == 64);
    assert!(core::mem::offset_of!(leafList_t, list) == 16);
    assert!(core::mem::offset_of!(leafList_t, bounds) == 24);
    assert!(core::mem::offset_of!(leafList_t, lastLeaf) == 48);
    assert!(core::mem::offset_of!(leafList_t, storeLeafs) == 56);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<leafList_t>() == 48);
    assert!(core::mem::offset_of!(leafList_t, list) == 12);
    assert!(core::mem::offset_of!(leafList_t, bounds) == 16);
    assert!(core::mem::offset_of!(leafList_t, lastLeaf) == 40);
    assert!(core::mem::offset_of!(leafList_t, storeLeafs) == 44);
};
