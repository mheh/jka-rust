#![allow(non_camel_case_types, non_snake_case)]
use core::ffi::{c_float, c_int, c_void};
use core::slice;

use mp_qshared::shared::qhandle_t;

use crate::tr_local::shader_s::shader_t;

use super::cter_vert::CTerVert;
use super::ctrheight_details::CTRHeightDetails;
use super::ctrpatch::CTRPatch;
use super::spatch_info::TPatchInfo;

// Raven's `#if _DEBUG` `mCycleCount` field is not present in a release build; the
// asserted layout below matches the non-debug shape.

// Raven `#define HEIGHT_RESOLUTION 256`.
// Source: oracle/codemp/qcommon/cm_landscape.h:13
const HEIGHT_RESOLUTION: usize = 256;

/// Raven `CTRLandScape` — the renderer-side landscape instance: patch storage,
/// sort order, terrain shaders, and per-height detail shaders.
///
/// Type definition source: `oracle/codemp/renderer/tr_landscape.h:119-186`
#[repr(C)]
pub struct CTRLandScape {
    //TODO: Port CCMLandScape
    // Source: oracle/codemp/qcommon/cm_landscape.h:135
    pub common: *const c_void,
    /// Local patch info
    pub mTRPatches: *mut CTRPatch,
    pub mSortedPatches: *mut TPatchInfo,

    pub mPatchMinx: c_int,
    pub mPatchMaxx: c_int,
    pub mPatchMiny: c_int,
    pub mPatchMaxy: c_int,
    /// terxels * terxels = exit condition for splitting
    pub mMaxNode: c_int,
    pub mSortedCount: c_int,

    pub mPatchSize: c_float,

    /// shader the terrain got its contents from
    pub mShader: *mut shader_t,

    /// modulation value and texture coords per vertex
    pub mRenderMap: *mut CTerVert,
    /// Scale of texture mapped to terrain
    pub mTextureScale: c_float,

    pub mScalarSize: c_float,

    /// Water shader
    pub mWaterShader: *mut shader_t,
    /// Flat ground shader
    pub mFlatShader: qhandle_t,

    /// Array of info specific to height
    pub mHeightDetails: [CTRHeightDetails; HEIGHT_RESOLUTION],
}

// The two `Z_Malloc`'d arrays this class owns (`mRenderMap`, `mTRPatches`) are
// ABI-layout raw pointers. Their raw walks are quarantined here (§D11) so the
// `tr_terrain.cpp` logic port stays entirely safe.
impl CTRLandScape {
    /// Raven's `mRenderMap` array — one `CTerVert` per heightmap sample.
    ///
    /// # Safety invariant
    /// `mRenderMap` is the `Z_Malloc(sizeof(CTerVert) * common->GetRealArea())`
    /// block the `CTRLandScape(const char *)` ctor allocates
    /// (`tr_terrain.cpp:899`) and `~CTRLandScape` frees (`:866-870`); it stays
    /// live for the whole landscape's lifetime. `len` must be at most that
    /// landscape's `CCMLandScape::GetRealArea()` — callers pass
    /// `CmLandScape::real_area()`, the value the allocation was sized from.
    ///
    /// Source: `oracle/codemp/renderer/tr_landscape.h:135`
    pub fn render_map(&self, len: usize) -> &[CTerVert] {
        unsafe { slice::from_raw_parts(self.mRenderMap, len) }
    }

    /// Mutable twin of [`CTRLandScape::render_map`] — same safety invariant.
    ///
    /// Source: `oracle/codemp/renderer/tr_landscape.h:135`
    pub fn render_map_mut(&mut self, len: usize) -> &mut [CTerVert] {
        unsafe { slice::from_raw_parts_mut(self.mRenderMap, len) }
    }

    /// Raven `CTRLandScape::GetPatch` — `mTRPatches + (blockWidth * y) + x`.
    /// Raven reads `blockWidth` through the `common` back-pointer; it is
    /// threaded in here (§B4).
    ///
    /// # Safety invariant
    /// `mTRPatches` is the `Z_Malloc(sizeof(CTRPatch) * common->GetBlockCount())`
    /// block the ctor allocates (`tr_terrain.cpp:918`) and `~CTRLandScape`
    /// frees (`:856-860`). `x` must be in `[0, blockWidth)` and `y` in
    /// `[0, GetBlockHeight())`, so the index stays inside `GetBlockCount()`.
    ///
    /// Source: `oracle/codemp/renderer/tr_landscape.h:168`
    pub fn patch_mut(&mut self, x: c_int, y: c_int, blockWidth: c_int) -> &mut CTRPatch {
        unsafe { &mut *self.mTRPatches.add(((blockWidth * y) + x) as usize) }
    }

    /// Raven `CTRPatch::SetRenderMap` (`tr_terrain.cpp:806-809`) fused with the
    /// `CTRLandScape::GetRenderMap` accessor it calls through its `localowner`
    /// back-pointer — `mRenderMap + x + (y * common->GetRealWidth())`. Owning
    /// the pair here keeps `CTRPatch::mRenderMap`, an ABI-layout `CTerVert *`,
    /// the only raw pointer the terrain logic port ever produces, and keeps
    /// producing it inside this quarantine (§D11). Raven's two back-pointers
    /// are threaded in as the patch's block coordinates plus `blockWidth`/
    /// `realWidth` (§B4).
    ///
    /// # Safety invariant
    /// `patchX`/`patchY`/`blockWidth` carry [`CTRLandScape::patch_mut`]'s
    /// invariant. The offset itself is a `wrapping_add`, so forming the
    /// pointer is safe; every later *read* through `CTRPatch::mRenderMap`
    /// requires `x + (y * realWidth)` to land inside the
    /// `Z_Malloc(sizeof(CTerVert) * common->GetRealArea())` block described on
    /// [`CTRLandScape::render_map`], which holds for the heightmap coordinates
    /// `InitRendererPatches` passes (`x < realWidth`, `y < GetRealHeight()`)
    /// with `realWidth` the same `CCMLandScape::GetRealWidth()` the allocation
    /// was sized from.
    ///
    /// Source: `oracle/codemp/renderer/tr_landscape.h:167`;
    /// `oracle/codemp/renderer/tr_terrain.cpp:806-809`
    pub fn set_patch_render_map(
        &mut self,
        patchX: c_int,
        patchY: c_int,
        blockWidth: c_int,
        x: c_int,
        y: c_int,
        realWidth: c_int,
    ) {
        let render_map = self.mRenderMap;
        let patch = self.patch_mut(patchX, patchY, blockWidth);
        patch.mRenderMap = render_map.wrapping_add((x + (y * realWidth)) as usize);
    }
}

const _: () = assert!(core::mem::offset_of!(CTRLandScape, common) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<CTRLandScape>() == 1120);
    assert!(core::mem::offset_of!(CTRLandScape, mTRPatches) == 8);
    assert!(core::mem::offset_of!(CTRLandScape, mSortedPatches) == 16);
    assert!(core::mem::offset_of!(CTRLandScape, mPatchMinx) == 24);
    assert!(core::mem::offset_of!(CTRLandScape, mPatchMaxx) == 28);
    assert!(core::mem::offset_of!(CTRLandScape, mPatchMiny) == 32);
    assert!(core::mem::offset_of!(CTRLandScape, mPatchMaxy) == 36);
    assert!(core::mem::offset_of!(CTRLandScape, mMaxNode) == 40);
    assert!(core::mem::offset_of!(CTRLandScape, mSortedCount) == 44);
    assert!(core::mem::offset_of!(CTRLandScape, mPatchSize) == 48);
    assert!(core::mem::offset_of!(CTRLandScape, mShader) == 56);
    assert!(core::mem::offset_of!(CTRLandScape, mRenderMap) == 64);
    assert!(core::mem::offset_of!(CTRLandScape, mTextureScale) == 72);
    assert!(core::mem::offset_of!(CTRLandScape, mScalarSize) == 76);
    assert!(core::mem::offset_of!(CTRLandScape, mWaterShader) == 80);
    assert!(core::mem::offset_of!(CTRLandScape, mFlatShader) == 88);
    assert!(core::mem::offset_of!(CTRLandScape, mHeightDetails) == 92);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<CTRLandScape>() == 1088);
    assert!(core::mem::offset_of!(CTRLandScape, mTRPatches) == 4);
    assert!(core::mem::offset_of!(CTRLandScape, mSortedPatches) == 8);
    assert!(core::mem::offset_of!(CTRLandScape, mPatchMinx) == 12);
    assert!(core::mem::offset_of!(CTRLandScape, mPatchMaxx) == 16);
    assert!(core::mem::offset_of!(CTRLandScape, mPatchMiny) == 20);
    assert!(core::mem::offset_of!(CTRLandScape, mPatchMaxy) == 24);
    assert!(core::mem::offset_of!(CTRLandScape, mMaxNode) == 28);
    assert!(core::mem::offset_of!(CTRLandScape, mSortedCount) == 32);
    assert!(core::mem::offset_of!(CTRLandScape, mPatchSize) == 36);
    assert!(core::mem::offset_of!(CTRLandScape, mShader) == 40);
    assert!(core::mem::offset_of!(CTRLandScape, mRenderMap) == 44);
    assert!(core::mem::offset_of!(CTRLandScape, mTextureScale) == 48);
    assert!(core::mem::offset_of!(CTRLandScape, mScalarSize) == 52);
    assert!(core::mem::offset_of!(CTRLandScape, mWaterShader) == 56);
    assert!(core::mem::offset_of!(CTRLandScape, mFlatShader) == 60);
    assert!(core::mem::offset_of!(CTRLandScape, mHeightDetails) == 64);
};
