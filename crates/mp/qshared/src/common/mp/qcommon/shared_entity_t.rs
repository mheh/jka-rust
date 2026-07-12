//! MP `sharedEntity_t` copied from Raven `codemp/game/g_public.h`.
//!
//! Type definition source: `oracle/codemp/game/g_public.h:679-715`

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

use super::entity_state::entityState_t;
use super::parms::parms_t;
use super::player_state::playerState_t;
use crate::common::mp::gentity::{MAX_FAILED_NODES, NUM_BSETS, NUM_TIDS};
use crate::shared::{entityShared_t, vec3_t};

/// Raven `sharedEntity_t`.
///
/// Type definition source: `oracle/codemp/game/g_public.h:679-715`
#[repr(C)]
#[derive(Debug)]
pub struct sharedEntity_t {
    /// Communicated by server to clients.
    /// Raven field source: `oracle/codemp/game/g_public.h:680`
    pub s: entityState_t,
    /// Needs to be in the gentity for bg entity access if you want to
    /// actually see the contents I guess you will have to be sure to VMA it
    /// first.
    /// Raven field source: `oracle/codemp/game/g_public.h:681-683`
    pub playerState: *mut playerState_t,
    //TODO: Port Vehicle_t
    // Source: oracle/codemp/game/bg_vehicles.h:477 (used *mut only via g_public.h:684)
    /// Placeholder for `Vehicle_t *m_pVehicle` until `Vehicle_t` is ported.
    /// Vehicle data.
    /// Raven field source: `oracle/codemp/game/g_public.h:684`
    pub m_pVehicle: *mut c_void,
    /// G2 instance.
    /// Raven field source: `oracle/codemp/game/g_public.h:685`
    pub ghoul2: *mut c_void,
    /// Index locally (game/cgame) to anim data for this skel.
    /// Raven field source: `oracle/codemp/game/g_public.h:686`
    pub localAnimIndex: c_int,
    /// Needed for g2 collision.
    /// Raven field source: `oracle/codemp/game/g_public.h:687`
    pub modelScale: vec3_t,
    /// From here up must also be unified with bgEntity/centity.
    ///
    /// Shared by both the server system and game.
    /// Raven field source: `oracle/codemp/game/g_public.h:691`
    pub r: entityShared_t,
    /// Script/ICARUS-related field.
    /// Raven field source: `oracle/codemp/game/g_public.h:694`
    pub taskID: [c_int; NUM_TIDS],
    /// Raven field source: `oracle/codemp/game/g_public.h:695`
    pub parms: *mut parms_t,
    /// Raven field source: `oracle/codemp/game/g_public.h:696`
    pub behaviorSet: [*mut c_char; NUM_BSETS],
    /// Raven field source: `oracle/codemp/game/g_public.h:697`
    pub script_targetname: *mut c_char,
    /// Raven field source: `oracle/codemp/game/g_public.h:698`
    pub delayScriptTime: c_int,
    /// Raven field source: `oracle/codemp/game/g_public.h:699`
    pub fullName: *mut c_char,
    /// rww - targetname and classname are now shared as well. ICARUS needs
    /// access to them.
    /// Raven field source: `oracle/codemp/game/g_public.h:702`
    pub targetname: *mut c_char,
    /// Set in QuakeEd.
    /// Raven field source: `oracle/codemp/game/g_public.h:703`
    pub classname: *mut c_char,
    /// rww - and yet more things to share. This is because the nav code is
    /// in the exe because it's all C++.
    ///
    /// Set once per frame, if you've moved, and if someone asks.
    /// Raven field source: `oracle/codemp/game/g_public.h:706`
    pub waypoint: c_int,
    /// To make sure you don't double-back.
    /// Raven field source: `oracle/codemp/game/g_public.h:707`
    pub lastWaypoint: c_int,
    /// ALWAYS valid - used for tracking someone you lost.
    /// Raven field source: `oracle/codemp/game/g_public.h:708`
    pub lastValidWaypoint: c_int,
    /// Debouncer - so don't keep checking every waypoint in existance every
    /// frame that you can't find one.
    /// Raven field source: `oracle/codemp/game/g_public.h:709`
    pub noWaypointTime: c_int,
    /// Raven field source: `oracle/codemp/game/g_public.h:710`
    pub combatPoint: c_int,
    /// Raven field source: `oracle/codemp/game/g_public.h:711`
    pub failedWaypoints: [c_int; MAX_FAILED_NODES],
    /// Raven field source: `oracle/codemp/game/g_public.h:712`
    pub failedWaypointCheckTime: c_int,
    /// rww - npc's need to know when they're getting roff'd.
    /// Raven field source: `oracle/codemp/game/g_public.h:714`
    pub next_roff_time: c_int,
}

const _: () = assert!(core::mem::size_of::<sharedEntity_t>() == 976);
const _: () = assert!(core::mem::offset_of!(sharedEntity_t, s) == 0);
const _: () = assert!(core::mem::offset_of!(sharedEntity_t, playerState) == 536);
const _: () = assert!(core::mem::offset_of!(sharedEntity_t, m_pVehicle) == 544);
const _: () = assert!(core::mem::offset_of!(sharedEntity_t, ghoul2) == 552);
const _: () = assert!(core::mem::offset_of!(sharedEntity_t, localAnimIndex) == 560);
const _: () = assert!(core::mem::offset_of!(sharedEntity_t, modelScale) == 564);
const _: () = assert!(core::mem::offset_of!(sharedEntity_t, r) == 576);
const _: () = assert!(core::mem::offset_of!(sharedEntity_t, taskID) == 688);
const _: () = assert!(core::mem::offset_of!(sharedEntity_t, parms) == 728);
const _: () = assert!(core::mem::offset_of!(sharedEntity_t, behaviorSet) == 736);
const _: () = assert!(core::mem::offset_of!(sharedEntity_t, script_targetname) == 872);
const _: () = assert!(core::mem::offset_of!(sharedEntity_t, delayScriptTime) == 880);
const _: () = assert!(core::mem::offset_of!(sharedEntity_t, fullName) == 888);
const _: () = assert!(core::mem::offset_of!(sharedEntity_t, targetname) == 896);
const _: () = assert!(core::mem::offset_of!(sharedEntity_t, classname) == 904);
const _: () = assert!(core::mem::offset_of!(sharedEntity_t, waypoint) == 912);
const _: () = assert!(core::mem::offset_of!(sharedEntity_t, lastWaypoint) == 916);
const _: () = assert!(core::mem::offset_of!(sharedEntity_t, lastValidWaypoint) == 920);
const _: () = assert!(core::mem::offset_of!(sharedEntity_t, noWaypointTime) == 924);
const _: () = assert!(core::mem::offset_of!(sharedEntity_t, combatPoint) == 928);
const _: () = assert!(core::mem::offset_of!(sharedEntity_t, failedWaypoints) == 932);
const _: () = assert!(core::mem::offset_of!(sharedEntity_t, failedWaypointCheckTime) == 964);
const _: () = assert!(core::mem::offset_of!(sharedEntity_t, next_roff_time) == 968);
