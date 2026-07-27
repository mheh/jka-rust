#![allow(non_camel_case_types, non_snake_case)]

use core::ptr::addr_of;
use core::slice;

use mp_qshared::shared::{cplane_t, vec3_t};

use super::surface_type_t::surfaceType_t;

/// `VERTEXSIZE` — non-`_XBOX` build: `6 + (MAXLIGHTMAPS * 3)` = 18 floats per point.
///
/// Source: `oracle/codemp/renderer/tr_local.h:730`
const VERTEXSIZE: usize = 18;

/// Raven `srfSurfaceFace_t` — planar surface (Q3 "face"), variable-sized.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:799-812`
#[repr(C)]
pub struct srfSurfaceFace_t {
    pub surfaceType: surfaceType_t,
    pub plane: cplane_t,
    /// dynamic lighting information
    pub dlightBits: i32,
    /// triangle definitions (no normals at points)
    pub numPoints: i32,
    pub numIndices: i32,
    pub ofsIndices: i32,
    /// variable sized; there is a variable length list of indices here also
    pub points: [[f32; VERTEXSIZE]; 1],
}

// `points` is a flexible array member: the nominal `[[f32; VERTEXSIZE]; 1]`
// bound covers only the first point, and the real trailing arrays (points,
// then indices at `ofsIndices`) are sized by the loader's allocation. The
// walks past that bound are quarantined here (§D11) so the `tr_world.cpp`
// logic port stays entirely safe.
impl srfSurfaceFace_t {
    /// Point `idx`'s leading xyz triple — the `VERTEXSIZE`-float stride walk
    /// over the trailing point array (`VectorCopy(face->points[i], ...)`,
    /// `tr_world.cpp:739-742`).
    ///
    /// # Safety invariant
    /// `idx` must be less than `numPoints` (or, for the wireframe walk, an
    /// index the face's own index array supplies), so the read stays inside
    /// the `Hunk_Alloc`'d trailing array `R_LoadSurfaces` sized from the BSP
    /// lump (`tr_bsp.cpp`); that block lives as long as the world asset.
    ///
    /// This accessor retires with the type at the #41 type pass.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:811`
    pub fn point(&self, idx: usize) -> vec3_t {
        unsafe {
            let base = addr_of!(self.points) as *const f32;
            let p = base.add(idx * VERTEXSIZE);
            [*p, *p.add(1), *p.add(2)]
        }
    }

    /// The face's trailing index array — `numIndices` ints at the byte offset
    /// `ofsIndices` from the face itself (`(int *)((byte *)face +
    /// face->ofsIndices)`, `tr_world.cpp:880`).
    ///
    /// # Safety invariant
    /// [`srfSurfaceFace_t::point`]'s invariant: `ofsIndices`/`numIndices` are
    /// written by the loader to describe the same `Hunk_Alloc`'d block the
    /// face lives in.
    ///
    /// This accessor retires with the type at the #41 type pass.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:810`
    pub fn indices(&self) -> &[i32] {
        unsafe {
            let base = (self as *const srfSurfaceFace_t as *const u8).add(self.ofsIndices as usize)
                as *const i32;
            slice::from_raw_parts(base, self.numIndices as usize)
        }
    }
}

const _: () = assert!(core::mem::size_of::<srfSurfaceFace_t>() == 112);
const _: () = assert!(core::mem::offset_of!(srfSurfaceFace_t, surfaceType) == 0);
const _: () = assert!(core::mem::offset_of!(srfSurfaceFace_t, plane) == 4);
const _: () = assert!(core::mem::offset_of!(srfSurfaceFace_t, dlightBits) == 24);
const _: () = assert!(core::mem::offset_of!(srfSurfaceFace_t, numPoints) == 28);
const _: () = assert!(core::mem::offset_of!(srfSurfaceFace_t, numIndices) == 32);
const _: () = assert!(core::mem::offset_of!(srfSurfaceFace_t, ofsIndices) == 36);
const _: () = assert!(core::mem::offset_of!(srfSurfaceFace_t, points) == 40);
