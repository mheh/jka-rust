//! MP `combatPoint_t`.
//!
//! Type definition source: `oracle/codemp/game/g_local.h:762-773`

#![allow(non_camel_case_types)]

use core::ffi::c_int;

use mp_qshared::shared::{qboolean, vec3_t};

/// Raven `MAX_COMBAT_POINTS`. Source: `oracle/codemp/game/g_local.h:762`
pub const MAX_COMBAT_POINTS: usize = 512;

/// Raven `combatPoint_t`.
///
/// Type definition source: `oracle/codemp/game/g_local.h:764-773`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct combatPoint_t {
    pub origin: vec3_t,
    pub flags: c_int,
    pub occupied: qboolean,
    pub waypoint: c_int,
    pub dangerTime: c_int,
}
const _: () = assert!(core::mem::size_of::<combatPoint_t>() == 28);
