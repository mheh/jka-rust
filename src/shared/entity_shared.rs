//! Raven `entityShared_t` shared server/entity linkage state.
//!
//! Source: `oracle/oracle/codemp/game/g_public.h:60-95`

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use crate::shared::{qboolean, vec3_t};

/// Raven `entityShared_t`.
///
/// Type definition source: `oracle/oracle/codemp/game/g_public.h:60-95`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct entityShared_t {
    pub linked: qboolean,
    pub linkcount: c_int,
    pub svFlags: c_int,
    pub singleClient: c_int,
    pub bmodel: qboolean,
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub contents: c_int,
    pub absmin: vec3_t,
    pub absmax: vec3_t,
    pub currentOrigin: vec3_t,
    pub currentAngles: vec3_t,
    /// Set to qtrue when the entity is being roffed.
    pub mIsRoffing: qboolean,
    pub ownerNum: c_int,
    /// First 32 clients are index 0, latter 32 clients are index 1.
    pub broadcastClients: [c_int; 2],
}
