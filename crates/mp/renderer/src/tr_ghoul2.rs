//! Raven `tr_ghoul2.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_ghoul2.cpp`
//!
//! Dropped, `G2_PERFORMANCE_ANALYSIS`-only surface (porting-rules §20 dead
//! surface): `G2Time_ResetTimers` (`:64-76`) and `G2Time_ReportTimers`
//! (`:78-92`) touch nothing but the `G2Time_*`/`G2PerformanceCounter_*`
//! instrumentation globals, and `G2_PERFORMANCE_ANALYSIS` is compiled only
//! `#ifndef FINAL_BUILD` (`oracle/codemp/game/q_shared.h:44-46`) — retail
//! compiles both functions' bodies (and every one of their call sites
//! throughout this file) out entirely. Not transcribed; every other ported
//! function below likewise drops its own `#ifdef G2_PERFORMANCE_ANALYSIS`
//! timer touches per the same ruling (DEC-37 A13.5) without a per-site note.

use mp_qshared::shared::q_math::{
    _DotProduct, _VectorMA, _VectorSubtract, CrossProduct, DotProductRow, VectorNormalize2,
};
use mp_qshared::shared::{mdxaBone_t, vec3_t, VectorNormalize};

use mp_host_interface::mdx::mdxa::MdxaRef;
use mp_host_interface::mdx::mdxm::{MdxmSurfaceView, MdxmVertView, MdxmView};
use mp_host_interface::EngineHost;

// USER RULING (DEC-32 one-home): every bone-evaluation type/function below is
// consumed from `mp_engine_ghoul2` (the DEC-35 canonical port of the very same
// `tr_ghoul2.cpp` definitions), never re-declared in this crate.
use mp_engine_ghoul2::ghoul2_system::{BoneCacheArena, BoneCacheId};
use mp_engine_ghoul2::misc::g2_setup_model_pointers;
use mp_engine_ghoul2::shared::bolt_info_t::boltInfo_t;
use mp_engine_ghoul2::shared::bone_info_t::boneInfo_t;
use mp_engine_ghoul2::shared::cghoul2_info::CGhoul2Info;
use mp_engine_ghoul2::shared::surface_info_t::surfaceInfo_t;

use crate::mdx_format::mdxm_vertex_t::mdxmVertex_t;
use crate::render_state::frame_state::FrameState;
use crate::render_state::model_asset::ModelHandle;
use crate::render_state::placeholders::RefEntity;
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::shader_asset::ShaderHandle;
use crate::render_state::skin_asset::SkinHandle;
use crate::tr_local::crenderable_surface::CRenderableSurface;
use crate::tr_local::model_s::model_t;
use crate::tr_model::frontend::mdxm_view_of;

// ---------------------------------------------------------------------------
// Confirmed `#define` values (porting-rules §C8: `#define` -> `const`).
//
// This wave's own packet does not carry these definitions (they live in
// `G2.h`/`G2_local.h`/`mdx_format.h`/`G2_bones.cpp`, outside `tr_ghoul2.cpp`),
// but every value below is cross-verified — not guessed (porting-rules §A2) —
// against the already-ported, individually-cited copies this repo already
// carries for the *same* oracle `#define`s in the DEC-35 g2sv crate
// (`crates/mp/engine/ghoul2/src/{bolts,api_bolts,render/skeleton}.rs`),
// which this file's own scope (DEC-37 A13.3: naming *state carriers*, not
// re-deriving constants) does not forbid reusing.
// ---------------------------------------------------------------------------

/// `#define G2SURFACEFLAG_GENERATED 0x00000200` (`mdx_format.h:50`).
const G2SURFACEFLAG_GENERATED: i32 = 0x0000_0200;
/// `#define iG2_TRISIDE_LONGEST 0` (`mdx_format.h:57`).
const IG2_TRISIDE_LONGEST: usize = 0;
/// `#define iG2_TRISIDE_SHORTEST 2` (`mdx_format.h:58`).
const IG2_TRISIDE_SHORTEST: usize = 2;
/// `#define MDX_TAG_ORIGIN 2` (`tr_ghoul2.cpp:2238`).
const MDX_TAG_ORIGIN: usize = 2;
/// `#define iG2_BONEWEIGHT_TOPBITS_SHIFT 12` / `iG2_BONEWEIGHT_TOPBITS_AND
/// 0x300` / `fG2_BONEWEIGHT_RECIPROCAL_MULT (1.0f/1023.0f)` — the vertex
/// bone-weight bit-packing constants (`mdx_format.h:57-66,290-322`), matching
/// `mp_host_interface::mdx::mdxm`'s own private copy of the same values.
const IG2_BONEWEIGHT_TOPBITS_SHIFT: u32 = 12;
const IG2_BONEWEIGHT_TOPBITS_AND: u32 = 0x300;
const FG2_BONEWEIGHT_RECIPROCAL_MULT: f32 = 1.0 / 1023.0;
/// `#define MAX_RENDER_SURFACES (2048)` — the `RSStorage` ring's length
/// (`tr_ghoul2.cpp:865`, four lines above `AllocRS`).
// Its one reader, `alloc_rs`, is still deferred on the ring's state carrier.
#[allow(dead_code)]
const MAX_RENDER_SURFACES: usize = 2048;

// ---------------------------------------------------------------------------
// USER RULING (DEC-32 one home per item): the bone-evaluation surface this
// wave re-ported here — `CBoneCache` (and its `Root`/`Eval`/`EvalUnsmooth`/
// `EvalRender`/`WasRendered`/`GetParent` methods), `SBoneCalc`,
// `CTransformBone`, `CGhoul2Info`, `boneInfo_t`/`surfaceInfo_t`/`boltInfo_t`,
// `EvalBoneCache`, `RemoveBoneCache`, `G2_GetModA`, `G2_GetBoneDependents`,
// `G2_WasBoneRendered`, `G2_GetBoneBasepose`, `G2_TransformGhoulBones`,
// `G2_TimingModel`, `G2_Sort_Models`, `G2_CreateQuaterion`,
// `G2_CreateMatrixFromQuaterion` and `Multiply_3x4Matrix` — is canonically
// ported in `mp_engine_ghoul2` (DEC-35: `render/{bone_cache,bone_transform,
// skeleton}.rs`, `shared/*`, `misc.rs`). Every one of those local copies is
// deleted; this file consumes the canonical items over the crate edge declared
// in `crates/mp/renderer/Cargo.toml`.
//
// `CBoneCache::SetRenderMatrix` is not consumed either: it is `_XBOX`-only and
// the canonical port drops it outright (porting-rules §20).
//
// Four of those canonical items are private to their engine-crate modules
// (`g2_get_mod_a` in `api_ragdoll.rs`; `g2_get_bone_dependents` and
// `g2_was_bone_rendered` in `ragdoll.rs`; `g2_sort_models` in
// `render/skeleton.rs`). No renderer call site needs them yet, so nothing was
// widened; the wave that wires live entity rendering requests the additive
// `pub` there rather than growing a second copy here.
// ---------------------------------------------------------------------------

/// Raven `class CConstructBoneList` — a data holder binding the inputs a
/// (not-yet-ported, later-wave) recursive bone-list construction pass needs.
///
/// Type definition source: `oracle/codemp/renderer/tr_ghoul2.cpp:128-165`
pub struct CConstructBoneList<'a> {
    /// Raven `int surfaceNum`.
    pub surface_num: i32,
    /// Raven `int *boneUsedList`.
    pub bone_used_list: &'a mut [i32],
    /// Raven `surfaceInfo_v &rootSList`.
    pub root_s_list: &'a mut Vec<surfaceInfo_t>,
    /// Raven `model_t *currentModel`.
    pub current_model: Option<ModelHandle>,
    /// Raven `boneInfo_v &boneList`.
    pub bone_list: &'a mut Vec<boneInfo_t>,
}

impl<'a> CConstructBoneList<'a> {
    /// Raven `CConstructBoneList::CConstructBoneList(...)` — a plain
    /// initializer-list ctor, no logic.
    ///
    /// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:151-162`
    pub fn new(
        surface_num: i32,
        bone_used_list: &'a mut [i32],
        root_s_list: &'a mut Vec<surfaceInfo_t>,
        current_model: Option<ModelHandle>,
        bone_list: &'a mut Vec<boneInfo_t>,
    ) -> Self {
        CConstructBoneList {
            surface_num,
            bone_used_list,
            root_s_list,
            current_model,
            bone_list,
        }
    }
}

/// Raven `class CRenderSurface` — a per-surface render data holder built
/// fresh each frame from the render-thread-local scratch it borrows.
/// `_G2_GORE` is ON in this build (matching the DEC-35 g2sv build config),
/// so the gore-carrying ctor arm is transcribed. `CGoreSet *initgore_set` has
/// no ported home anywhere in this crate yet (the gore subsystem's own wave);
/// kept as an opaque index placeholder.
///
/// Type definition source: `oracle/codemp/renderer/tr_ghoul2.cpp:800-864`
pub struct CRenderSurface<'a> {
    /// Raven `int surfaceNum`.
    pub surface_num: i32,
    /// Raven `surfaceInfo_v &rootSList`.
    pub root_s_list: &'a [surfaceInfo_t],
    /// Raven `shader_t *cust_shader`.
    pub cust_shader: Option<ShaderHandle>,
    /// Raven `int fogNum`.
    pub fog_num: i32,
    /// Raven `qboolean personalModel` (`bool` per §C7).
    pub personal_model: bool,
    /// Raven `CBoneCache *boneCache` — the canonical generational handle into
    /// `Ghoul2System.bone_caches` (DEC-35 `G2SV-D9`) rather than an aliasing
    /// borrow: the render pass that reads this field evaluates bones, which
    /// needs `&mut CBoneCache` out of the arena (§B5).
    pub bone_cache: Option<BoneCacheId>,
    /// Raven `int renderfx`.
    pub renderfx: i32,
    /// Raven `skin_t *skin`.
    pub skin: Option<SkinHandle>,
    /// Raven `model_t *currentModel`.
    pub current_model: Option<ModelHandle>,
    /// Raven `int lod`.
    pub lod: i32,
    /// Raven `boltInfo_v &boltList`.
    pub bolt_list: &'a mut [boltInfo_t],
    /// Raven `shader_t *gore_shader` (`_G2_GORE`).
    pub gore_shader: Option<ShaderHandle>,
    /// Raven `CGoreSet *gore_set` (`_G2_GORE`) — PORT-NOTE: opaque index
    /// placeholder; `CGoreSet` has no ported home in this crate yet.
    pub gore_set: Option<u32>,
}

impl<'a> CRenderSurface<'a> {
    /// Raven `CRenderSurface::CRenderSurface(...)` — a plain initializer-list
    /// ctor, no logic.
    ///
    /// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:825-861`
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        surface_num: i32,
        root_s_list: &'a [surfaceInfo_t],
        cust_shader: Option<ShaderHandle>,
        fog_num: i32,
        personal_model: bool,
        bone_cache: Option<BoneCacheId>,
        renderfx: i32,
        skin: Option<SkinHandle>,
        current_model: Option<ModelHandle>,
        lod: i32,
        bolt_list: &'a mut [boltInfo_t],
        gore_shader: Option<ShaderHandle>,
        gore_set: Option<u32>,
    ) -> Self {
        CRenderSurface {
            surface_num,
            root_s_list,
            cust_shader,
            fog_num,
            personal_model,
            bone_cache,
            renderfx,
            skin,
            current_model,
            lod,
            bolt_list,
            gore_shader,
            gore_set,
        }
    }
}

/// Raven `int G2_Find_Bone_ByNum(const model_t *mod, boneInfo_v &blist,
/// const int boneNum)`. `mod` is unused in the oracle body (kept in the
/// signature for call-site parity with future waves).
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:112-126`
pub fn g2_find_bone_by_num(
    _model: Option<ModelHandle>,
    blist: &[boneInfo_t],
    bone_num: i32,
) -> i32 {
    for (i, bone) in blist.iter().enumerate() {
        if bone.boneNumber == bone_num {
            return i as i32;
        }
    }
    -1
}

/// Raven `char *G2_GetBoneNameFromSkel(CGhoul2Info &ghoul2, int boneNum)`.
/// `char*` return -> `Option<String>` (translation dictionary: `NULL` ->
/// `None`, `char*` -> owned `String`). Raven's `ghoul2.mBoneCache` deref
/// becomes the canonical [`BoneCacheId`] lookup (DEC-35 `G2SV-D9`); the arena
/// is taken as its own parameter rather than the whole `Ghoul2System` so a
/// caller holding a `&CGhoul2Info` borrowed out of that same system can still
/// call this (the disjoint-borrow shape `g2_transform_ghoul_bones_inner`
/// already uses in `mp_engine_ghoul2`).
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:678-694`
pub fn g2_get_bone_name_from_skel(
    bone_caches: &BoneCacheArena,
    ghoul2: &CGhoul2Info,
    bone_num: i32,
) -> Option<String> {
    let bone_cache = ghoul2.bone_cache.and_then(|id| bone_caches.get(id))?;
    let mdxa = bone_cache
        .mdxa
        .expect("G2_GetBoneNameFromSkel: bone cache has no mdxa header");
    debug_assert!(bone_num >= 0 && bone_num < mdxa.num_bones());
    Some(mdxa.skel(bone_num).name.clone())
}

/// Raven `CRenderableSurface *AllocRS(void)` — hands out the next
/// render-thread-local `CRenderableSurface` scratch slot from a
/// `RSStorage[MAX_RENDER_SURFACES]` ring buffer.
// DEFERRED: AllocRS — the `RSStorage[MAX_RENDER_SURFACES]` ring and its
// `NextRS` cursor need a render-thread state carrier named (DEC-37 A13.3);
// Raven hands out a pointer *into* that ring, which the by-value signature
// below cannot express. The cap itself is resolved (see `MAX_RENDER_SURFACES`
// above).
// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:865-876`
pub fn alloc_rs() -> CRenderableSurface {
    todo!("Port AllocRS — oracle/codemp/renderer/tr_ghoul2.cpp:869-876")
}

/// Raven `static int R_GComputeFogNum(trRefEntity_t *ent)` — the fog volume
/// `ent` falls inside, if any.
// DEFERRED: R_GComputeFogNum — depends on `RenderAssets::world`
// (`WorldAsset`, an empty placeholder struct pending the `tr_bsp` wave's
// `numfogs`/`fogs` fields) and `FrameState::refdef` (`TrRefdef`, empty
// pending the `tr_scene` wave's `rdflags` field) — see
// `crates/mp/renderer/src/render_state/placeholders.rs`, out of this file's
// edit scope. A state home this packet marks mapped-but-not-yet-populated is
// an escalation, not an invention (preamble "state home ... ESCALATION").
// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:939-964`
pub fn r_g_compute_fog_num(_ent: &RefEntity, _assets: &RenderAssets, _frame: &FrameState) -> i32 {
    todo!("Port R_GComputeFogNum — oracle/codemp/renderer/tr_ghoul2.cpp:939-964")
}

/// Raven `static inline bool bInShadowRange(vec3_t location)`.
// DEFERRED: bInShadowRange — depends on `FrameState::view`/`FrameState::ori`
// (`ViewParms`/`OrientationR`, both empty placeholder structs pending the
// `tr_main` wave's `ori.axis`/`ori.origin` fields) and the `r_shadowRange`
// cvar, whose owner this packet could not confirm ("NOT this TU's state ...
// confirm the owner at port time") — see
// `crates/mp/renderer/src/render_state/placeholders.rs`, out of this file's
// edit scope.
// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:3369-3376`
pub fn b_in_shadow_range(
    _location: vec3_t,
    _assets: &RenderAssets,
    _frame: &FrameState,
    _r_shadow_range: f32,
) -> bool {
    todo!("Port bInShadowRange — oracle/codemp/renderer/tr_ghoul2.cpp:3369-3376")
}

/// Raven `bool G2_NeedsRecalc(CGhoul2Info *ghlInfo, int frameNum)`. Raven's
/// `ghlInfo->mBoneCache->mod != ghlInfo->currentModel` compares the cache's
/// stored model against the instance's; the canonical `CBoneCache` retypes
/// Raven's `const model_t *mod` to the `qhandle_t` it was built from
/// (`G2SV-D5`) and `g2_transform_ghoul_bones` stores `ghoul2.model` into it, so
/// the same staleness test is `cache.model != ghl_info.model`. The
/// `_G2_LISTEN_SERVER_OPT` arm (`G2API_OverrideServerWithClientData`) is
/// compiled out — that macro is OFF in this build config (`G2SV-D4`), the same
/// ruling under which `mp_engine_ghoul2` drops the rest of its surface.
/// The arena is a separate parameter for the reason given on
/// [`g2_get_bone_name_from_skel`].
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:3544-3563`
pub fn g2_needs_recalc(
    bone_caches: &BoneCacheArena,
    host: &mut impl EngineHost,
    ghl_info: &mut CGhoul2Info,
    frame_num: i32,
) -> bool {
    g2_setup_model_pointers(host, ghl_info);
    // not sure if I still need this test, probably
    let stale_cache = match ghl_info.bone_cache.and_then(|id| bone_caches.get(id)) {
        None => true,
        Some(cache) => cache.model != ghl_info.model,
    };
    if ghl_info.skel_frame_num != frame_num || stale_cache {
        ghl_info.skel_frame_num = frame_num;
        return true;
    }
    false
}

/// Raven `static int G2_GetBonePoolIndex(const mdxaHeader_t *pMDXAHeader, int
/// iFrame, int iBone)` — the compressed-bone pool index for `<frame, bone>`,
/// AND'd to 24 bits. Identical logic to [`MdxaRef::frame_bone_pool_index`]
/// (whose own doc comment cites this exact function); delegated rather than
/// re-duplicated, matching that fn's already-verified byte arithmetic.
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:1148-1155`
pub fn g2_get_bone_pool_index(p_mdxa_header: MdxaRef, i_frame: i32, i_bone: i32) -> i32 {
    p_mdxa_header.frame_bone_pool_index(i_frame, i_bone)
}

/// Transform one `mdxmVertex_t` into model space by its weighted bones — the
/// per-vertex accumulation loop `G2_ProcessSurfaceBolt` runs at each of its
/// four call sites (`tr_ghoul2.cpp:2288-2302` and its siblings).
fn g2_process_surface_bolt_transform(
    bone_ptr: &[(i32, mdxaBone_t)],
    vert: MdxmVertView,
    surface: MdxmSurfaceView,
) -> vec3_t {
    let mut p = [0.0f32; 3];
    let vert_coords = vert.vert_coords();
    let num_weights = vert.num_weights();
    let mut total_weight = 0.0f32;
    for k in 0..num_weights {
        let bone_index = vert.bone_index(k);
        let bone_weight = vert.bone_weight(k, &mut total_weight, num_weights);
        let bone_ref = surface.bone_ref(bone_index) as usize;
        let m = &bone_ptr[bone_ref].1;
        p[0] += bone_weight * (DotProductRow(&m.matrix[0], vert_coords) + m.matrix[0][3]);
        p[1] += bone_weight * (DotProductRow(&m.matrix[1], vert_coords) + m.matrix[1][3]);
        p[2] += bone_weight * (DotProductRow(&m.matrix[2], vert_coords) + m.matrix[2][3]);
    }
    p
}

/// Raven `void G2_ProcessSurfaceBolt(mdxaBone_v &bonePtr, mdxmSurface_t
/// *surface, int boltNum, boltInfo_v &boltList, surfaceInfo_t *surfInfo,
/// model_t *mod)`. `mdxaBone_v` (`vector<pair<int,mdxaBone_t>>`) ->
/// `&[(i32, mdxaBone_t)]`; `mdxmSurface_t*` -> [`MdxmSurfaceView`] (the
/// already-ported safe byte view replacing the raw pointer arithmetic);
/// `G2_FindSurface` (this wave's packet flagged it "NOT RESOLVED ...  confirm
/// before use") is confirmed here as [`MdxmView::find_surface`] — that fn's
/// own doc comment names both `G2_FindSurface`/`G2_FindSurface_BC` as the
/// oracle functions it replaces (`mdx_format.h:199-212`).
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:2246-2491`
pub fn g2_process_surface_bolt(
    bone_ptr: &[(i32, mdxaBone_t)],
    surface: MdxmSurfaceView,
    bolt_num: i32,
    bolt_list: &mut [boltInfo_t],
    surf_info: Option<&surfaceInfo_t>,
    mod_: &model_t,
) {
    let n = bolt_num as usize;
    // now there are two types of tag surface - model ones and procedural
    // generated types - lets decide which one we have here.
    if let Some(surf_info) = surf_info.filter(|s| s.offFlags == G2SURFACEFLAG_GENERATED) {
        let surf_number = surf_info.genPolySurfaceIndex & 0x0ffff;
        let poly_number = (surf_info.genPolySurfaceIndex >> 16) & 0x0ffff;

        // find original surface our original poly was in.
        let mdxm_view = mdxm_view_of(mod_);
        let original_surf = mdxm_view.find_surface(surf_number, surf_info.genLod);

        // get the original polys indexes
        let [index0, index1, index2] = original_surf.triangle(poly_number);

        // now go and transform just the points we need from the surface that
        // was hit originally
        let p_tri = [
            g2_process_surface_bolt_transform(bone_ptr, original_surf.vert(index0), original_surf),
            g2_process_surface_bolt_transform(bone_ptr, original_surf.vert(index1), original_surf),
            g2_process_surface_bolt_transform(bone_ptr, original_surf.vert(index2), original_surf),
        ];

        // work out baryCentricK
        let bary_centric_k = 1.0 - (surf_info.genBarycentricI + surf_info.genBarycentricJ);

        // now we have the model transformed into model space, now generate an
        // origin.
        bolt_list[n].position.matrix[0][3] = (p_tri[0][0] * surf_info.genBarycentricI)
            + (p_tri[1][0] * surf_info.genBarycentricJ)
            + (p_tri[2][0] * bary_centric_k);
        bolt_list[n].position.matrix[1][3] = (p_tri[0][1] * surf_info.genBarycentricI)
            + (p_tri[1][1] * surf_info.genBarycentricJ)
            + (p_tri[2][1] * bary_centric_k);
        bolt_list[n].position.matrix[2][3] = (p_tri[0][2] * surf_info.genBarycentricI)
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
        bolt_list[n].position.matrix[0][0] = normal[0];
        bolt_list[n].position.matrix[1][0] = normal[1];
        bolt_list[n].position.matrix[2][0] = normal[2];

        // up will be towards point 0 of the original triangle.
        // so lets work it out. Vector is hit point - point 0
        let mut up = [
            bolt_list[n].position.matrix[0][3] - p_tri[0][0],
            bolt_list[n].position.matrix[1][3] - p_tri[0][1],
            bolt_list[n].position.matrix[2][3] - p_tri[0][2],
        ];
        // normalise it
        VectorNormalize(&mut up);

        // that's the up vector
        bolt_list[n].position.matrix[0][1] = up[0];
        bolt_list[n].position.matrix[1][1] = up[1];
        bolt_list[n].position.matrix[2][1] = up[2];

        // right is always straight
        let mut right = [0.0f32; 3];
        CrossProduct(normal, up, &mut right);
        // that's the up vector
        bolt_list[n].position.matrix[0][2] = right[0];
        bolt_list[n].position.matrix[1][2] = right[1];
        bolt_list[n].position.matrix[2][2] = right[2];
    } else {
        // no, we are looking at a normal model tag
        // whip through and actually transform each vertex
        let mut p_tri = [[0.0f32; 3]; 3];
        for (j, slot) in p_tri.iter_mut().enumerate() {
            *slot = g2_process_surface_bolt_transform(bone_ptr, surface.vert(j as i32), surface);
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
        let a0 = axes[0];
        _VectorMA(a0, -d, axes[1], &mut axes[0]);
        let a0b = axes[0];
        VectorNormalize2(a0b, &mut axes[0]);

        CrossProduct(
            sides[IG2_TRISIDE_LONGEST],
            sides[IG2_TRISIDE_SHORTEST],
            &mut axes[2],
        );
        let a2 = axes[2];
        VectorNormalize2(a2, &mut axes[2]);

        // set up location in world space of the origin point in out going
        // matrix
        bolt_list[n].position.matrix[0][3] = p_tri[MDX_TAG_ORIGIN][0];
        bolt_list[n].position.matrix[1][3] = p_tri[MDX_TAG_ORIGIN][1];
        bolt_list[n].position.matrix[2][3] = p_tri[MDX_TAG_ORIGIN][2];

        // copy axis to matrix - do some magic to orient minus Y to positive X
        // and so on so bolt on stuff is oriented correctly
        bolt_list[n].position.matrix[0][0] = axes[1][0];
        bolt_list[n].position.matrix[0][1] = axes[0][0];
        bolt_list[n].position.matrix[0][2] = -axes[2][0];

        bolt_list[n].position.matrix[1][0] = axes[1][1];
        bolt_list[n].position.matrix[1][1] = axes[0][1];
        bolt_list[n].position.matrix[1][2] = -axes[2][1];

        bolt_list[n].position.matrix[2][0] = axes[1][2];
        bolt_list[n].position.matrix[2][1] = axes[0][2];
        bolt_list[n].position.matrix[2][2] = -axes[2][2];
    }
}

/// Raven `void *G2_FindSurface_BC(const model_s *mod, int index, int lod)`.
/// `void*` return -> [`MdxmSurfaceView`] (the already-ported safe byte view
/// this exact walk already implements — `mdx_format.h:199-212`'s doc comment
/// names both `G2_FindSurface`/`G2_FindSurface_BC` as the functions it
/// replaces).
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:2952-2978`
pub fn g2_find_surface_bc<'a>(model: &'a model_t, index: i32, lod: i32) -> MdxmSurfaceView<'a> {
    let view: MdxmView<'a> = mdxm_view_of(model);
    view.find_surface(index, lod)
}

/// Raven `static inline float G2_GetVertBoneWeightNotSlow(const
/// mdxmVertex_t *pVert, const int iWeightNum)`.
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:3628-3639`
pub fn g2_get_vert_bone_weight_not_slow(p_vert: &mdxmVertex_t, i_weight_num: i32) -> f32 {
    let mut i_temp = p_vert.BoneWeightings[i_weight_num as usize] as u32;
    i_temp |= (p_vert.uiNmWeightsAndBoneIndexes
        >> (IG2_BONEWEIGHT_TOPBITS_SHIFT + (i_weight_num as u32 * 2)))
        & IG2_BONEWEIGHT_TOPBITS_AND;
    FG2_BONEWEIGHT_RECIPROCAL_MULT * i_temp as f32
}
