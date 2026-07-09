#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_float, c_int};

use sp_qshared::shared::vec3_t;

/// Raven `modelDef_s` (typedef `modelDef_t`) — a 3D model preview definition
/// used by UI item defs.
///
/// Type definition source: `oracle/code/ui/ui_shared.h:350-365`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct modelDef_s {
    pub angle: c_int,
    pub origin: vec3_t,
    pub fov_x: c_float,
    pub fov_y: c_float,
    pub rotationSpeed: c_int,

    /// required
    pub g2mins: vec3_t,
    /// required
    pub g2maxs: vec3_t,
    /// optional
    pub g2skin: c_int,
    /// optional
    pub g2anim: c_int,
    // JLF MPMOVED
    // Transition extras
    pub g2mins2: vec3_t,
    pub g2maxs2: vec3_t,
    pub g2minsEffect: vec3_t,
    pub g2maxsEffect: vec3_t,
    pub fov_x2: c_float,
    pub fov_y2: c_float,
    pub fov_Effectx: c_float,
    pub fov_Effecty: c_float,
}

const _: () = assert!(core::mem::size_of::<modelDef_s>() == 124);
const _: () = assert!(core::mem::offset_of!(modelDef_s, angle) == 0);
const _: () = assert!(core::mem::offset_of!(modelDef_s, origin) == 4);
const _: () = assert!(core::mem::offset_of!(modelDef_s, fov_x) == 16);
const _: () = assert!(core::mem::offset_of!(modelDef_s, fov_y) == 20);
const _: () = assert!(core::mem::offset_of!(modelDef_s, rotationSpeed) == 24);
const _: () = assert!(core::mem::offset_of!(modelDef_s, g2mins) == 28);
const _: () = assert!(core::mem::offset_of!(modelDef_s, g2maxs) == 40);
const _: () = assert!(core::mem::offset_of!(modelDef_s, g2skin) == 52);
const _: () = assert!(core::mem::offset_of!(modelDef_s, g2anim) == 56);
const _: () = assert!(core::mem::offset_of!(modelDef_s, g2mins2) == 60);
const _: () = assert!(core::mem::offset_of!(modelDef_s, g2maxs2) == 72);
const _: () = assert!(core::mem::offset_of!(modelDef_s, g2minsEffect) == 84);
const _: () = assert!(core::mem::offset_of!(modelDef_s, g2maxsEffect) == 96);
const _: () = assert!(core::mem::offset_of!(modelDef_s, fov_x2) == 108);
const _: () = assert!(core::mem::offset_of!(modelDef_s, fov_y2) == 112);
const _: () = assert!(core::mem::offset_of!(modelDef_s, fov_Effectx) == 116);
const _: () = assert!(core::mem::offset_of!(modelDef_s, fov_Effecty) == 120);
