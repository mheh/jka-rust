#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use mp_qshared::shared::vec3_t;

/// `MAX_STRINGFIELD`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp` (`l_struct.h` field width).
pub const MAX_STRINGFIELD: usize = 80;

/// Raven `iteminfo_t` — configuration info for one item class.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_goal.cpp:107-119`
#[repr(C)]
pub struct iteminfo_t {
    /// classname of the item
    pub classname: [c_char; 32],
    /// name of the item
    pub name: [c_char; MAX_STRINGFIELD],
    /// model of the item
    pub model: [c_char; MAX_STRINGFIELD],
    /// model index
    pub modelindex: i32,
    /// item type
    pub r#type: i32,
    /// index in the inventory
    pub index: i32,
    /// respawn time
    pub respawntime: f32,
    /// mins of the item
    pub mins: vec3_t,
    /// maxs of the item
    pub maxs: vec3_t,
    /// number of the item info
    pub number: i32,
}

pub type iteminfo_s = iteminfo_t;

const _: () = assert!(core::mem::size_of::<iteminfo_t>() == 236);
const _: () = assert!(core::mem::offset_of!(iteminfo_t, classname) == 0);
const _: () = assert!(core::mem::offset_of!(iteminfo_t, name) == 32);
const _: () = assert!(core::mem::offset_of!(iteminfo_t, model) == 112);
const _: () = assert!(core::mem::offset_of!(iteminfo_t, modelindex) == 192);
const _: () = assert!(core::mem::offset_of!(iteminfo_t, r#type) == 196);
const _: () = assert!(core::mem::offset_of!(iteminfo_t, index) == 200);
const _: () = assert!(core::mem::offset_of!(iteminfo_t, respawntime) == 204);
const _: () = assert!(core::mem::offset_of!(iteminfo_t, mins) == 208);
const _: () = assert!(core::mem::offset_of!(iteminfo_t, maxs) == 220);
const _: () = assert!(core::mem::offset_of!(iteminfo_t, number) == 232);
