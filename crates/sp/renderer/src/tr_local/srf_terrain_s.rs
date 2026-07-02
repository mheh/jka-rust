#![allow(non_camel_case_types, non_snake_case)]

use crate::tr_landscape::ctrland_scape::CTRLandScape;

use super::surface_type_t::surfaceType_t;

/// Raven `srfTerrain_s` (typedef `srfTerrain_t`) — terrain surface.
///
/// Type definition source: `oracle/oracle/code/renderer/tr_local.h:1106-1110`
#[repr(C)]
pub struct srfTerrain_t {
    pub surfaceType: surfaceType_t,
    pub landscape: *mut CTRLandScape,
}

/// Raven manifest tag name; the typedef is `srfTerrain_t`.
pub type srfTerrain_s = srfTerrain_t;

const _: () = assert!(core::mem::size_of::<srfTerrain_t>() == 16);
const _: () = assert!(core::mem::offset_of!(srfTerrain_t, surfaceType) == 0);
const _: () = assert!(core::mem::offset_of!(srfTerrain_t, landscape) == 8);
