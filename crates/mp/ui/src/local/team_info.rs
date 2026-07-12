#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use mp_qshared::shared::qhandle_t;

// Raven's `#define TEAM_MEMBERS 8//5`.
// Source: oracle/codemp/ui/ui_local.h:581
pub const TEAM_MEMBERS: usize = 8;

/// Raven `teamInfo` — per-team UI data (name, image, member list, icons).
///
/// Type definition source: `oracle/codemp/ui/ui_local.h:614-622`
#[repr(C)]
pub struct teamInfo {
    pub teamName: *const c_char,
    pub imageName: *const c_char,
    pub teamMembers: [*const c_char; TEAM_MEMBERS],
    pub teamIcon: qhandle_t,
    pub teamIcon_Metal: qhandle_t,
    pub teamIcon_Name: qhandle_t,
    pub cinematic: i32,
}

const _: () = assert!(core::mem::size_of::<teamInfo>() == 96);
const _: () = assert!(core::mem::offset_of!(teamInfo, teamName) == 0);
const _: () = assert!(core::mem::offset_of!(teamInfo, imageName) == 8);
const _: () = assert!(core::mem::offset_of!(teamInfo, teamMembers) == 16);
const _: () = assert!(core::mem::offset_of!(teamInfo, teamIcon) == 80);
const _: () = assert!(core::mem::offset_of!(teamInfo, teamIcon_Metal) == 84);
const _: () = assert!(core::mem::offset_of!(teamInfo, teamIcon_Name) == 88);
const _: () = assert!(core::mem::offset_of!(teamInfo, cinematic) == 92);
