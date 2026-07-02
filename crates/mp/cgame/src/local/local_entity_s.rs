#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::shared::{qboolean, trajectory_t, vec3_t};

use super::le_bounce_sound_type_t::leBounceSoundType_t;
use super::le_mark_type_t::leMarkType_t;
use super::le_type_t::leType_t;

/// Anonymous struct for `localEntity_s::data::sprite` (no Raven name — anonymous in the header).
///
/// Type definition source: `oracle/oracle/codemp/cgame/cg_local.h:552-557`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct localEntity_t_sprite {
    pub radius: f32,
    pub dradius: f32,
    pub startRGB: vec3_t,
    pub dRGB: vec3_t,
}

/// Anonymous struct for `localEntity_s::data::trail` (no Raven name — anonymous in the header).
///
/// Type definition source: `oracle/oracle/codemp/cgame/cg_local.h:558-565`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct localEntity_t_trail {
    pub width: f32,
    pub dwidth: f32,
    pub length: f32,
    pub dlength: f32,
    pub startRGB: vec3_t,
    pub dRGB: vec3_t,
}

/// Anonymous struct for `localEntity_s::data::line` (no Raven name — anonymous in the header).
///
/// Raven: below are bezier specific; `control1`/`control2` are the initial position of
/// control points, `*_velocity` their initial velocity, `*_acceleration` their constant
/// acceleration.
/// Type definition source: `oracle/oracle/codemp/cgame/cg_local.h:566-576`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct localEntity_t_line {
    pub width: f32,
    pub dwidth: f32,
    pub control1: vec3_t,
    pub control2: vec3_t,
    pub control1_velocity: vec3_t,
    pub control2_velocity: vec3_t,
    pub control1_acceleration: vec3_t,
    pub control2_acceleration: vec3_t,
}

/// Anonymous struct for `localEntity_s::data::line2` (no Raven name — anonymous in the header).
///
/// Type definition source: `oracle/oracle/codemp/cgame/cg_local.h:577-584`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct localEntity_t_line2 {
    pub width: f32,
    pub dwidth: f32,
    pub width2: f32,
    pub dwidth2: f32,
    pub startRGB: vec3_t,
    pub dRGB: vec3_t,
}

/// Anonymous struct for `localEntity_s::data::cylinder` (no Raven name — anonymous in the header).
///
/// Type definition source: `oracle/oracle/codemp/cgame/cg_local.h:585-592`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct localEntity_t_cylinder {
    pub width: f32,
    pub dwidth: f32,
    pub width2: f32,
    pub dwidth2: f32,
    pub height: f32,
    pub dheight: f32,
}

/// Anonymous struct for `localEntity_s::data::electricity` (no Raven name — anonymous in the header).
///
/// Type definition source: `oracle/oracle/codemp/cgame/cg_local.h:593-596`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct localEntity_t_electricity {
    pub width: f32,
    pub dwidth: f32,
}

/// Anonymous struct for `localEntity_s::data::particle` (no Raven name — anonymous in the header).
///
/// Raven: `dir`'s magnitude is 1, but this is oldpos - newpos right before the particle is
/// sent to the renderer. May want to add something like particle::localEntity_s *le (for the
/// particle's think fn).
/// Type definition source: `oracle/oracle/codemp/cgame/cg_local.h:597-606`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct localEntity_t_particle {
    pub radius: f32,
    pub dradius: f32,
    pub thinkFn: Option<unsafe extern "C" fn(le: *mut localEntity_t) -> qboolean>,
    pub dir: vec3_t,
}

/// Anonymous struct for `localEntity_s::data::spawner` (no Raven name — anonymous in the header).
///
/// Type definition source: `oracle/oracle/codemp/cgame/cg_local.h:607-617`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct localEntity_t_spawner {
    pub dontDie: qboolean,
    pub dir: vec3_t,
    pub variance: f32,
    pub delay: c_int,
    pub nextthink: c_int,
    pub thinkFn: Option<unsafe extern "C" fn(le: *mut localEntity_t) -> qboolean>,
    pub data1: c_int,
    pub data2: c_int,
}

/// Anonymous struct for `localEntity_s::data::fragment` (no Raven name — anonymous in the header).
///
/// Type definition source: `oracle/oracle/codemp/cgame/cg_local.h:618-621`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct localEntity_t_fragment {
    pub radius: f32,
}

/// Anonymous union for `localEntity_s::data` (no Raven name — anonymous in the header).
///
/// Type definition source: `oracle/oracle/codemp/cgame/cg_local.h:551-622`
#[repr(C)]
#[derive(Clone, Copy)]
pub union localEntity_t_data {
    pub sprite: localEntity_t_sprite,
    pub trail: localEntity_t_trail,
    pub line: localEntity_t_line,
    pub line2: localEntity_t_line2,
    pub cylinder: localEntity_t_cylinder,
    pub electricity: localEntity_t_electricity,
    pub particle: localEntity_t_particle,
    pub spawner: localEntity_t_spawner,
    pub fragment: localEntity_t_fragment,
}

/// Raven `localEntity_t` — client-side-only entity for effects (sparks, blood, smoke, etc).
///
/// Raven: `lifeRate` is `1.0 / (endTime - startTime)`; `bounceFactor` is 0.0 = no bounce,
/// 1.0 = perfect; `bounceSound` is an optional sound index to play upon bounce; `leMarkType`
/// is the mark to leave on fragment impact.
/// Type definition source: `oracle/oracle/codemp/cgame/cg_local.h:519-625`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct localEntity_t {
    pub prev: *mut localEntity_t,
    pub next: *mut localEntity_t,
    pub leType: leType_t,
    pub leFlags: c_int,

    pub startTime: c_int,
    pub endTime: c_int,
    pub fadeInTime: c_int,

    pub lifeRate: f32,

    pub pos: trajectory_t,
    pub angles: trajectory_t,

    pub bounceFactor: f32,
    pub bounceSound: c_int,

    pub alpha: f32,
    pub dalpha: f32,

    pub forceAlpha: c_int,

    pub color: [f32; 4],

    pub radius: f32,

    pub light: f32,
    pub lightColor: vec3_t,

    pub leMarkType: leMarkType_t,
    pub leBounceSoundType: leBounceSoundType_t,

    pub data: localEntity_t_data,

    pub refEntity: refEntity_t,
}

const _: () = assert!(core::mem::size_of::<localEntity_t>() == 472);
const _: () = assert!(core::mem::offset_of!(localEntity_t, prev) == 0);
const _: () = assert!(core::mem::offset_of!(localEntity_t, next) == 8);
const _: () = assert!(core::mem::offset_of!(localEntity_t, leType) == 16);
const _: () = assert!(core::mem::offset_of!(localEntity_t, leFlags) == 20);
const _: () = assert!(core::mem::offset_of!(localEntity_t, startTime) == 24);
const _: () = assert!(core::mem::offset_of!(localEntity_t, endTime) == 28);
const _: () = assert!(core::mem::offset_of!(localEntity_t, fadeInTime) == 32);
const _: () = assert!(core::mem::offset_of!(localEntity_t, lifeRate) == 36);
const _: () = assert!(core::mem::offset_of!(localEntity_t, pos) == 40);
const _: () = assert!(core::mem::offset_of!(localEntity_t, angles) == 76);
const _: () = assert!(core::mem::offset_of!(localEntity_t, bounceFactor) == 112);
const _: () = assert!(core::mem::offset_of!(localEntity_t, bounceSound) == 116);
const _: () = assert!(core::mem::offset_of!(localEntity_t, alpha) == 120);
const _: () = assert!(core::mem::offset_of!(localEntity_t, dalpha) == 124);
const _: () = assert!(core::mem::offset_of!(localEntity_t, forceAlpha) == 128);
const _: () = assert!(core::mem::offset_of!(localEntity_t, color) == 132);
const _: () = assert!(core::mem::offset_of!(localEntity_t, radius) == 148);
const _: () = assert!(core::mem::offset_of!(localEntity_t, light) == 152);
const _: () = assert!(core::mem::offset_of!(localEntity_t, lightColor) == 156);
const _: () = assert!(core::mem::offset_of!(localEntity_t, leMarkType) == 168);
const _: () = assert!(core::mem::offset_of!(localEntity_t, leBounceSoundType) == 172);
const _: () = assert!(core::mem::offset_of!(localEntity_t, data) == 176);
const _: () = assert!(core::mem::offset_of!(localEntity_t, refEntity) == 256);
