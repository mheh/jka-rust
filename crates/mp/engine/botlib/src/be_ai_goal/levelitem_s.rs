#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::vec3_t;

/// Raven `levelitem_t` — a runtime item instance in the level (doubly linked).
///
/// Type definition source: `oracle/codemp/botlib/be_ai_goal.cpp:93-105`
#[repr(C)]
pub struct levelitem_t {
    /// number of the level item
    pub number: i32,
    /// index into the item info
    pub iteminfo: i32,
    /// item flags
    pub flags: i32,
    /// fixed roam weight
    pub weight: f32,
    /// origin of the item
    pub origin: vec3_t,
    /// area the item is in
    pub goalareanum: i32,
    /// goal origin within the area
    pub goalorigin: vec3_t,
    /// entity number
    pub entitynum: i32,
    /// item is removed after this time
    pub timeout: f32,
    pub prev: *mut levelitem_t,
    pub next: *mut levelitem_t,
}

pub type levelitem_s = levelitem_t;

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<levelitem_t>() == 72);
    assert!(core::mem::offset_of!(levelitem_t, number) == 0);
    assert!(core::mem::offset_of!(levelitem_t, iteminfo) == 4);
    assert!(core::mem::offset_of!(levelitem_t, flags) == 8);
    assert!(core::mem::offset_of!(levelitem_t, weight) == 12);
    assert!(core::mem::offset_of!(levelitem_t, origin) == 16);
    assert!(core::mem::offset_of!(levelitem_t, goalareanum) == 28);
    assert!(core::mem::offset_of!(levelitem_t, goalorigin) == 32);
    assert!(core::mem::offset_of!(levelitem_t, entitynum) == 44);
    assert!(core::mem::offset_of!(levelitem_t, timeout) == 48);
    assert!(core::mem::offset_of!(levelitem_t, prev) == 56);
    assert!(core::mem::offset_of!(levelitem_t, next) == 64);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<levelitem_t>() == 60);
    assert!(core::mem::offset_of!(levelitem_t, number) == 0);
    assert!(core::mem::offset_of!(levelitem_t, iteminfo) == 4);
    assert!(core::mem::offset_of!(levelitem_t, flags) == 8);
    assert!(core::mem::offset_of!(levelitem_t, weight) == 12);
    assert!(core::mem::offset_of!(levelitem_t, origin) == 16);
    assert!(core::mem::offset_of!(levelitem_t, goalareanum) == 28);
    assert!(core::mem::offset_of!(levelitem_t, goalorigin) == 32);
    assert!(core::mem::offset_of!(levelitem_t, entitynum) == 44);
    assert!(core::mem::offset_of!(levelitem_t, timeout) == 48);
    assert!(core::mem::offset_of!(levelitem_t, prev) == 52);
    assert!(core::mem::offset_of!(levelitem_t, next) == 56);
};
