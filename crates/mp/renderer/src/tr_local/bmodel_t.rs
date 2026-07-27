#![allow(non_camel_case_types, non_snake_case)]
use core::ffi::c_int;
use core::slice;

use mp_qshared::shared::vec3_t;

use super::msurface_s::msurface_t;

/// Raven `bmodel_t` — an inline (brush) model's bounds and surface range.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:938-942`
#[repr(C)]
pub struct bmodel_t {
    // for culling
    pub bounds: [vec3_t; 2],
    pub firstSurface: *mut msurface_t,
    pub numSurfaces: c_int,
}

impl bmodel_t {
    /// Raven's `bmodel->firstSurface[i]` walk over the model's `numSurfaces`
    /// surfaces (`RE_GetBModelVerts`, `tr_world.cpp:665-687`). The raw walk is
    /// quarantined here (§D11) so the `tr_world.cpp` logic port stays entirely
    /// safe.
    ///
    /// # Safety invariant
    /// `firstSurface`/`numSurfaces` are written by `R_LoadSubmodels`
    /// (`tr_bsp.cpp`) to point into the world's `Hunk_Alloc`'d `msurface_t`
    /// array; they stay valid while the world asset lives.
    ///
    /// This accessor retires with the type at the #41 type pass.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:940-941`
    pub fn surfaces(&self) -> &[msurface_t] {
        unsafe { slice::from_raw_parts(self.firstSurface, self.numSurfaces as usize) }
    }
}

const _: () = assert!(core::mem::offset_of!(bmodel_t, bounds) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<bmodel_t>() == 40);
    assert!(core::mem::offset_of!(bmodel_t, firstSurface) == 24);
    assert!(core::mem::offset_of!(bmodel_t, numSurfaces) == 32);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<bmodel_t>() == 32);
    assert!(core::mem::offset_of!(bmodel_t, firstSurface) == 24);
    assert!(core::mem::offset_of!(bmodel_t, numSurfaces) == 28);
};
