//! MP `sharedRagDollUpdateParams_t` copied from Raven `codemp/game/q_shared.h`.
//!
//! Source: `oracle/oracle/codemp/game/q_shared.h:924-933`

#![allow(non_camel_case_types)]

use core::ffi::c_int;

use crate::shared::vec3_t;

/// Raven `sharedRagDollUpdateParams_t`.
///
/// Raven comment: `And one for updating during model animation.`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct sharedRagDollUpdateParams_t {
    pub angles: vec3_t,
    pub position: vec3_t,
    pub scale: vec3_t,
    pub velocity: vec3_t,
    pub me: c_int,
    pub settle_frame: c_int,
}
