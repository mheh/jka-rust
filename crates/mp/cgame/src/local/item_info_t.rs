#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_void;

use mp_bg::public::g_item::MAX_ITEM_MODELS;
use mp_qshared::shared::qboolean;
use mp_qshared::shared::qhandle_t;

/// Raven `itemInfo_t` — cgame-side per-item model/icon cache entry.
///
/// Ghoul2 Insert Start/End marks the `g2Models`/`radius` fields.
/// Type definition source: `oracle/codemp/cgame/cg_local.h:708-720`
#[repr(C)]
pub struct itemInfo_t {
    pub registered: qboolean,
    pub models: [qhandle_t; MAX_ITEM_MODELS],
    pub icon: qhandle_t,
    // Ghoul2 Insert Start
    pub g2Models: [*mut c_void; MAX_ITEM_MODELS],
    pub radius: [f32; MAX_ITEM_MODELS],
    // Ghoul2 Insert End
}

// The head of the struct holds only 4-byte members, so these offsets hold on both pointer widths.
const _: () = assert!(core::mem::offset_of!(itemInfo_t, registered) == 0);
const _: () = assert!(core::mem::offset_of!(itemInfo_t, models) == 4);
const _: () = assert!(core::mem::offset_of!(itemInfo_t, icon) == 20);
// `g2Models` is an array of pointers, so the size and the tail from `g2Models` onward go in the width-gated blocks.
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<itemInfo_t>() == 72);
    assert!(core::mem::offset_of!(itemInfo_t, g2Models) == 24);
    assert!(core::mem::offset_of!(itemInfo_t, radius) == 56);
};
// ILP32 twin: clang i386 ground truth, where msvc and linux-gnu agree.
// These numbers are the retail 32-bit module ABI.
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<itemInfo_t>() == 56);
    assert!(core::mem::offset_of!(itemInfo_t, g2Models) == 24);
    assert!(core::mem::offset_of!(itemInfo_t, radius) == 40);
};
