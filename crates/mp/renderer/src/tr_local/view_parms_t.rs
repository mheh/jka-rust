#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_float;

use mp_qshared::shared::{cplane_t, qboolean, vec3_t};

use super::orientationr_t::orientationr_t;

/// Raven `viewParms_t` — per-view render parameters (orientation, culling
/// frustum, viewport, projection).
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:629-644`
#[repr(C)]
pub struct viewParms_t {
    /// Can't use "or" as it is a reserved word with gcc DREWS 2/2/2002
    pub ori: orientationr_t,
    pub world: orientationr_t,
    /// may be different than or.origin for portals
    pub pvsOrigin: vec3_t,
    /// true if this view is through a portal
    pub isPortal: qboolean,
    /// the portal is a mirror, invert the face culling
    pub isMirror: qboolean,
    /// copied from tr.frameSceneNum
    pub frameSceneNum: i32,
    /// copied from tr.frameCount
    pub frameCount: i32,
    /// clip anything behind this if mirroring
    pub portalPlane: cplane_t,
    pub viewportX: i32,
    pub viewportY: i32,
    pub viewportWidth: i32,
    pub viewportHeight: i32,
    pub fovX: c_float,
    pub fovY: c_float,
    pub projectionMatrix: [c_float; 16],
    pub frustum: [cplane_t; 4],
    pub visBounds: [vec3_t; 2],
    pub zFar: c_float,
}

const _: () = assert!(core::mem::size_of::<viewParms_t>() == 492);
const _: () = assert!(core::mem::offset_of!(viewParms_t, ori) == 0);
const _: () = assert!(core::mem::offset_of!(viewParms_t, world) == 124);
const _: () = assert!(core::mem::offset_of!(viewParms_t, pvsOrigin) == 248);
const _: () = assert!(core::mem::offset_of!(viewParms_t, isPortal) == 260);
const _: () = assert!(core::mem::offset_of!(viewParms_t, isMirror) == 264);
const _: () = assert!(core::mem::offset_of!(viewParms_t, frameSceneNum) == 268);
const _: () = assert!(core::mem::offset_of!(viewParms_t, frameCount) == 272);
const _: () = assert!(core::mem::offset_of!(viewParms_t, portalPlane) == 276);
const _: () = assert!(core::mem::offset_of!(viewParms_t, viewportX) == 296);
const _: () = assert!(core::mem::offset_of!(viewParms_t, viewportY) == 300);
const _: () = assert!(core::mem::offset_of!(viewParms_t, viewportWidth) == 304);
const _: () = assert!(core::mem::offset_of!(viewParms_t, viewportHeight) == 308);
const _: () = assert!(core::mem::offset_of!(viewParms_t, fovX) == 312);
const _: () = assert!(core::mem::offset_of!(viewParms_t, fovY) == 316);
const _: () = assert!(core::mem::offset_of!(viewParms_t, projectionMatrix) == 320);
const _: () = assert!(core::mem::offset_of!(viewParms_t, frustum) == 384);
const _: () = assert!(core::mem::offset_of!(viewParms_t, visBounds) == 464);
const _: () = assert!(core::mem::offset_of!(viewParms_t, zFar) == 488);
