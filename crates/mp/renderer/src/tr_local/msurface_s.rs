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
/// Both this enum and the accessors below retire with `msurface_t` at the
/// #41 type pass, when the world owns a real surface arena.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:656-678`
pub enum SurfaceRef<'a> {
    Face(&'a srfSurfaceFace_t),
    Grid(&'a srfGridMesh_t),
    Triangles(&'a srfTriangles_t),
    Other,
}

/// Mutable twin of [`SurfaceRef`] — the oracle's dispatch arms mutate the
/// concrete surface (`R_DlightFace`/`R_DlightGrid`/`R_DlightTrisurf` each
/// stash the surviving `dlightBits` on it).
pub enum SurfaceRefMut<'a> {
    Face(&'a mut srfSurfaceFace_t),
    Grid(&'a mut srfGridMesh_t),
    Triangles(&'a mut srfTriangles_t),
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

    /// Mutable twin of [`msurface_t::surface_kind`] — same safety invariant.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:877`
    pub fn surface_kind_mut(&mut self) -> SurfaceRefMut<'_> {
        unsafe {
            match *self.data {
                surfaceType_t::SF_FACE => {
                    SurfaceRefMut::Face(&mut *(self.data as *mut srfSurfaceFace_t))
                }
                surfaceType_t::SF_GRID => {
                    SurfaceRefMut::Grid(&mut *(self.data as *mut srfGridMesh_t))
                }
                surfaceType_t::SF_TRIANGLES => {
                    SurfaceRefMut::Triangles(&mut *(self.data as *mut srfTriangles_t))
                }
                _ => SurfaceRefMut::Other,
            }
        }
    }

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
