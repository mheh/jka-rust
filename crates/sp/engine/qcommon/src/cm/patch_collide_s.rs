#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::shared::vec3_t;

use super::facet_t::facet_t;
use super::patch_plane_t::patchPlane_t;

/// Raven `patchCollide_t` — a patch mesh's collision representation: bounding
/// box plus its planes and facets.
///
/// Type definition source: `oracle/code/qcommon/cm_patch.h:93-99`
#[repr(C)]
pub struct patchCollide_s {
    pub bounds: [vec3_t; 2],
    /// surface planes plus edge planes
    pub numPlanes: i32,
    pub planes: *mut patchPlane_t,
    pub numFacets: i32,
    pub facets: *mut facet_t,
}

pub type patchCollide_t = patchCollide_s;

const _: () = assert!(core::mem::size_of::<patchCollide_t>() == 56);
const _: () = assert!(core::mem::offset_of!(patchCollide_t, bounds) == 0);
const _: () = assert!(core::mem::offset_of!(patchCollide_t, numPlanes) == 24);
const _: () = assert!(core::mem::offset_of!(patchCollide_t, planes) == 32);
const _: () = assert!(core::mem::offset_of!(patchCollide_t, numFacets) == 40);
const _: () = assert!(core::mem::offset_of!(patchCollide_t, facets) == 48);
