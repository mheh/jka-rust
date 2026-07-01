//! MP `waypointData_t`.
//!
//! Type definition source: `oracle/oracle/codemp/game/g_local.h:810-818`

#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int};

use mp_qshared::shared::MAX_QPATH;

/// Raven `waypointData_t` — cleared as each map is entered.
///
/// Type definition source: `oracle/oracle/codemp/game/g_local.h:810-818`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct waypointData_t {
    pub targetname: [c_char; MAX_QPATH],
    pub target: [c_char; MAX_QPATH],
    pub target2: [c_char; MAX_QPATH],
    pub target3: [c_char; MAX_QPATH],
    pub target4: [c_char; MAX_QPATH],
    pub nodeID: c_int,
}
const _: () = assert!(core::mem::size_of::<waypointData_t>() == 324);
