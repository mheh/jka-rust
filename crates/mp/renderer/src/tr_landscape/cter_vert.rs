#![allow(non_camel_case_types, non_snake_case)]
use core::ffi::c_int;

use mp_qshared::common::mp::cgame::color4ub_t::color4ub_t;
use mp_qshared::shared::vec3_t;

/// Raven `CTerVert` — a single landscape terxel's render data (position, lighting,
/// texture coords, and tess-array bookkeeping).
///
/// Type definition source: `oracle/oracle/codemp/renderer/tr_landscape.h:22-35`
#[repr(C)]
pub struct CTerVert {
    /// real world coords of terxel
    pub coords: vec3_t,
    /// required to calculate lighting and used in physics
    pub normal: vec3_t,
    /// tint at this terxel
    pub tint: color4ub_t,
    /// texture coordinates at this terxel
    pub tex: [f32; 2],
    /// Copy of heightmap data
    pub height: c_int,
    /// Index of the vert in the tess array
    pub tessIndex: c_int,
    /// ...... for the tess with this registration
    pub tessRegistration: c_int,
}

const _: () = assert!(core::mem::size_of::<CTerVert>() == 48);
const _: () = assert!(core::mem::offset_of!(CTerVert, coords) == 0);
const _: () = assert!(core::mem::offset_of!(CTerVert, normal) == 12);
const _: () = assert!(core::mem::offset_of!(CTerVert, tint) == 24);
const _: () = assert!(core::mem::offset_of!(CTerVert, tex) == 28);
const _: () = assert!(core::mem::offset_of!(CTerVert, height) == 36);
const _: () = assert!(core::mem::offset_of!(CTerVert, tessIndex) == 40);
const _: () = assert!(core::mem::offset_of!(CTerVert, tessRegistration) == 44);
