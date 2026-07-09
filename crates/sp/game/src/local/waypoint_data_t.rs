#![allow(non_camel_case_types, non_snake_case)]

//! SP `g_local.h` waypoint node data.
//!
//! Type definition source: `oracle/code/game/g_local.h:146-154`

use core::ffi::c_char;

use sp_qshared::shared::MAX_QPATH;

/// Raven `waypointData_t`.
///
/// Type definition source: `oracle/code/game/g_local.h:146-154`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct waypointData_t {
    pub targetname: [c_char; MAX_QPATH],
    pub target: [c_char; MAX_QPATH],
    pub target2: [c_char; MAX_QPATH],
    pub target3: [c_char; MAX_QPATH],
    pub target4: [c_char; MAX_QPATH],
    pub nodeID: i32,
}

const _: () = assert!(core::mem::size_of::<waypointData_t>() == 324);
const _: () = assert!(core::mem::offset_of!(waypointData_t, targetname) == 0);
const _: () = assert!(core::mem::offset_of!(waypointData_t, target) == 64);
const _: () = assert!(core::mem::offset_of!(waypointData_t, target2) == 128);
const _: () = assert!(core::mem::offset_of!(waypointData_t, target3) == 192);
const _: () = assert!(core::mem::offset_of!(waypointData_t, target4) == 256);
const _: () = assert!(core::mem::offset_of!(waypointData_t, nodeID) == 320);
