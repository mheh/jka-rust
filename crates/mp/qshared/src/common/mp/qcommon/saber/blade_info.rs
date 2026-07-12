//! MP `bladeInfo_t` and `MAX_BLADES`.
//!
//! Type definition source: `oracle/codemp/game/q_shared.h:652-670`

#![allow(non_camel_case_types)]

use core::ffi::c_int;

use crate::shared::{qboolean, vec3_t};

use super::saber_colors::saber_colors_t;
use super::saber_trail::saberTrail_t;

/// Raven `bladeInfo_t` — one blade of a saber.
///
/// Type definition source: `oracle/codemp/game/q_shared.h:652-669`
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct bladeInfo_t {
    pub active: qboolean,
    pub color: saber_colors_t,
    pub radius: f32,
    pub length: f32,
    pub lengthMax: f32,
    pub lengthOld: f32,
    pub desiredLength: f32,
    pub muzzlePoint: vec3_t,
    pub muzzlePointOld: vec3_t,
    pub muzzleDir: vec3_t,
    pub muzzleDirOld: vec3_t,
    pub trail: saberTrail_t,
    pub hitWallDebounceTime: c_int,
    pub storageTime: c_int,
    pub extendDebounce: c_int,
}
const _: () = assert!(core::mem::size_of::<bladeInfo_t>() == 204);

/// Raven `MAX_BLADES`.
///
/// Source: `oracle/codemp/game/q_shared.h:670`
pub const MAX_BLADES: usize = 8;
