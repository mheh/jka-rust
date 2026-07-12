//! SP `AIGroupInfo_t`.
//!
//! Type definition source: `oracle/code/game/ai.h:106-125`

#![allow(non_camel_case_types)]

use core::ffi::c_int;

use sp_qshared::common::sp::gentity::gentity_t;
use sp_qshared::shared::{qboolean, vec3_t};

use super::consts::{MAX_GROUP_MEMBERS, NUM_SQUAD_STATES};
use super::group_member::AIGroupMember_t;
use crate::teams::team_t;

/// Raven SP `AIGroupInfo_t` — squad/group AI shared state.
///
/// Raven: `!!!!!!!!!! LOADSAVE-affecting structure !!!!!!!!!!`
/// Byte-identical layout to MP (SP is the origin); differs only in that `team`
/// is SP's faction `team_t`. Pointer-bearing => arch-dependent; the assert pins
/// the host-64-bit size.
/// Type definition source: `oracle/code/game/ai.h:106-125`
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
