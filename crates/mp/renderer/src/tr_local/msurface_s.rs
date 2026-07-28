#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use super::shader_s::shader_s;
use super::srf_grid_mesh_s::srfGridMesh_t;
use super::srf_surface_face_t::srfSurfaceFace_t;
use super::srf_triangles_t::srfTriangles_t;
use super::surface_type_t::surfaceType_t;

/// Raven `msurface_s` (typedef `msurface_t`) — a renderable BSP surface.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:872-878`
#[repr(C)]
pub struct msurface_t {
    /// if == tr.viewCount, already added
    pub viewCount: c_int,
    pub shader: *mut shader_s,
    pub fogIndex: c_int,

    /// any of srf*_t
    pub data: *mut surfaceType_t,
}

pub type msurface_s = msurface_t;

/// Borrowed view over the surface kind `msurface_t::data` points at — the
/// tagged-union dispatch Raven writes as `switch(*surf->data)` followed by a
/// cast to the concrete `srf*_t`. Only the kinds the renderer dispatches on
/// have variants; every other tag (and `SF_BAD`) is `Other`, matching the
/// oracle's `default:` arms.
///
/// The **world's** surfaces no longer come through here: DEC-43 gave
/// `WorldAsset::surfaces` an owned `Surface`/`SurfaceData` carrier
/// (`tr_bsp.rs`) and the whole `tr_world.cpp` world walk matches on that.
/// What is left is the inline **brush-model** walk — `RE_GetBModelVerts` and
/// `R_AddBrushModelSurfaces` reach their surfaces through
/// `model_t::bmodel` -> `bmodel_t::firstSurface`, still a raw
/// `*mut msurface_t` because `R_LoadSubmodels`' `model_t` registration is
/// itself unported (`tr_bsp.rs`). This enum and the two accessors below
/// retire when that registration lands and brush models address
/// `WorldAsset::surfaces` by their `BModel` range.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:656-678`
pub enum SurfaceRef<'a> {
    Face(&'a srfSurfaceFace_t),
    Grid(&'a srfGridMesh_t),
    Triangles(&'a srfTriangles_t),
    Other,
}

// `data: *mut surfaceType_t` is Raven's tagged-union pointer: every variant
// leads with the same `surfaceType_t` discriminant, so reading the tag and
// reinterpreting to the concrete struct is the layout contract the loader
// writes. The raw walks are quarantined here (§D11) so the `tr_world.cpp`
// logic port stays entirely safe.
impl msurface_t {
    /// Reads the `surfaceType_t` tag at `data` and borrows the concrete
    /// surface behind it.
    ///
    /// # Safety invariant
    /// `data` is the `Hunk_Alloc`'d surface `R_LoadSurfaces` built for this
    /// `msurface_t` (`tr_bsp.cpp`); it is non-null and stays valid while the
    /// world asset lives, and its leading `surfaceType_t` tag truthfully
    /// describes the struct that follows.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:877`
    pub fn surface_kind(&self) -> SurfaceRef<'_> {
        unsafe {
            match *self.data {
                surfaceType_t::SF_FACE => {
                    SurfaceRef::Face(&*(self.data as *const srfSurfaceFace_t))
                }
                surfaceType_t::SF_GRID => SurfaceRef::Grid(&*(self.data as *const srfGridMesh_t)),
                surfaceType_t::SF_TRIANGLES => {
                    SurfaceRef::Triangles(&*(self.data as *const srfTriangles_t))
                }
                _ => SurfaceRef::Other,
            }
        }
    }

    // The mutable twin `surface_kind_mut`/`SurfaceRefMut` was dropped by
    // DEC-43: its only consumers were `R_DlightSurface`/`R_AddWorldSurface`,
    // which now mutate the owned `SurfaceData`, and the surviving brush-model
    // walks read only (porting-rules §20 — dead surface, and dead `unsafe`).

    /// Unchecked `(srfSurfaceFace_t *)surf->data` cast — Raven's brush-model
    /// walk (`RE_GetBModelVerts`, `tr_world.cpp:672`) casts without testing
    /// the tag, so the tag test stays out of this accessor too (no behavior
    /// invented that the oracle lacks).
    ///
    /// # Safety invariant
    /// [`msurface_t::surface_kind`]'s invariant, plus: the caller must only
    /// use this on surfaces whose tag really is `SF_FACE` — inline brush
    /// models, whose surfaces the loader builds as faces.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:877`
    pub fn face(&self) -> &srfSurfaceFace_t {
        unsafe { &*(self.data as *const srfSurfaceFace_t) }
    }
}

const _: () = assert!(core::mem::offset_of!(msurface_t, viewCount) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<msurface_t>() == 32);
    assert!(core::mem::offset_of!(msurface_t, shader) == 8);
    assert!(core::mem::offset_of!(msurface_t, fogIndex) == 16);
    assert!(core::mem::offset_of!(msurface_t, data) == 24);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<msurface_t>() == 16);
    assert!(core::mem::offset_of!(msurface_t, shader) == 4);
    assert!(core::mem::offset_of!(msurface_t, fogIndex) == 8);
    assert!(core::mem::offset_of!(msurface_t, data) == 12);
};
