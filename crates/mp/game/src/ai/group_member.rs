//! MP `AIGroupMember_t`.
//!
//! Type definition source: `oracle/oracle/codemp/game/ai.h:86-93`

#![allow(non_camel_case_types)]

use core::ffi::c_int;

/// Raven `AIGroupMember_t`.
///
/// Raven: `!!!!!!!!!! LOADSAVE-affecting structure !!!!!!!!!!`
/// Type definition source: `oracle/oracle/codemp/game/ai.h:87-93`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AIGroupMember_t {
    pub number: c_int,
    pub waypoint: c_int,
    pub pathCostToEnemy: c_int,
    pub closestBuddy: c_int,
}
const _: () = assert!(core::mem::size_of::<AIGroupMember_t>() == 16);
