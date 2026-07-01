//! MP `saberTrail_t`.
//!
//! Type definition source: `oracle/oracle/codemp/game/q_shared.h:633-650`

#![allow(non_camel_case_types)]

use core::ffi::c_int;

use crate::shared::{qboolean, vec3_t};

/// Raven `saberTrail_t` — per-blade motion-trail + mark state.
///
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:633-650`
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct saberTrail_t {
    // Actual trail stuff
    pub inAction: c_int, // controls whether should we even consider starting one
    pub duration: c_int, // how long each trail seg stays in existence
    pub lastTime: c_int, // time a saber segement was last stored
    pub base: vec3_t,
    pub tip: vec3_t,

    pub dualbase: vec3_t,
    pub dualtip: vec3_t,

    // Marks stuff
    pub haveOldPos: [qboolean; 2],
    pub oldPos: [vec3_t; 2],
    pub oldNormal: [vec3_t; 2], // store this in case we don't have a connect-the-dots situation
}
const _: () = assert!(core::mem::size_of::<saberTrail_t>() == 116);
