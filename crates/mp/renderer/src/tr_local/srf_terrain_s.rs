#![allow(non_camel_case_types, non_snake_case)]

use crate::tr_landscape::ctrland_scape::CTRLandScape;

use super::surface_type_t::surfaceType_t;

/// Raven `srfTerrain_s` (typedef `srfTerrain_t`) — terrain surface.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:744-748`
#[repr(C)]
pub struct srfTerrain_t {
    pub surfaceType: surfaceType_t,
    pub landscape: *mut CTRLandScape,
}

/// Raven manifest tag name; the typedef is `srfTerrain_t`.
pub type srfTerrain_s = srfTerrain_t;

impl srfTerrain_t {
    /// Raven's `((srfTerrain_t *)surface)->landscape` deref — the terrain
    /// surface's `CTRLandScape` (`R_AddTerrainSurfaces`,
    /// `tr_terrain.cpp:998-1004`).
    ///
    /// # Safety invariant
    /// `landscape` is set by the terrain loader to the live
    /// `CTRLandScape` for the current world and stays valid while that world
    /// asset lives; callers first test `landscape.is_null()`, the oracle's own
    /// guard (`tr_terrain.cpp:999`).
    ///
    /// This accessor retires with the type at the #41 type pass.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:744-748`
    pub fn landscape(&self) -> &CTRLandScape {
        debug_assert!(!self.landscape.is_null());
        unsafe { &*self.landscape }
    }
}

const _: () = assert!(core::mem::offset_of!(srfTerrain_t, surfaceType) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<srfTerrain_t>() == 16);
    assert!(core::mem::offset_of!(srfTerrain_t, landscape) == 8);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<srfTerrain_t>() == 8);
    assert!(core::mem::offset_of!(srfTerrain_t, landscape) == 4);
};
