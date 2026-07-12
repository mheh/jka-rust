#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use mp_qshared::shared::vec3_t;

use super::facet_t::facet_t;
use super::patch_plane_t::patchPlane_t;

/// Raven `patchCollide_s` — collision representation of a curved-surface patch.
///
/// Type definition source: `oracle/codemp/qcommon/cm_patch.h:93-99`
#[repr(C)]
pub struct patchCollide_s {
    pub bounds: [vec3_t; 2],
    /// surface planes plus edge planes
    pub numPlanes: c_int,
    pub planes: *mut patchPlane_t,
    pub numFacets: c_int,
    pub facets: *mut facet_t,
}

pub type patchCollide_t = patchCollide_s;

const _: () = assert!(core::mem::offset_of!(patchCollide_t, bounds) == 0);
const _: () = assert!(core::mem::offset_of!(patchCollide_t, numPlanes) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<patchCollide_t>() == 56);
    assert!(core::mem::offset_of!(patchCollide_t, planes) == 32);
    assert!(core::mem::offset_of!(patchCollide_t, numFacets) == 40);
    assert!(core::mem::offset_of!(patchCollide_t, facets) == 48);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<patchCollide_t>() == 40);
    assert!(core::mem::offset_of!(patchCollide_t, planes) == 28);
    assert!(core::mem::offset_of!(patchCollide_t, numFacets) == 32);
    assert!(core::mem::offset_of!(patchCollide_t, facets) == 36);
};
