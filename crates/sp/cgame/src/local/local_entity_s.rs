#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use sp_qshared::common::sp::renderer::ref_entity_t::refEntity_t;
use sp_qshared::shared::{trajectory_t, vec3_t};

use super::le_bounce_sound_t::leBounceSound_t;
use super::le_type_t::leType_t;

/// Raven `localEntity_t` — client side temporary entity, not communicated to server.
///
/// Type definition source: `oracle/code/cgame/cg_local.h:222-248`
#[repr(C)]
pub struct localEntity_t {
    pub prev: *mut localEntity_t,
    pub next: *mut localEntity_t,
    pub leType: leType_t,
    pub leFlags: c_int,

    pub startTime: c_int,
    pub endTime: c_int,

    /// 1.0 / (endTime - startTime)
    pub lifeRate: f32,

    pub pos: trajectory_t,
    pub angles: trajectory_t,

    /// 0.0 = no bounce, 1.0 = perfect
    pub bounceFactor: f32,

    pub color: [f32; 4],

    pub radius: f32,

    pub light: f32,
    pub lightColor: vec3_t,

    pub leBounceSoundType: leBounceSound_t,

    pub refEntity: refEntity_t,
    pub ownerGentNum: c_int,
}

const _: () = assert!(core::mem::size_of::<localEntity_t>() == 336);
const _: () = assert!(core::mem::offset_of!(localEntity_t, prev) == 0);
const _: () = assert!(core::mem::offset_of!(localEntity_t, next) == 8);
const _: () = assert!(core::mem::offset_of!(localEntity_t, leType) == 16);
const _: () = assert!(core::mem::offset_of!(localEntity_t, leFlags) == 20);
const _: () = assert!(core::mem::offset_of!(localEntity_t, startTime) == 24);
const _: () = assert!(core::mem::offset_of!(localEntity_t, endTime) == 28);
const _: () = assert!(core::mem::offset_of!(localEntity_t, lifeRate) == 32);
const _: () = assert!(core::mem::offset_of!(localEntity_t, pos) == 36);
const _: () = assert!(core::mem::offset_of!(localEntity_t, angles) == 72);
const _: () = assert!(core::mem::offset_of!(localEntity_t, bounceFactor) == 108);
const _: () = assert!(core::mem::offset_of!(localEntity_t, color) == 112);
const _: () = assert!(core::mem::offset_of!(localEntity_t, radius) == 128);
const _: () = assert!(core::mem::offset_of!(localEntity_t, light) == 132);
const _: () = assert!(core::mem::offset_of!(localEntity_t, lightColor) == 136);
const _: () = assert!(core::mem::offset_of!(localEntity_t, leBounceSoundType) == 148);
const _: () = assert!(core::mem::offset_of!(localEntity_t, refEntity) == 152);
const _: () = assert!(core::mem::offset_of!(localEntity_t, ownerGentNum) == 328);
