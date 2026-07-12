#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use sp_qshared::shared::vec3_t;

/// Raven `MAX_REFNAME`. Source: `oracle/code/game/g_local.h:568`
pub const MAX_REFNAME: usize = 32;

/// Raven `reference_tag_s` — a named navigation/reference point.
///
/// Type definition source: `oracle/code/game/g_local.h:573-580`
#[repr(C)]
pub struct reference_tag_t {
    pub name: [c_char; MAX_REFNAME],
    pub origin: vec3_t,
    pub angles: vec3_t,
    /// Just in case
    pub flags: i32,
    /// For nav goals
    pub radius: i32,
}

const _: () = assert!(core::mem::size_of::<reference_tag_t>() == 64);
const _: () = assert!(core::mem::offset_of!(reference_tag_t, name) == 0);
const _: () = assert!(core::mem::offset_of!(reference_tag_t, origin) == 32);
const _: () = assert!(core::mem::offset_of!(reference_tag_t, angles) == 44);
const _: () = assert!(core::mem::offset_of!(reference_tag_t, flags) == 56);
const _: () = assert!(core::mem::offset_of!(reference_tag_t, radius) == 60);
