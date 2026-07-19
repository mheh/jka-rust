//! `G2_ConstructGhoulSkeleton` — the skeleton-(re)build entry point and its
//! bone-accessor siblings (`docs/subsystems/ghoul2-server.md` roster,
//! `render/skeleton.rs`, class "G2_ConstructGhoulSkeleton"): "`
//! G2_ConstructGhoulSkeleton`, `G2_TransformGhoulBones`, `G2_GetBoneMatrixLow`,
//! `G2_GetBoneBasepose`, `G2_RagGetBoneBasePoseMatrixLow`,
//! `worldMatrix`/`worldMatrixInv` scratch threading."
//!
//! Per the doc's `## Slice hooks` host-service map, this file is
//! **host-consuming**: `G2_ConstructGhoulSkeleton`/`G2_TransformGhoulBones`
//! read loader model memory (`model_mdxm`/`model_mdxa`, ruling 36) to build or
//! refresh a model's `CBoneCache` (`render/bone_cache.rs`, `G2SV-D9`); the pure
//! bone-read accessors (`G2_GetBoneMatrixLow`/`G2_GetBoneBasepose`/
//! `G2_RagGetBoneBasePoseMatrixLow`) do **not** take `host` — like Raven, they
//! only walk the cache's already-resolved `header` pointer (fetched via
//! `EngineHost::model_mdxa` once, at `CBoneCache::new`/refresh time), never
//! re-deriving it (`G2SV-D5`).
//!
//! `worldMatrix` (`tr_ghoul2.cpp:136`) is a file-scope global in Raven, read
//! only by `G2_GetBoneMatrixLow` (`:767`) in this file's cited range; per
//! porting-rules §B3/§B4 (no hidden globals, state threaded not reached) it
//! becomes an explicit `world_matrix: &mdxaBone_t` parameter on
//! `g2_get_bone_matrix_low` alone — no other function in this file reads it.
//! Its producer, `G2_GenerateWorldMatrix`, lives in `misc.rs` (method
//! transcription table); threading its output down to this read site is each
//! caller's responsibility, not this file's.
//!
//! **Doc/oracle gaps found while enumerating this class (reported upstream,
//! not fixed here — porting-rules §17/CLAUDE.md "private helpers included"):**
//! `G2_ConstructGhoulSkeleton`'s body (`tr_ghoul2.cpp:3570-3624`) directly
//! calls two `static` (TU-private) helpers neither the roster summary line
//! above nor the Method transcription table names anywhere in the doc:
//! `G2_Sort_Models` (`:2881-2950`) and `RootMatrix` (`:3333-3366`, which
//! itself recurses into `G2_ConstructGhoulSkeleton` when a model has the
//! `GHOUL2_NEWORIGIN` flag, `:3345`). Both are `static` — TU-private to
//! `tr_ghoul2.cpp`, i.e. this file's own oracle source — and both are on the
//! live server path (`G2_Sort_Models` unconditionally; `RootMatrix` whenever
//! `G2_ConstructGhoulSkeleton`'s `check_for_new_origin` is `true`), so they
//! are stubbed here as this class's private helpers rather than left
//! unstubbed, matching the same-file precedent `render/bone_cache.rs` set for
//! its un-rostered `Root()`. A third callee, `G2_GetBoltMatrixLow`
//! (`:3253-3331`, called directly from `G2_ConstructGhoulSkeleton` at `:3608`
//! and from `RootMatrix` at `:3346`), is **not** `static` — it is a
//! cross-TU-shared function (also called from the ragdoll solver,
//! `G2_bones.cpp:3423`, and the `G2API_GetBoltMatrix` chain,
//! `G2_API.cpp:1839`) with **no roster row or landed file anywhere** in this
//! doc/codebase as of this writing. Because it directly blocks
//! `G2_ConstructGhoulSkeleton`/`RootMatrix` here and no other home claims it,
//! it is stubbed in this file too (as `pub fn`, crate-visible for the future
//! `ragdoll.rs`/`api_bolts.rs` porters to call) — the doc should assign it a
//! permanent home; this is a transcription stopgap, not that decision.
//! `G2_GetBoltMatrixLow`'s own further private helpers
//! (`G2_ProcessSurfaceBolt2`, `G2_FindSurface_BC`) are ported below as this
//! function's private helpers: surface-attached bolts are a server-reachable
//! path (`G2_ConstructGhoulSkeleton`/`RootMatrix` drive them), so the surface
//! arm computes the real bolt matrix off the hit triangle's verts through the
//! bone cache rather than a no-op.
//!
//! **Aliasing finding (reported upstream under `problems`, same class as
//! `api_bolts.rs`'s own module-doc gap #3).** `g2_get_bolt_matrix_low`'s and
//! `g2_transform_ghoul_bones`'s pinned shapes each take `g2: &mut
//! Ghoul2System` *and* a `&`/`&mut CGhoul2Info` — but every real caller in
//! this file (`g2_construct_ghoul_skeleton`, `root_matrix`) needs to look that
//! `CGhoul2Info` up *out of* the very same `Ghoul2System` (`g2.info_array`),
//! which is an unsatisfiable simultaneous borrow (E0502), not a call-site
//! mistake. `resolve_bolt_matrix_low`/`g2_transform_ghoul_bones_inner` below
//! are alias-free cores — taking only the specific `Ghoul2System` fields each
//! needs (`bone_caches`) instead of the whole struct — that both the pinned
//! public entry points and this file's own internal callers route through, so
//! no borrow conflict and no behavior change.

use core::ffi::c_void;

use mp_host_interface::EngineHost;
use mp_qshared::shared::q_math::{
    _DotProduct, _VectorMA, _VectorSubtract, CrossProduct, DotProductRow, VectorNormalize2,
};
use mp_qshared::shared::{mdxaBone_t, vec3_t, VectorNormalize, VectorNormalizeRow, MAX_QPATH};

use crate::ghoul2_system::{BoneCacheArena, BoneCacheId, Ghoul2System};
use crate::render::bone_cache::CBoneCache;
use crate::render::bone_transform;
use crate::shared::bolt_info_t::boltInfo_t;
use crate::shared::bone_info_t::boneInfo_t;
use crate::shared::cghoul2_info::CGhoul2Info;
use crate::shared::cghoul2_info_v::CGhoul2Info_v;
use crate::shared::surface_info_t::surfaceInfo_t;

// ---------------------------------------------------------------------------
// `mModelBoltLink` bit-packing constants (`G2.h:30-40`). Duplicated locally
// per this crate's own per-file constant convention (`api_bolts.rs`,
// `surfaces.rs` each carry their own copy already).
// ---------------------------------------------------------------------------

/// Source: `oracle/codemp/ghoul2/G2.h:30-31`
const MODEL_WIDTH: i32 = 10;
const BOLT_WIDTH: i32 = 10;
const MODEL_AND: i32 = (1 << MODEL_WIDTH) - 1;
const BOLT_AND: i32 = (1 << BOLT_WIDTH) - 1;
const BOLT_SHIFT: i32 = 0;
const MODEL_SHIFT: i32 = BOLT_SHIFT + BOLT_WIDTH;

/// Raven `#define GHOUL2_NEWORIGIN 0x008` (`ghoul2_shared.h:232`) — read by
/// `root_matrix` alone in this file (only `api_bolts.rs`'s
/// `g2api_set_new_origin` writes it), matching this crate's per-file constant
/// duplication convention.
const GHOUL2_NEWORIGIN: i32 = 0x008;

// ---------------------------------------------------------------------------
// mdxaHeader_t / mdxaSkel_t byte-offset helpers — a further, file-local copy
// of the duplication `bones.rs`/`api_bones.rs` already carry for the same
// header (`G2SV-D5`: this crate never names `mdxaHeader_t`/`mdxaSkel_t`).
//
// Source: `oracle/codemp/renderer/mdx_format.h:349-396`
// ---------------------------------------------------------------------------

/// `mdxaHeader_t` field order: `ident,version:i32` (8) + `name[MAX_QPATH]` +
/// `fScale:f32` (4) then `numFrames`.
const MDXA_NUM_FRAMES_OFFSET: usize = 4 + 4 + MAX_QPATH + 4;
/// `numFrames` (4) + `ofsFrames` (4) then `numBones`.
const MDXA_NUM_BONES_OFFSET: usize = MDXA_NUM_FRAMES_OFFSET + 4 + 4;
/// `numBones` (4) + `ofsCompBonePool` (4) + `ofsSkel` (4) + `ofsEnd` (4) =
/// `sizeof(mdxaHeader_t)`.
const MDXA_HEADER_SIZE: usize = MDXA_NUM_BONES_OFFSET + 4 + 4 + 4 + 4;
/// `mdxaSkel_t::BasePoseMat` offset: `name[MAX_QPATH]`(64) + `flags`(4) +
/// `parent`(4) precede it.
const SKEL_OFS_BASE_POSE_MAT: usize = MAX_QPATH + 4 + 4;
/// `mdxaSkel_t::BasePoseMatInv` offset: `BasePoseMat` (48 bytes, `mdxaBone_t`)
/// precedes it.
const SKEL_OFS_BASE_POSE_MAT_INV: usize = SKEL_OFS_BASE_POSE_MAT + 48;

/// Raven `mdxaHeader_t->numBones` — `tr_ghoul2.cpp:2104` (`!aHeader->numBones`).
///
/// # Safety
/// `header` must be a valid, non-null `EngineHost::model_mdxa` block pointer.
pub(crate) unsafe fn mdxa_num_bones(header: *const c_void) -> i32 {
    core::ptr::read_unaligned((header as *const u8).add(MDXA_NUM_BONES_OFFSET) as *const i32)
}

/// Raven `(mdxaSkelOffsets_t*)((byte*)header + sizeof(mdxaHeader_t))->
/// offsets[boneNum]` then `(mdxaSkel_t*)(base + offset)` —
/// `tr_ghoul2.cpp:672-673,705-706,745-746,3297-3298`.
///
/// # Safety
/// `header` must be a valid, non-null `EngineHost::model_mdxa` block pointer
/// and `bone_index` must be `< numBones`.
pub(crate) unsafe fn mdxa_skel_ptr(header: *const c_void, bone_index: i32) -> *const u8 {
    let base = (header as *const u8).add(MDXA_HEADER_SIZE);
    let skel_offset = core::ptr::read_unaligned((base as *const i32).add(bone_index as usize));
    base.offset(skel_offset as isize)
}

/// Raven's file-scope `const static mdxaBone_t identityMatrix` —
/// `tr_ghoul2.cpp:128-133`. A real (not `const`) item so the "yikes"/no-cache
/// fallback paths below can hand out a stable `*mut mdxaBone_t` into it,
/// mirroring Raven's own `const_cast<mdxaBone_t *>(&identityMatrix)`.
static IDENTITY_MATRIX: mdxaBone_t = mdxaBone_t {
    matrix: [
        [0.0, -1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
    ],
};


// ---------------------------------------------------------------------------
// mdxm mesh byte-offset table — the surface-bolt path (`G2_FindSurface_BC` /
// `G2_ProcessSurfaceBolt2`) walks `mod->mdxm`'s LOD/surface/vertex arrays.
// Per `G2SV-D5` this crate never names `mdxmHeader_t`/`mdxmSurface_t`/
// `mdxmVertex_t` — only byte arithmetic off the raw `EngineHost::model_mdxm`
// pointer, exactly as the oracle does off `model_t::mdxm`. Offsets derive from
// the field order in `oracle/codemp/renderer/mdx_format.h`; duplicated locally
// per this crate's own convention (`surfaces.rs`'s identical header table).
// ---------------------------------------------------------------------------

/// `mdxmHeader_t::ofsLODs` (`mdx_format.h:166`) — `animIndex`(136) + `numBones`
/// (4) + `numLODs`(4) precede it.
const MDXM_OFS_OFS_LODS: usize = 148;
/// `sizeof(mdxmLOD_t)` — a single `int ofsEnd` (`mdx_format.h:203-207`), whose
/// value is read at offset 0 while walking to the requested LOD.
const MDXM_LOD_SIZE: usize = 4;

/// `mdxmSurface_t::ofsVerts` (`mdx_format.h:227`) — `ident`(0) + `thisSurface
/// Index`(4) + `ofsHeader`(8) + `numVerts`(12) precede it.
const SURF_OFS_VERTS: usize = 16;
/// `mdxmSurface_t::ofsTriangles` (`mdx_format.h:230`) — `numTriangles`(20)
/// precedes it.
const SURF_OFS_TRIANGLES: usize = 24;
/// `mdxmSurface_t::ofsBoneReferences` (`mdx_format.h:240`) — `numBone
/// References`(28) precedes it.
const SURF_OFS_BONE_REFERENCES: usize = 32;

/// `sizeof(mdxmVertex_t)` (non-`_XBOX`): `normal`(12) + `vertCoords`(12) +
/// `uiNmWeightsAndBoneIndexes`(4) + `BoneWeightings[4]`(4) — "kept at 32 bytes
/// for cache-aligning" (`mdx_format.h:263-280`).
const MDXM_VERTEX_SIZE: usize = 32;
/// `sizeof(mdxmTriangle_t)` — `int indexes[3]` (`mdx_format.h:250-253`).
const MDXM_TRIANGLE_SIZE: usize = 12;
/// `mdxmVertex_t::vertCoords` (`mdx_format.h:272`) — `normal`(12) precedes it.
const VERT_OFS_VERT_COORDS: usize = 12;
/// `mdxmVertex_t::uiNmWeightsAndBoneIndexes` (`mdx_format.h:281`).
const VERT_OFS_WEIGHTS_AND_INDEXES: usize = 24;
/// `mdxmVertex_t::BoneWeightings[0]` (`mdx_format.h:288`).
const VERT_OFS_BONE_WEIGHTINGS: usize = 28;

/// `#define iG2_BITS_PER_BONEREF 5` (`mdx_format.h:61`).
const IG2_BITS_PER_BONEREF: u32 = 5;
/// `#define iG2_BONEWEIGHT_TOPBITS_SHIFT ((5 * 4) - 8)` (`mdx_format.h:65`).
const IG2_BONEWEIGHT_TOPBITS_SHIFT: u32 = 12;
/// `#define iG2_BONEWEIGHT_TOPBITS_AND 0x300` (`mdx_format.h:66`).
const IG2_BONEWEIGHT_TOPBITS_AND: u32 = 0x300;
/// `#define fG2_BONEWEIGHT_RECIPROCAL_MULT ((float)(1.0f/1023.0f))`
/// (`mdx_format.h:60`).
const FG2_BONEWEIGHT_RECIPROCAL_MULT: f32 = 1.0 / 1023.0;
/// `#define iG2_TRISIDE_LONGEST 0` (`mdx_format.h:57`).
const IG2_TRISIDE_LONGEST: usize = 0;
/// `#define iG2_TRISIDE_SHORTEST 2` (`mdx_format.h:58`).
const IG2_TRISIDE_SHORTEST: usize = 2;
/// `#define MDX_TAG_ORIGIN 2` (`tr_ghoul2.cpp:2238`).
const MDX_TAG_ORIGIN: usize = 2;
/// `#define G2SURFACEFLAG_GENERATED 0x00000200` (`mdx_format.h:50`) — the
/// procedurally-generated tag-surface marker. Duplicated locally, matching
/// `surfaces.rs`'s identical copy.
const G2SURFACEFLAG_GENERATED: i32 = 0x00000200;

/// Read an `i32` at `offset` bytes into a raw block (the block is a
/// `EngineHost::model_mdxm` pointer, or an interior surface/vertex pointer —
/// `G2SV-D5`, the header types are never named). `read_unaligned` because
/// nothing here proves 4-byte alignment; native byte order, no cross-endian
/// concern at this layer (mirrors `surfaces.rs`'s `read_i32_at`).
///
/// # Safety
/// `base` non-null and `offset..offset+4` inside the block.
unsafe fn read_i32(base: *const u8, offset: usize) -> i32 {
    unsafe { base.add(offset).cast::<i32>().read_unaligned() }
}

/// Read an `f32` at `offset` bytes into a raw block. See [`read_i32`].
///
/// # Safety
/// `base` non-null and `offset..offset+4` inside the block.
unsafe fn read_f32(base: *const u8, offset: usize) -> f32 {
    unsafe { base.add(offset).cast::<f32>().read_unaligned() }
}


/// `G2_GetVertWeights` (`mdx_format.h:290-295`) — the packed weight count (1..4).
///
/// # Safety
/// `vert` must point at a valid `mdxmVertex_t`.
unsafe fn vert_weights(vert: *const u8) -> i32 {
    let packed = unsafe { read_i32(vert, VERT_OFS_WEIGHTS_AND_INDEXES) } as u32;
    ((packed >> 30) + 1) as i32
}

/// `G2_GetVertBoneIndex` (`mdx_format.h:297-302`) — the `iWeightNum`-th 5-bit
/// bone reference.
///
/// # Safety
/// `vert` must point at a valid `mdxmVertex_t`.
unsafe fn vert_bone_index(vert: *const u8, weight_num: i32) -> i32 {
    let packed = unsafe { read_i32(vert, VERT_OFS_WEIGHTS_AND_INDEXES) } as u32;
    ((packed >> (IG2_BITS_PER_BONEREF * weight_num as u32)) & ((1 << IG2_BITS_PER_BONEREF) - 1))
        as i32
}

/// `G2_GetVertBoneWeight` (`mdx_format.h:304-323`) — the `iWeightNum`-th bone
/// weight; the last weight closes to `1.0 - fTotalWeight`, the rest decode from
/// the 8-bit `BoneWeightings[]` entry plus its 2-bit overflow in the packed int
/// and accumulate into `total_weight`.
///
/// # Safety
/// `vert` must point at a valid `mdxmVertex_t`.
unsafe fn vert_bone_weight(
    vert: *const u8,
    weight_num: i32,
    total_weight: &mut f32,
    num_weights: i32,
) -> f32 {
    if weight_num == num_weights - 1 {
        1.0 - *total_weight
    } else {
        let packed = unsafe { read_i32(vert, VERT_OFS_WEIGHTS_AND_INDEXES) } as u32;
        let mut temp = unsafe { *vert.add(VERT_OFS_BONE_WEIGHTINGS + weight_num as usize) } as i32;
        temp |= ((packed >> (IG2_BONEWEIGHT_TOPBITS_SHIFT + (weight_num as u32 * 2)))
            & IG2_BONEWEIGHT_TOPBITS_AND) as i32;
        let bone_weight = FG2_BONEWEIGHT_RECIPROCAL_MULT * temp as f32;
        *total_weight += bone_weight;
        bone_weight
    }
}

/// Transform one `mdxmVertex_t` into model space by its weighted bones — the
/// inner accumulation loop `G2_ProcessSurfaceBolt2` runs per triangle vertex
/// (`tr_ghoul2.cpp:2288-2302` and its three siblings). `bone_refs` is the
/// surface's `int` bone-reference array; each vert bone index selects into it,
/// and the referenced bone is evaluated through the cache (`Eval`, not
/// `EvalRender` — `G2EVALRENDER` is undefined).
///
/// # Safety
/// `vert` a valid `mdxmVertex_t`, `bone_refs` the surface's bone-reference
/// array, and `cache` the surface's model's live bone cache.
unsafe fn transform_vertex(
    cache: &mut CBoneCache,
    vert: *const u8,
    bone_refs: *const u8,
) -> [f32; 3] {
    // VectorClear( pTri[j] );
    let mut p = [0.0f32; 3];
    let vert_coords = unsafe {
        [
            read_f32(vert, VERT_OFS_VERT_COORDS),
            read_f32(vert, VERT_OFS_VERT_COORDS + 4),
            read_f32(vert, VERT_OFS_VERT_COORDS + 8),
        ]
    };
    let num_weights = unsafe { vert_weights(vert) };
    let mut total_weight = 0.0f32;
    for k in 0..num_weights {
        let bone_index = unsafe { vert_bone_index(vert, k) };
        let bone_weight = unsafe { vert_bone_weight(vert, k, &mut total_weight, num_weights) };
        let bone_ref = unsafe { read_i32(bone_refs, 4 * bone_index as usize) };
        let bone = cache.eval(bone_ref);
        p[0] += bone_weight * (DotProductRow(&bone.matrix[0], vert_coords) + bone.matrix[0][3]);
        p[1] += bone_weight * (DotProductRow(&bone.matrix[1], vert_coords) + bone.matrix[1][3]);
        p[2] += bone_weight * (DotProductRow(&bone.matrix[2], vert_coords) + bone.matrix[2][3]);
    }
    p
}

/// Raven `void *G2_FindSurface_BC(const model_s *mod, int index, int lod)` —
/// the by-index surface lookup the surface-bolt path uses: walk `mod->mdxm`'s
/// LOD list to `lod`, step over the `mdxmLOD_t` header to the
/// `mdxmLODSurfOffset_t` offset array, and return the pointer to surface
/// `index`. Takes the raw `mdxm` block directly (this crate never names
/// `model_s`/`mdxmHeader_t`, `G2SV-D5`); the caller resolves it once via
/// `EngineHost::model_mdxm`.
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:2952-2977`
///
/// # Safety
/// `mdxm` a valid, non-null `EngineHost::model_mdxm` block; `lod` in
/// `0..numLODs`; `index` in `0..numSurfaces` (matching the oracle's own
/// unchecked-under-`NDEBUG` asserts).
unsafe fn find_surface_bc(mdxm: *const c_void, index: i32, lod: i32) -> *const c_void {
    unsafe {
        let mdxm = mdxm as *const u8;
        // point at first lod list
        let mut current = mdxm.add(read_i32(mdxm, MDXM_OFS_OFS_LODS) as usize);
        // walk the lods (mdxmLOD_t::ofsEnd is its only, first, field)
        for _ in 0..lod {
            let ofs_end = read_i32(current, 0);
            current = current.add(ofs_end as usize);
        }
        // avoid the lod pointer data structure
        current = current.add(MDXM_LOD_SIZE);
        // we are now looking at the offset array (mdxmLODSurfOffset_t)
        let offset = read_i32(current, 4 * index as usize);
        current.add(offset as usize) as *const c_void
    }
}

/// Raven `void G2_ProcessSurfaceBolt2(CBoneCache &boneCache, const
/// mdxmSurface_t *surface, int boltNum, boltInfo_v &boltList, const
/// surfaceInfo_t *surfInfo, const model_t *mod, mdxaBone_t &retMatrix)` —
/// computes a surface-attached bolt's matrix. Two surface kinds: a
/// procedurally-generated tag (`surfInfo->offFlags == G2SURFACEFLAG_GENERATED`)
/// re-finds the hit poly's original three verts, transforms them through the
/// bone cache, weights them by the stored barycentric coordinates to get the
/// origin, and builds an orthonormal basis from the poly normal + towards-point-0
/// up; else a normal model tag transforms the surface's first triangle and
/// derives the basis from its longest/shortest sides. Always writes all twelve
/// matrix entries, so returned by value per §C7. `boltNum`/`boltList` are unread
/// by the body (kept out of this port's parameter list; the caller already holds
/// them). `mod` collapses to the raw `mdxm` block (the generated path's own
/// `G2_FindSurface_BC` call needs it; `G2SV-D5`).
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:2983-3251`
fn g2_process_surface_bolt2(
    cache: &mut CBoneCache,
    surface: *const c_void,
    surf_info: Option<&surfaceInfo_t>,
    mdxm: *const c_void,
) -> mdxaBone_t {
    let mut ret_matrix = IDENTITY_MATRIX;

    // now there are two types of tag surface - model ones and procedural
    // generated types - lets decide which one we have here.
    if let Some(surf_info) = surf_info.filter(|s| s.offFlags == G2SURFACEFLAG_GENERATED) {
        let surf_number = surf_info.genPolySurfaceIndex & 0x0ffff;
        let poly_number = (surf_info.genPolySurfaceIndex >> 16) & 0x0ffff;

        // SAFETY: `mdxm` is the surface's model block; `surf_number`/`gen_lod`
        // index the poly's original surface, matching the oracle's own
        // unchecked reads off `mod->mdxm`.
        unsafe {
            // find original surface our original poly was in.
            let original_surf = find_surface_bc(mdxm, surf_number, surf_info.genLod) as *const u8;
            let tri_base = original_surf.add(read_i32(original_surf, SURF_OFS_TRIANGLES) as usize);
            let poly = tri_base.add(poly_number as usize * MDXM_TRIANGLE_SIZE);

            // get the original polys indexes
            let index0 = read_i32(poly, 0);
            let index1 = read_i32(poly, 4);
            let index2 = read_i32(poly, 8);

            // decide where the original verts are
            let vert_base = original_surf.add(read_i32(original_surf, SURF_OFS_VERTS) as usize);
            let vert0 = vert_base.add(index0 as usize * MDXM_VERTEX_SIZE);
            let vert1 = vert_base.add(index1 as usize * MDXM_VERTEX_SIZE);
            let vert2 = vert_base.add(index2 as usize * MDXM_VERTEX_SIZE);

            let bone_refs =
                original_surf.add(read_i32(original_surf, SURF_OFS_BONE_REFERENCES) as usize);

            // now go and transform just the points we need from the surface
            // that was hit originally.
            let p_tri = [
                transform_vertex(cache, vert0, bone_refs),
                transform_vertex(cache, vert1, bone_refs),
                transform_vertex(cache, vert2, bone_refs),
            ];

            // work out baryCentricK (Raven's `float baryCentricK = 1.0 - (...)`
            // — the `1.0` double literal makes this one subtraction a double
            // intermediate before the store to `float`; preserved).
            let bary_centric_k =
                (1.0_f64 - (surf_info.genBarycentricI + surf_info.genBarycentricJ) as f64) as f32;

            // now we have the model transformed into model space, now generate
            // an origin.
            ret_matrix.matrix[0][3] = (p_tri[0][0] * surf_info.genBarycentricI)
                + (p_tri[1][0] * surf_info.genBarycentricJ)
                + (p_tri[2][0] * bary_centric_k);
            ret_matrix.matrix[1][3] = (p_tri[0][1] * surf_info.genBarycentricI)
                + (p_tri[1][1] * surf_info.genBarycentricJ)
                + (p_tri[2][1] * bary_centric_k);
            ret_matrix.matrix[2][3] = (p_tri[0][2] * surf_info.genBarycentricI)
                + (p_tri[1][2] * surf_info.genBarycentricJ)
                + (p_tri[2][2] * bary_centric_k);

            // generate a normal to this new triangle
            let mut vec0 = [0.0f32; 3];
            _VectorSubtract(p_tri[0], p_tri[1], &mut vec0);
            let mut vec1 = [0.0f32; 3];
            _VectorSubtract(p_tri[2], p_tri[1], &mut vec1);
            let mut normal = [0.0f32; 3];
            CrossProduct(vec0, vec1, &mut normal);
            VectorNormalize(&mut normal);

            // forward vector
            ret_matrix.matrix[0][0] = normal[0];
            ret_matrix.matrix[1][0] = normal[1];
            ret_matrix.matrix[2][0] = normal[2];

            // up will be towards point 0 of the original triangle. so lets work
            // it out. Vector is hit point - point 0
            let mut up = [
                ret_matrix.matrix[0][3] - p_tri[0][0],
                ret_matrix.matrix[1][3] - p_tri[0][1],
                ret_matrix.matrix[2][3] - p_tri[0][2],
            ];
            // normalise it
            VectorNormalize(&mut up);

            // that's the up vector
            ret_matrix.matrix[0][1] = up[0];
            ret_matrix.matrix[1][1] = up[1];
            ret_matrix.matrix[2][1] = up[2];

            // right is always straight
            let mut right = [0.0f32; 3];
            CrossProduct(normal, up, &mut right);
            ret_matrix.matrix[0][2] = right[0];
            ret_matrix.matrix[1][2] = right[1];
            ret_matrix.matrix[2][2] = right[2];
        }
    } else {
        // no, we are looking at a normal model tag.
        //
        // Divergence (§19): oracle derefs `surface` unconditionally; null here is an
        // unreachable oracle null-deref (UB) — pick the defined identity fallback.
        if surface.is_null() {
            return ret_matrix;
        }
        let surface = surface as *const u8;

        // SAFETY: `surface` is a valid `mdxmSurface_t` (`G2_FindSurface_BC` or
        // the slist `surfInfo`); its `ofsVerts`/`ofsBoneReferences` interior
        // reads match the oracle's own.
        unsafe {
            // whip through and actually transform each vertex
            let mut v = surface.add(read_i32(surface, SURF_OFS_VERTS) as usize);
            let bone_refs = surface.add(read_i32(surface, SURF_OFS_BONE_REFERENCES) as usize);

            let mut p_tri = [[0.0f32; 3]; 3];
            for slot in p_tri.iter_mut() {
                *slot = transform_vertex(cache, v, bone_refs);
                // v++ (advance by sizeof(mdxmVertex_t))
                v = v.add(MDXM_VERTEX_SIZE);
            }

            // work out actual sides of the tag triangle
            let mut sides = [[0.0f32; 3]; 3];
            for j in 0..3 {
                sides[j][0] = p_tri[(j + 1) % 3][0] - p_tri[j][0];
                sides[j][1] = p_tri[(j + 1) % 3][1] - p_tri[j][1];
                sides[j][2] = p_tri[(j + 1) % 3][2] - p_tri[j][2];
            }

            // do math trig to work out what the matrix will be from this
            // triangle's translated position
            let mut axes = [[0.0f32; 3]; 3];
            VectorNormalize2(sides[IG2_TRISIDE_LONGEST], &mut axes[0]);
            VectorNormalize2(sides[IG2_TRISIDE_SHORTEST], &mut axes[1]);

            // project shortest side so that it is exactly 90 degrees to the
            // longer side
            let d = _DotProduct(axes[0], axes[1]);
            _VectorMA(axes[0], -d, axes[1], &mut axes[0]);
            let a0 = axes[0];
            VectorNormalize2(a0, &mut axes[0]);

            CrossProduct(
                sides[IG2_TRISIDE_LONGEST],
                sides[IG2_TRISIDE_SHORTEST],
                &mut axes[2],
            );
            let a2 = axes[2];
            VectorNormalize2(a2, &mut axes[2]);

            // set up location in world space of the origin point in out going
            // matrix
            ret_matrix.matrix[0][3] = p_tri[MDX_TAG_ORIGIN][0];
            ret_matrix.matrix[1][3] = p_tri[MDX_TAG_ORIGIN][1];
            ret_matrix.matrix[2][3] = p_tri[MDX_TAG_ORIGIN][2];

            // copy axis to matrix - do some magic to orient minus Y to positive
            // X and so on so bolt on stuff is oriented correctly
            ret_matrix.matrix[0][0] = axes[1][0];
            ret_matrix.matrix[0][1] = axes[0][0];
            ret_matrix.matrix[0][2] = -axes[2][0];

            ret_matrix.matrix[1][0] = axes[1][1];
            ret_matrix.matrix[1][1] = axes[0][1];
            ret_matrix.matrix[1][2] = -axes[2][1];

            ret_matrix.matrix[2][0] = axes[1][2];
            ret_matrix.matrix[2][1] = axes[0][2];
            ret_matrix.matrix[2][2] = -axes[2][2];
        }
    }

    ret_matrix
}

/// Shared, alias-free core of `G2_GetBoltMatrixLow` (`tr_ghoul2.cpp:3253-3331`):
/// resolves one bolt's matrix from an already-built `CBoneCache` alone. Both
/// the pinned [`g2_get_bolt_matrix_low`] entry point and this file's own
/// `g2_construct_ghoul_skeleton`/`root_matrix` (which need a *different*
/// model's already-computed cache while `&mut Ghoul2System` is borrowed
/// elsewhere in the same call) route through this instead of hitting the
/// E0502 documented in the module doc above.
///
/// The surface-attached arm (`:3301-3324`) locates the bolt's surface — an
/// override entry in `slist` matching `surfaceNumber`, else the mesh surface by
/// index (`G2_FindSurface_BC`) — then computes the bolt matrix from its verts
/// (`g2_process_surface_bolt2`). `host` resolves the surface's `mdxm` block.
/// `pub(crate)`: `api_bolts::g2api_get_bolt_matrix` needs this same split-
/// borrow shape (its arena instance is already field-projected out of `g2`).
pub(crate) fn resolve_bolt_matrix_low(
    bone_caches: &mut BoneCacheArena,
    host: &mut impl EngineHost,
    bone_cache: Option<BoneCacheId>,
    slist: &[surfaceInfo_t],
    bltlist: &[boltInfo_t],
    bolt_num: i32,
    scale: vec3_t,
) -> mdxaBone_t {
    let Some(cache) = bone_cache.and_then(|id| bone_caches.get_mut(id)) else {
        // Raven: `if (!ghoul2.mBoneCache) { retMatrix=identityMatrix; return; }`.
        return IDENTITY_MATRIX;
    };
    // Raven's `assert(boltNum>=0&&boltNum<boltList.size())` is a compiled
    // no-op under this build's `-DNDEBUG` (module doc `## Raven ground
    // truth`); an out-of-range index is oracle UB. This port picks the
    // defined "treat as unbolted" identity fallback instead of an OOB panic,
    // matching the crate-wide convention already documented in `api_bolts.rs`.
    let Some(bolt) = bltlist.get(bolt_num as usize) else {
        return IDENTITY_MATRIX;
    };

    if bolt.boneNumber >= 0 {
        let evaluated = cache.eval_unsmooth(bolt.boneNumber);
        // SAFETY: `cache.header` is the block `EngineHost::model_mdxa` handed
        // back for this cache's model (`CBoneCache::new`/refresh); `boneNumber`
        // is caller-set bone data, matching Raven's own unchecked read.
        let base_pose_mat: mdxaBone_t = unsafe {
            let skel = mdxa_skel_ptr(cache.header as *const c_void, bolt.boneNumber);
            core::ptr::read_unaligned(skel.add(SKEL_OFS_BASE_POSE_MAT) as *const mdxaBone_t)
        };
        let mut ret_matrix = IDENTITY_MATRIX;
        // Raven: `Multiply_3x4Matrix(&retMatrix, (mdxaBone_t*)&boneCache.
        // EvalUnsmooth(...), &skel->BasePoseMat);` — dest first arg.
        bone_transform::multiply_3x4_matrix(&mut ret_matrix, &evaluated, &base_pose_mat);
        ret_matrix
    } else if bolt.surfaceNumber >= 0 {
        // find the override surfaceInfo for this bolt's surface, if any. The
        // oracle loop has no `break`, so the LAST match wins.
        let mut surf_info: Option<&surfaceInfo_t> = None;
        for t in slist {
            if t.surface == bolt.surfaceNumber {
                surf_info = Some(t);
            }
        }

        let mdxm = host.model_mdxm(cache.model) as *const c_void;
        let mut surface: *const c_void = core::ptr::null();
        // SAFETY: `mdxm` is `cache.model`'s loader block — non-null on the live
        // bolt path (a built cache implies a valid model), matching the oracle's
        // own unchecked `boneCache.mod->mdxm`.
        if surf_info.is_none() {
            surface = unsafe { find_surface_bc(mdxm, bolt.surfaceNumber, 0) };
        }
        if surface.is_null() {
            if let Some(si) = surf_info {
                if si.surface < 10000 {
                    surface = unsafe { find_surface_bc(mdxm, si.surface, 0) };
                }
            }
        }
        let _ = scale;
        g2_process_surface_bolt2(cache, surface, surf_info, mdxm)
    } else {
        // Raven: "we have a bolt without a bone or surface, not a huge
        // problem but we ought to at least clear the bolt matrix."
        IDENTITY_MATRIX
    }
}

/// Alias-free core of `G2_TransformGhoulBones` (`tr_ghoul2.cpp:2075-2234`),
/// taking only the sibling `bone_caches` arena instead of the whole
/// `Ghoul2System` — see the module-doc aliasing finding. The pinned
/// [`g2_transform_ghoul_bones`] wrapper and `g2_construct_ghoul_skeleton`'s
/// own loop (which already holds a split `&mut Ghoul2System` borrow) both
/// route through this.
fn g2_transform_ghoul_bones_inner(
    bone_caches: &mut BoneCacheArena,
    host: &mut impl EngineHost,
    root_bone_list: *mut Vec<boneInfo_t>,
    root_matrix: &mdxaBone_t,
    ghoul2: &mut CGhoul2Info,
    time: i32,
    smooth: bool,
) {
    // `HackadelicOnClient` is const-`false` server-side (module doc `##
    // Raven ground truth`), so `smooth` never affects the folded-`else`
    // bodies below — kept as a parameter only for 1:1 arity (`G2SV-D6`-style
    // fidelity), matching `api_bolts.rs`'s own `let _ = model_list;` precedent
    // for an unread Raven parameter.
    let _ = smooth;

    // Raven: `model_t *currentModel=(model_t*)ghoul2.currentModel; mdxaHeader_t
    // *aHeader=(mdxaHeader_t*)ghoul2.aHeader;` then the (compiled-out-under-
    // NDEBUG) non-null asserts. Per `G2SV-D5` this crate routes every header
    // read through `EngineHost::model_mdxa(ghoul2.model)` rather than trusting
    // the already-cached opaque `ghoul2.a_header` field — same underlying
    // loader block, sanctioned single source of truth (module doc `##
    // Raven ground truth` / `G2SV-D15`).
    let header = host.model_mdxa(ghoul2.model);
    if header.is_null() {
        // Divergence (§19): Raven's own non-null asserts are dead under
        // `-DNDEBUG`, so a genuine null here is oracle UB (a hard crash) that
        // in practice is never reached (`G2_SetupModelPointers` validates
        // first). Picking the defined "do nothing" behavior instead.
        return;
    }
    // Raven: `if (!aHeader->numBones) { assert(0); return; }` — the assert is
    // a compiled no-op under NDEBUG, but the guard + `return` are real control
    // flow that survives.
    if unsafe { mdxa_num_bones(header as *const c_void) } == 0 {
        return;
    }

    let cache_id = match ghoul2.bone_cache {
        Some(id) => id,
        None => {
            // Raven: `ghoul2.mBoneCache=new CBoneCache(currentModel,aHeader);`
            let cache = CBoneCache::new(host, ghoul2.model);
            let id = bone_caches.insert(cache);
            ghoul2.bone_cache = Some(id);
            id
        }
    };
    let Some(cache) = bone_caches.get_mut(cache_id) else {
        // The arena slot vanished out from under a live handle — unreachable
        // through this crate's own arena API (`remove` only runs through
        // `Ghoul2System::delete_low`/`RemoveBoneCache`), kept total rather
        // than panicking.
        return;
    };

    // Raven: `ghoul2.mBoneCache->mod=currentModel; header=aHeader;`
    cache.model = ghoul2.model;
    cache.header = header;

    cache.smoothing_active = false;
    cache.unsquash = false;

    // Master smoothing control (`:2124-2201`) and the touch-render stamp
    // (`:2205-2215`): `HackadelicOnClient` is const-`false` server-side
    // (module doc), so both permanently fold to their `else` arms —
    // `mSmoothFactor=1.0f` and `mCurrentTouchRender=0` unconditionally.
    cache.smooth_factor = 1.0;
    cache.current_touch += 1;
    cache.current_touch_render = 0;

    // Raven: `ghoul2.mBoneCache->frameSize = 0;` ("can be deleted in new G2
    // format").
    cache.frame_size = 0;
    cache.root_bone_list = root_bone_list;
    cache.root_matrix = *root_matrix;
    cache.incoming_time = time;

    let root = cache.root();
    root.new_frame = 0;
    root.current_frame = 0;
    root.backlerp = 0.0;
    root.blend_frame = 0.0;
    root.blend_old_frame = 0;
    root.blend_mode = false;
    root.blend_lerp = 0.0;
}

/// Raven `void G2_ConstructGhoulSkeleton(CGhoul2Info_v &ghoul2, const int
/// frameNum, bool checkForNewOrigin, const vec3_t scale)` — "builds a
/// complete skeleton for all ghoul models in a `CGhoul2Info_v` class - using
/// LOD 0" (Raven's own header comment). Sorts the models (`G2_Sort_Models`)
/// so bolt-parents build before their bolt-children, computes the root matrix
/// (`RootMatrix` when `check_for_new_origin`, else identity), then drives
/// `G2_TransformGhoulBones` per valid model — either off a bolt matrix
/// (`G2_GetBoltMatrixLow`) when bolted to another model, or off the shared
/// root matrix otherwise.
///
/// Frozen verbatim in `docs/subsystems/ghoul2-server.md` `## Seam definition`.
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:3565-3624`
pub fn g2_construct_ghoul_skeleton(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v,
    frame_num: i32,
    check_for_new_origin: bool,
    scale: vec3_t,
) {
    let root_mtx = if check_for_new_origin {
        root_matrix(g2, host, ghoul2, frame_num, scale)
    } else {
        IDENTITY_MATRIX
    };

    let model_list = g2_sort_models(g2, ghoul2);
    let item = ghoul2.mItem;

    for (j, &i) in model_list.iter().enumerate() {
        // Disjoint-borrow `info_array`/`bone_caches` (module-doc aliasing
        // finding): the bolt-attached arm needs a DIFFERENT model's
        // already-built cache (built earlier in this same sort-ordered loop,
        // per `G2_Sort_Models`' topological guarantee) while this model's own
        // cache is mutated — the pinned `g2_get_bolt_matrix_low`/
        // `g2_transform_ghoul_bones` signatures can't express that borrow, so
        // this loop calls the alias-free cores directly instead.
        let Ghoul2System {
            info_array,
            bone_caches,
            ..
        } = &mut *g2;
        let models = info_array.get_mut(item);
        let i_usize = i as usize;

        // Raven: `if (ghoul2[i].mValid)` — only `mValid` is checked here
        // (unlike `G2_Sort_Models`/`RootMatrix`, which also check
        // `mModelindex != -1`).
        if !models[i_usize].valid {
            continue;
        }

        let model_bolt_link = models[i_usize].model_bolt_link;
        if j != 0 && model_bolt_link != -1 {
            let bolt_mod = ((model_bolt_link >> MODEL_SHIFT) & MODEL_AND) as usize;
            let bolt_num = (model_bolt_link >> BOLT_SHIFT) & BOLT_AND;

            let bolt = {
                let parent = &models[bolt_mod];
                resolve_bolt_matrix_low(
                    bone_caches,
                    host,
                    parent.bone_cache,
                    &parent.slist,
                    &parent.bltlist,
                    bolt_num,
                    scale,
                )
            };

            let blist_ptr = &mut models[i_usize].blist as *mut Vec<boneInfo_t>;
            let ghl = &mut models[i_usize];
            g2_transform_ghoul_bones_inner(
                bone_caches,
                host,
                blist_ptr,
                &bolt,
                ghl,
                frame_num,
                check_for_new_origin,
            );
        } else {
            let blist_ptr = &mut models[i_usize].blist as *mut Vec<boneInfo_t>;
            let ghl = &mut models[i_usize];
            g2_transform_ghoul_bones_inner(
                bone_caches,
                host,
                blist_ptr,
                &root_mtx,
                ghl,
                frame_num,
                check_for_new_origin,
            );
        }
    }
}

/// Raven `void G2_TransformGhoulBones(boneInfo_v &rootBoneList, mdxaBone_t
/// &rootMatrix, CGhoul2Info &ghoul2, int time, bool smooth=true)` — builds
/// `ghoul2`'s `CBoneCache` on first use (`CBoneCache::new`, `render/
/// bone_cache.rs`) or refreshes an existing one's `header`/`model` from the
/// current loader model memory, resets the per-construct smoothing/touch
/// state, stores `root_bone_list`/`root_matrix`/`time` as the cache's scratch
/// fields for the transform chain (`render/bone_transform.rs`), and seeds the
/// traversal root's `SBoneCalc`. The C++ default arg (`smooth=true`) has no
/// Rust equivalent; callers pass it explicitly.
///
/// `HackadelicOnClient` is const-`false` server-side (`## Raven ground
/// truth`), so the `if (HackadelicOnClient && smooth && !com_dedicated->
/// integer)` smoothing-control branch and the `if (HackadelicOnClient) {...}`
/// touch-render branch both permanently fold to their `else` arms here
/// (§C10) — `mSmoothFactor=1.0f` and `mCurrentTouchRender=0` unconditionally.
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:2075-2234`
pub fn g2_transform_ghoul_bones(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    root_bone_list: *mut Vec<boneInfo_t>,
    root_matrix: &mdxaBone_t,
    ghoul2: &mut CGhoul2Info,
    time: i32,
    smooth: bool,
) {
    g2_transform_ghoul_bones_inner(
        &mut g2.bone_caches,
        host,
        root_bone_list,
        root_matrix,
        ghoul2,
        time,
        smooth,
    );
}

/// Raven `void G2_GetBoneMatrixLow(CGhoul2Info &ghoul2, int boneNum, const
/// vec3_t scale, mdxaBone_t &retMatrix, mdxaBone_t *&retBasepose, mdxaBone_t
/// *&retBaseposeInv)` — evaluates bone `bone_num` (`CBoneCache::Eval`,
/// touch-memoized), applies `scale` to the translation column and
/// re-normalizes the rotation basis, then premultiplies by `world_matrix`
/// (Raven's file-scope `worldMatrix`, threaded explicitly here — module-doc
/// note) to produce the returned matrix; `retBasepose`/`retBaseposeInv` alias
/// the bone's `mdxaSkel_t` basepose matrices out of the cache's `header`. Every
/// path writes all three outputs (including the no-bone-cache "yikes" identity
/// fallback), so per §C7's out-param default this returns them by value
/// instead of write-through out-params — there is no failure signal to
/// preserve (`G2SV-D1`'s discriminator does not apply: this is not a
/// `qboolean`-returning function).
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:727-778`
pub fn g2_get_bone_matrix_low(
    g2: &mut Ghoul2System,
    ghoul2: &CGhoul2Info,
    bone_num: i32,
    scale: vec3_t,
    world_matrix: &mdxaBone_t,
) -> (mdxaBone_t, *mut mdxaBone_t, *mut mdxaBone_t) {
    let Some(cache) = ghoul2.bone_cache.and_then(|id| g2.bone_caches.get_mut(id)) else {
        // Raven: "yikes" — no bone cache, identity matrix + identity basepose.
        let id_ptr = &IDENTITY_MATRIX as *const mdxaBone_t as *mut mdxaBone_t;
        return (IDENTITY_MATRIX, id_ptr, id_ptr);
    };

    let evaluated = cache.eval(bone_num);
    // SAFETY: `cache.header` is the block `EngineHost::model_mdxa` handed
    // back for this cache's model; `bone_num` is caller-provided bone data,
    // matching Raven's own unchecked read (its bounds assert is dead under
    // `-DNDEBUG`).
    let (base_ptr, base_inv_ptr) = unsafe {
        let skel = mdxa_skel_ptr(cache.header as *const c_void, bone_num);
        (
            skel.add(SKEL_OFS_BASE_POSE_MAT) as *mut mdxaBone_t,
            skel.add(SKEL_OFS_BASE_POSE_MAT_INV) as *mut mdxaBone_t,
        )
    };
    let base_pose_mat: mdxaBone_t = unsafe { core::ptr::read_unaligned(base_ptr) };

    // Raven: `Multiply_3x4Matrix(&bolt, (mdxaBone_t*)&boneCache.Eval(boneNum),
    // &skel->BasePoseMat); // DEST FIRST ARG`
    let mut bolt = IDENTITY_MATRIX;
    bone_transform::multiply_3x4_matrix(&mut bolt, &evaluated, &base_pose_mat);

    if scale[0] != 0.0 {
        bolt.matrix[0][3] *= scale[0];
    }
    if scale[1] != 0.0 {
        bolt.matrix[1][3] *= scale[1];
    }
    if scale[2] != 0.0 {
        bolt.matrix[2][3] *= scale[2];
    }
    VectorNormalizeRow(&mut bolt.matrix[0]);
    VectorNormalizeRow(&mut bolt.matrix[1]);
    VectorNormalizeRow(&mut bolt.matrix[2]);

    let mut ret_matrix = IDENTITY_MATRIX;
    bone_transform::multiply_3x4_matrix(&mut ret_matrix, world_matrix, &bolt);

    (ret_matrix, base_ptr, base_inv_ptr)
}

/// Raven `void G2_GetBoneBasepose(CGhoul2Info &ghoul2, int boneNum,
/// mdxaBone_t *&retBasepose, mdxaBone_t *&retBaseposeInv)` — a pure read of
/// `bone_num`'s `mdxaSkel_t::BasePoseMat`/`BasePoseMatInv` out of the cache's
/// `header` (no `Eval`, so no cache mutation); the no-bone-cache path aliases
/// the identity matrix instead ("yikes", Raven's own comment). Every path
/// writes both outputs, so returned by value per §C7 (no failure signal).
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:656-676`
pub fn g2_get_bone_basepose(
    g2: &Ghoul2System,
    ghoul2: &CGhoul2Info,
    bone_num: i32,
) -> (*mut mdxaBone_t, *mut mdxaBone_t) {
    let Some(cache) = ghoul2.bone_cache.and_then(|id| g2.bone_caches.get(id)) else {
        let id_ptr = &IDENTITY_MATRIX as *const mdxaBone_t as *mut mdxaBone_t;
        return (id_ptr, id_ptr);
    };
    // SAFETY: see `g2_get_bone_matrix_low`.
    unsafe {
        let skel = mdxa_skel_ptr(cache.header as *const c_void, bone_num);
        (
            skel.add(SKEL_OFS_BASE_POSE_MAT) as *mut mdxaBone_t,
            skel.add(SKEL_OFS_BASE_POSE_MAT_INV) as *mut mdxaBone_t,
        )
    }
}

/// Raven `void G2_RagGetBoneBasePoseMatrixLow(CGhoul2Info &ghoul2, int
/// boneNum, mdxaBone_t &boneMatrix, mdxaBone_t &retMatrix, vec3_t scale)` —
/// the ragdoll-solver variant: multiplies the caller-supplied `bone_matrix`
/// (an input, not an out-param — Raven's non-`const` reference is read-only in
/// the body) by `bone_num`'s basepose matrix (`Multiply_3x4Matrix`), applies
/// `scale` to the translation column, and re-normalizes the rotation basis.
/// Pure read of the cache's `header` (no `Eval`, no mutation), unconditionally
/// written, so returned by value per §C7. Called from the RagDoll solver
/// (`G2_bones.cpp:3385`), hence `pub` for `ragdoll.rs`.
///
/// Divergence (§19): Raven asserts `ghoul2.mBoneCache` non-null
/// unconditionally (no other guard) and that assert is a compiled no-op under
/// `-DNDEBUG` — a genuinely absent cache is oracle UB (a null deref) that no
/// live caller reaches. This port picks the defined identity-matrix fallback
/// instead of a null-pointer crash.
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:696-725`
pub fn g2_rag_get_bone_base_pose_matrix_low(
    g2: &Ghoul2System,
    ghoul2: &CGhoul2Info,
    bone_num: i32,
    bone_matrix: &mdxaBone_t,
    scale: vec3_t,
) -> mdxaBone_t {
    let Some(cache) = ghoul2.bone_cache.and_then(|id| g2.bone_caches.get(id)) else {
        return IDENTITY_MATRIX;
    };

    let mut ret_matrix = IDENTITY_MATRIX;
    // SAFETY: see `g2_get_bone_matrix_low`.
    let base_pose_mat: mdxaBone_t = unsafe {
        let skel = mdxa_skel_ptr(cache.header as *const c_void, bone_num);
        core::ptr::read_unaligned(skel.add(SKEL_OFS_BASE_POSE_MAT) as *const mdxaBone_t)
    };
    bone_transform::multiply_3x4_matrix(&mut ret_matrix, bone_matrix, &base_pose_mat);

    if scale[0] != 0.0 {
        ret_matrix.matrix[0][3] *= scale[0];
    }
    if scale[1] != 0.0 {
        ret_matrix.matrix[1][3] *= scale[1];
    }
    if scale[2] != 0.0 {
        ret_matrix.matrix[2][3] *= scale[2];
    }
    VectorNormalizeRow(&mut ret_matrix.matrix[0]);
    VectorNormalizeRow(&mut ret_matrix.matrix[1]);
    VectorNormalizeRow(&mut ret_matrix.matrix[2]);

    ret_matrix
}

/// Raven `static void G2_Sort_Models(CGhoul2Info_v &ghoul2, int * const
/// modelList, int * const modelCount)` (private helper, not named in this
/// file's roster row — module-doc gap note) — "sort all the ghoul models in
/// this list so if they go in reference order[,] ... ensur[ing] the model
/// being attached to is built and rendered first": a breadth-first walk
/// starting from every parentless (`mModelBoltLink == -1`) valid model,
/// inserting each bolt-child once its bolt-parent is already in the list.
/// Ported as a returned `Vec<i32>` (the sorted model-index list; Raven's
/// `modelCount` is its `len()`) rather than a fixed `[i32; 256]` + out-param
/// count, per §C7 (internal-only, no ABI surface, §A1 latitude).
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:2877-2950`
fn g2_sort_models(g2: &Ghoul2System, ghoul2: &CGhoul2Info_v) -> Vec<i32> {
    let models = g2.info_array.get(ghoul2.mItem);
    let mut model_list = Vec::new();

    // First walk all the possible ghoul2 models, and stuff the out array with
    // those with no parents (`:2888-2908`).
    for (i, m) in models.iter().enumerate() {
        if m.modelindex == -1 || !m.valid {
            continue;
        }
        if m.model_bolt_link == -1 {
            model_list.push(i as i32);
        }
    }

    // Now, using that list of parentless models, walk the descendant tree for
    // each of them, inserting the descendants in the list (`:2910-2949`).
    let mut start = 0usize;
    let mut end = model_list.len();
    while start != end {
        for (i, m) in models.iter().enumerate() {
            if m.modelindex == -1 || !m.valid {
                continue;
            }
            if m.model_bolt_link != -1 {
                let bolt_to = (m.model_bolt_link >> MODEL_SHIFT) & MODEL_AND;
                if model_list[start..end].contains(&bolt_to) {
                    model_list.push(i as i32);
                }
            }
        }
        start = end;
        end = model_list.len();
    }

    model_list
}

/// Raven `static void RootMatrix(CGhoul2Info_v &ghoul2, int time, const
/// vec3_t scale, mdxaBone_t &retMatrix)` (private helper, not named in this
/// file's roster row — module-doc gap note) — finds the first valid model
/// flagged `GHOUL2_NEWORIGIN`, recursively (re)builds the whole skeleton
/// (`g2_construct_ghoul_skeleton(..., check_for_new_origin: false, ...)`) so
/// its bolt is current, reads that model's new-origin bolt matrix
/// (`g2_get_bolt_matrix_low`), and returns the translation-only matrix that
/// re-origins everything around it; falls back to the identity matrix when no
/// model has the flag. Always writes, so returned by value per §C7.
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:3333-3366`
fn root_matrix(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v,
    time: i32,
    scale: vec3_t,
) -> mdxaBone_t {
    let item = ghoul2.mItem;
    let count = g2.info_array.get(item).len();

    for i in 0..count {
        let (model_index, valid, flags, new_origin) = {
            let m = &g2.info_array.get(item)[i];
            (m.modelindex, m.valid, m.flags, m.new_origin)
        };

        if model_index != -1 && valid && (flags & GHOUL2_NEWORIGIN) != 0 {
            // Raven: `G2_ConstructGhoulSkeleton(ghoul2,time,false,scale);`
            g2_construct_ghoul_skeleton(g2, host, ghoul2, time, false, scale);

            // Re-derive after the recursive construct above (it may have
            // rebuilt bone caches) — module-doc aliasing finding: this needs
            // a DIFFERENT model's cache while `g2` is otherwise free, so it
            // routes through the alias-free `resolve_bolt_matrix_low` rather
            // than the pinned `g2_get_bolt_matrix_low`.
            let bolt = {
                let Ghoul2System {
                    info_array,
                    bone_caches,
                    ..
                } = &mut *g2;
                let parent = &info_array.get(item)[i];
                resolve_bolt_matrix_low(
                    bone_caches,
                    host,
                    parent.bone_cache,
                    &parent.slist,
                    &parent.bltlist,
                    new_origin,
                    scale,
                )
            };

            // Raven:
            // tempMatrix = { {1,0,0,-bolt[0][3]}, {0,1,0,-bolt[1][3]}, {0,0,1,-bolt[2][3]} };
            // Multiply_3x4Matrix(&retMatrix, &tempMatrix, (mdxaBone_t*)&identityMatrix);
            let temp_matrix = mdxaBone_t {
                matrix: [
                    [1.0, 0.0, 0.0, -bolt.matrix[0][3]],
                    [0.0, 1.0, 0.0, -bolt.matrix[1][3]],
                    [0.0, 0.0, 1.0, -bolt.matrix[2][3]],
                ],
            };
            let mut ret_matrix = IDENTITY_MATRIX;
            bone_transform::multiply_3x4_matrix(&mut ret_matrix, &temp_matrix, &IDENTITY_MATRIX);
            return ret_matrix;
        }
    }

    // Raven: `retMatrix=identityMatrix;` (no model has `GHOUL2_NEWORIGIN`).
    IDENTITY_MATRIX
}

/// Raven `void G2_GetBoltMatrixLow(CGhoul2Info &ghoul2, int boltNum, const
/// vec3_t scale, mdxaBone_t &retMatrix)` (cross-TU-shared, no roster row
/// anywhere in this doc — module-doc gap note, stubbed here as the blocking
/// dependency of `g2_construct_ghoul_skeleton`/`root_matrix`; also called from
/// the ragdoll solver, `G2_bones.cpp:3423`, and `G2API_GetBoltMatrix`,
/// `G2_API.cpp:1839` — a permanent home is this doc's open item, not settled
/// by this stub) — resolves bolt `bolt_num`'s matrix: bone-attached bolts
/// evaluate the bone unsmoothed (`CBoneCache::EvalUnsmooth`) premultiplied by
/// its basepose; surface-attached bolts delegate to the surface-bolt
/// transform (`g2_process_surface_bolt2`, this file's private helper); an
/// unattached bolt (neither bone nor surface) falls back to the identity
/// matrix. Always writes, so returned by value per §C7.
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:3253-3331`
pub fn g2_get_bolt_matrix_low(
    g2: &mut Ghoul2System,
    host: &mut impl EngineHost,
    ghoul2: &CGhoul2Info,
    bolt_num: i32,
    scale: vec3_t,
) -> mdxaBone_t {
    resolve_bolt_matrix_low(
        &mut g2.bone_caches,
        host,
        ghoul2.bone_cache,
        &ghoul2.slist,
        &ghoul2.bltlist,
        bolt_num,
        scale,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::cghoul2_info_v::CGhoul2Info_v;
    use mp_host_interface::mock::MockHost;

    /// Builds a synthetic `mdxaHeader_t` + `mdxaSkelOffsets_t` + `mdxaSkel_t`
    /// byte blob (one bone) matching the layout `MDXA_*`/`SKEL_OFS_*` above
    /// assume, so the raw pointer arithmetic can be checked without any real
    /// `.gla` file or a live `EngineHost`/`CBoneCache::new` body (still
    /// `todo!()` in `render/bone_cache.rs`).
    fn build_test_mdxa(base_pose: mdxaBone_t, base_pose_inv: mdxaBone_t) -> Vec<u8> {
        let mut buf = vec![0u8; MDXA_HEADER_SIZE];
        buf[MDXA_NUM_BONES_OFFSET..MDXA_NUM_BONES_OFFSET + 4].copy_from_slice(&1i32.to_ne_bytes());

        // mdxaSkelOffsets_t: one i32 offset (relative to the base), then the
        // single mdxaSkel_t entry: name[MAX_QPATH] + flags(4) + parent(4) +
        // BasePoseMat(48) + BasePoseMatInv(48).
        let skel_offset = 4i32; // one i32 offsets table entry precedes the skel data
        buf.extend_from_slice(&skel_offset.to_ne_bytes());
        buf.extend_from_slice(&[0u8; MAX_QPATH]); // name
        buf.extend_from_slice(&0i32.to_ne_bytes()); // flags
        buf.extend_from_slice(&(-1i32).to_ne_bytes()); // parent
        for row in base_pose.matrix {
            for v in row {
                buf.extend_from_slice(&v.to_ne_bytes());
            }
        }
        for row in base_pose_inv.matrix {
            for v in row {
                buf.extend_from_slice(&v.to_ne_bytes());
            }
        }
        buf
    }

    #[test]
    fn mdxa_reads_match_the_assumed_layout() {
        let base_pose = mdxaBone_t {
            matrix: [
                [1.0, 2.0, 3.0, 4.0],
                [5.0, 6.0, 7.0, 8.0],
                [9.0, 10.0, 11.0, 12.0],
            ],
        };
        let base_pose_inv = mdxaBone_t {
            matrix: [
                [21.0, 22.0, 23.0, 24.0],
                [25.0, 26.0, 27.0, 28.0],
                [29.0, 30.0, 31.0, 32.0],
            ],
        };
        let buf = build_test_mdxa(base_pose, base_pose_inv);
        let header = buf.as_ptr() as *const c_void;
        unsafe {
            assert_eq!(mdxa_num_bones(header), 1);
            let skel = mdxa_skel_ptr(header, 0);
            let read_base: mdxaBone_t =
                core::ptr::read_unaligned(skel.add(SKEL_OFS_BASE_POSE_MAT) as *const mdxaBone_t);
            let read_base_inv: mdxaBone_t = core::ptr::read_unaligned(
                skel.add(SKEL_OFS_BASE_POSE_MAT_INV) as *const mdxaBone_t,
            );
            assert_eq!(read_base, base_pose);
            assert_eq!(read_base_inv, base_pose_inv);
        }
    }

    /// `resolve_bolt_matrix_low`'s three identity-fallback arms (no bone
    /// cache; out-of-range bolt index; a bolt with neither a bone nor a
    /// surface) never touch `CBoneCache::eval_unsmooth` (still `todo!()` in
    /// `render/bone_cache.rs`), so they're testable in isolation.
    #[test]
    fn resolve_bolt_matrix_low_falls_back_to_identity() {
        let mut bone_caches = BoneCacheArena::default();
        let mut host = MockHost::new();
        let scale = [1.0, 1.0, 1.0];

        // No bone cache at all.
        assert_eq!(
            resolve_bolt_matrix_low(&mut bone_caches, &mut host, None, &[], &[], 0, scale),
            IDENTITY_MATRIX
        );

        // A live cache (built directly via the struct literal — every field is
        // `pub` — bypassing the still-`todo!()` `CBoneCache::new` ctor), but an
        // out-of-range bolt index.
        let id = bone_caches.insert(test_bone_cache());
        assert_eq!(
            resolve_bolt_matrix_low(&mut bone_caches, &mut host, Some(id), &[], &[], 0, scale),
            IDENTITY_MATRIX
        );

        // A bolt with neither a bone nor a surface (`boneNumber`/`surfaceNumber`
        // both negative).
        let bltlist = [boltInfo_t {
            boneNumber: -1,
            surfaceNumber: -1,
            surfaceType: 0,
            boltUsed: 0,
            position: IDENTITY_MATRIX,
        }];
        assert_eq!(
            resolve_bolt_matrix_low(
                &mut bone_caches,
                &mut host,
                Some(id),
                &[],
                &bltlist,
                0,
                scale
            ),
            IDENTITY_MATRIX
        );
    }

    /// A minimal, field-literal `CBoneCache` for tests that only exercise
    /// arena bookkeeping / fallback arms, not the still-`todo!()` evaluation
    /// methods.
    fn test_bone_cache() -> CBoneCache {
        CBoneCache {
            frame_size: 0,
            header: core::ptr::null_mut(),
            model: 0,
            bones: Vec::new(),
            final_bones: Vec::new(),
            smooth_bones: Vec::new(),
            root_bone_list: core::ptr::null_mut(),
            root_matrix: IDENTITY_MATRIX,
            incoming_time: 0,
            current_touch: 3,
            current_touch_render: 0,
            last_touch: 2,
            last_last_touch: 1,
            smoothing_active: false,
            unsquash: false,
            smooth_factor: 1.0,
        }
    }

    /// `g2_sort_models` is pure `CGhoul2Info` array logic — no sibling `todo!()`
    /// calls — so the topological-sort behavior is fully testable: a child
    /// bolted to a parent must sort after that parent, and an invalid/
    /// no-model slot is skipped entirely.
    #[test]
    fn g2_sort_models_orders_bolt_children_after_their_parents() {
        let mut g2 = Ghoul2System::default();
        let mut ghoul2 = CGhoul2Info_v { mItem: 0 };
        ghoul2.resize(&mut g2, 3);

        // model 0: parentless root.
        // model 1: bolted to model 0.
        // model 2: no model in this slot (mModelindex == -1) — skipped.
        {
            let models = g2.info_array.get_mut(ghoul2.mItem);
            models[0].modelindex = 0;
            models[0].valid = true;
            models[0].model_bolt_link = -1;

            models[1].modelindex = 1;
            models[1].valid = true;
            models[1].model_bolt_link = 0 << MODEL_SHIFT; // bolted to model 0

            models[2].modelindex = -1;
            models[2].valid = true;
        }

        let sorted = g2_sort_models(&g2, &ghoul2);
        assert_eq!(sorted, vec![0, 1]);
    }
}
