//! SP `saberTrail_t`.
//!
//! Type definition source: `oracle/code/game/q_shared.h:1616-1630`

#![allow(non_camel_case_types)]

use core::ffi::c_int;

use crate::shared::{qboolean, vec3_t};

/// Raven SP `saberTrail_t` — per-blade motion-trail + mark state.
///
/// Raven: `!!!!!!! loadsave affecting struct !!!!!!!`.
/// Diverges from MP: SP has **no** `dualbase`/`dualtip` (MP-only dual-blade trail).
/// Type definition source: `oracle/code/game/q_shared.h:1616-1630`
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct saberTrail_t {
    pub inAction: c_int,
    pub duration: c_int,
    pub lastTime: c_int,
    pub base: vec3_t,
    pub tip: vec3_t,

    // Marks stuff
    pub haveOldPos: [qboolean; 2],
    pub oldPos: [vec3_t; 2],
    pub oldNormal: [vec3_t; 2],
}
const _: () = assert!(core::mem::size_of::<saberTrail_t>() == 92);
