#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use mp_qshared::common::mp::cgame::refdef_t::{
    MAX_MAP_AREA_BYTES, MAX_RENDER_STRINGS, MAX_RENDER_STRING_LENGTH,
};
use mp_qshared::shared::{qboolean, vec3_t};

use super::dlight_s::dlight_t;
use super::draw_surf_s::drawSurf_t;
use super::srf_poly_s::srfPoly_t;
use super::tr_mini_ref_entity_t::trMiniRefEntity_t;
use super::tr_ref_entity_t::trRefEntity_t;

/// Raven `trRefdef_t`.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:563-598`
#[repr(C)]
pub struct trRefdef_t {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub fov_x: f32,
    pub fov_y: f32,
    pub vieworg: vec3_t,
    /// transformation matrix
    pub viewaxis: [vec3_t; 3],

    /// time in milliseconds for shader effects and other time dependent rendering issues
    pub time: i32,
    pub frametime: i32,
    /// RDF_NOWORLDMODEL, etc
    pub rdflags: i32,

    /// 1 bits will prevent the associated area from rendering at all
    pub areamask: [u8; MAX_MAP_AREA_BYTES],
    /// qtrue if areamask changed since last scene
    pub areamaskModified: qboolean,

    /// tr.refdef.time / 1000.0
    pub floatTime: f32,

    /// text messages for deform text shaders
    pub text: [[c_char; MAX_RENDER_STRING_LENGTH]; MAX_RENDER_STRINGS],

    pub num_entities: i32,
    pub entities: *mut trRefEntity_t,
    pub miniEntities: *mut trMiniRefEntity_t,

    pub num_dlights: i32,
    pub dlights: *mut dlight_t,

    pub numPolys: i32,
    pub polys: *mut srfPoly_t,

    pub numDrawSurfs: i32,
    pub drawSurfs: *mut drawSurf_t,
}

const _: () = assert!(core::mem::offset_of!(trRefdef_t, x) == 0);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, y) == 4);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, width) == 8);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, height) == 12);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, fov_x) == 16);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, fov_y) == 20);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, vieworg) == 24);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, viewaxis) == 36);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, time) == 72);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, frametime) == 76);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, rdflags) == 80);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, areamask) == 84);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, areamaskModified) == 116);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, floatTime) == 120);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, text) == 124);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, num_entities) == 380);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<trRefdef_t>() == 448);
    assert!(core::mem::offset_of!(trRefdef_t, entities) == 384);
    assert!(core::mem::offset_of!(trRefdef_t, miniEntities) == 392);
    assert!(core::mem::offset_of!(trRefdef_t, num_dlights) == 400);
    assert!(core::mem::offset_of!(trRefdef_t, dlights) == 408);
    assert!(core::mem::offset_of!(trRefdef_t, numPolys) == 416);
    assert!(core::mem::offset_of!(trRefdef_t, polys) == 424);
    assert!(core::mem::offset_of!(trRefdef_t, numDrawSurfs) == 432);
    assert!(core::mem::offset_of!(trRefdef_t, drawSurfs) == 440);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<trRefdef_t>() == 416);
    assert!(core::mem::offset_of!(trRefdef_t, entities) == 384);
    assert!(core::mem::offset_of!(trRefdef_t, miniEntities) == 388);
    assert!(core::mem::offset_of!(trRefdef_t, num_dlights) == 392);
    assert!(core::mem::offset_of!(trRefdef_t, dlights) == 396);
    assert!(core::mem::offset_of!(trRefdef_t, numPolys) == 400);
    assert!(core::mem::offset_of!(trRefdef_t, polys) == 404);
    assert!(core::mem::offset_of!(trRefdef_t, numDrawSurfs) == 408);
    assert!(core::mem::offset_of!(trRefdef_t, drawSurfs) == 412);
};
