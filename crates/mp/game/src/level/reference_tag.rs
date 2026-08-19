//! MP `reference_tag_t`.
//!
//! Type definition source: `oracle/codemp/game/g_local.h:1234-1248`

#![allow(non_camel_case_types)]

use core::ffi::c_int;

use mp_qshared::shared::{qboolean, vec3_t};

/// Raven `MAX_REFNAME`. Source: `oracle/codemp/game/g_local.h:1234`
pub const MAX_REFNAME: usize = 32;

pub const RTF_NONE: c_int = 0;
pub const RTF_NAVGOAL: c_int = 0x00000001;

/// Raven `reference_tag_t`.
///
/// Type definition source: `oracle/codemp/game/g_local.h:1241-1248`
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct reference_tag_t {
    pub name: [core::ffi::c_char; MAX_REFNAME],
    pub origin: vec3_t,
    pub angles: vec3_t,
    pub flags: c_int,  //Just in case
    pub radius: c_int, //For nav goals
    pub inuse: qboolean,
}
const _: () = assert!(core::mem::size_of::<reference_tag_t>() == 68);
