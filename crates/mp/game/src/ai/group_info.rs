//! MP `AIGroupInfo_t`.
//!
//! Type definition source: `oracle/codemp/game/ai.h:96-116`

#![allow(non_camel_case_types)]

use core::ffi::c_int;

use mp_bg::team_t;
use mp_qshared::common::mp::gentity::gentity_t;
use mp_qshared::shared::{qboolean, vec3_t};

use super::consts::{MAX_GROUP_MEMBERS, NUM_SQUAD_STATES};
use super::group_member::AIGroupMember_t;

/// Raven `AIGroupInfo_t` — squad/group AI shared state.
///
/// Raven: `!!!!!!!!!! LOADSAVE-affecting structure !!!!!!!!!!`
/// Pointer-bearing (`gentity_t *`) => arch-dependent layout; the `size_of`
/// assert pins the host-64-bit size.
/// Type definition source: `oracle/codemp/game/ai.h:97-116`
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AIGroupInfo_t {
    pub numGroup: c_int,
    pub processed: qboolean,
    pub team: team_t,
    pub enemy: *mut gentity_t,
    pub enemyWP: c_int,
    pub speechDebounceTime: c_int,
    pub lastClearShotTime: c_int,
    pub lastSeenEnemyTime: c_int,
    pub morale: c_int,
    pub moraleAdjust: c_int,
    pub moraleDebounce: c_int,
    pub memberValidateTime: c_int,
    pub activeMemberNum: c_int,
    pub commander: *mut gentity_t,
    pub enemyLastSeenPos: vec3_t,
    pub numState: [c_int; NUM_SQUAD_STATES],
    pub member: [AIGroupMember_t; MAX_GROUP_MEMBERS],
}
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<AIGroupInfo_t>() == 624);
