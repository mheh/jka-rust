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
    _DotProduct, _VectorMA, _VectorSubtract, vec3_origin, CrossProduct, DotProductRow,
    VectorNormalize2,
};
use mp_qshared::shared::{cplane_t, mdxaBone_t, vec3_t, VectorNormalize};

use mp_host_interface::mdx::mdxa::{MdxaRef, MdxaView};
use mp_host_interface::mdx::mdxm::{MdxmSurfaceView, MdxmVertView, MdxmView};
use mp_host_interface::EngineHost;

// USER RULING (DEC-32 one-home): every bone-evaluation type/function below is
// consumed from `mp_engine_ghoul2` (the DEC-35 canonical port of the very same
// `tr_ghoul2.cpp` definitions), never re-declared in this crate.
use mp_engine_ghoul2::bolts::g2_find_bolt_surface_num;
use mp_engine_ghoul2::bones::{g2_add_bone, g2_find_bone};
use mp_engine_ghoul2::ghoul2_system::{BoneCacheArena, BoneCacheId, Ghoul2System};
use mp_engine_ghoul2::misc::g2_setup_model_pointers;
use mp_engine_ghoul2::render::bone_cache::CBoneCache;
use mp_engine_ghoul2::render::bone_transform::{multiply_3x4_matrix, uncompress_bone};
use mp_engine_ghoul2::render::skeleton::g2_get_bone_matrix_low;
use mp_engine_ghoul2::shared::bolt_info_t::boltInfo_t;
use mp_engine_ghoul2::shared::bone_info_t::boneInfo_t;
use mp_engine_ghoul2::shared::cghoul2_info::CGhoul2Info;
use mp_engine_ghoul2::shared::surface_info_t::surfaceInfo_t;
use mp_engine_ghoul2::surfaces::g2_find_override_surface;

use mp_engine_qcommon::common::Common;

use crate::mdx_format::mdxm_vertex_t::mdxmVertex_t;
use crate::render_state::frame_state::FrameState;
use crate::render_state::model_asset::ModelHandle;
use crate::render_state::placeholders::RefEntity;
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::render_state::shader_asset::ShaderHandle;
use crate::render_state::skin_asset::SkinHandle;
use crate::tr_local::crenderable_surface::CRenderableSurface;
use crate::tr_local::model_s::model_t;
use crate::tr_local::orientationr_t::orientationr_t;
use crate::tr_main::{R_CullLocalPointAndRadius, CULL_CLIP, CULL_IN, CULL_OUT};
use crate::tr_mesh::project_radius;
use crate::tr_model::frontend::mdxm_view_of;
use crate::tr_shade_calc::myftol;

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
/// `#define G2SURFACEFLAG_ISBOLT 0x00000001` (`mdx_format.h:40`).
const G2SURFACEFLAG_ISBOLT: i32 = 0x0000_0001;
/// `#define G2SURFACEFLAG_OFF 0x00000002` (`mdx_format.h:41`) — cross-verified
/// against the already-ported copy in `mp_engine_ghoul2::surfaces`.
const G2SURFACEFLAG_OFF: i32 = 0x0000_0002;
/// `#define G2SURFACEFLAG_NODESCENDANTS 0x00000100` (`mdx_format.h:49`) —
/// cross-verified against the already-ported copy in
/// `mp_engine_ghoul2::misc`/`mp_engine_ghoul2::surfaces`.
const G2SURFACEFLAG_NODESCENDANTS: i32 = 0x0000_0100;
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

// ---------------------------------------------------------------------------
// R3 wave 1 (`tr_ghoul2.wave1.md`).
//
// RECONCILED, NOT RE-PORTED (marker law: "never re-port an already-ported
// fn" — preamble). Four of this wave's eleven assigned fns are already
// canonically ported in `mp_engine_ghoul2` (the DEC-32 one-home surface this
// file's own header already establishes for the rest of `CBoneCache` and
// friends) and are consumed from there rather than re-declared here:
//
// - `SmoothLow` -> `mp_engine_ghoul2::render::bone_cache::CBoneCache::
//   smooth_low` — a *private* method of the canonical `CBoneCache`; nothing
//   in this file needs to call it directly (it's `CBoneCache::EvalRender`'s
//   own internal memoization step), so no re-export is needed either.
// - `G2_GetBoneMatrixLow` ->
//   `mp_engine_ghoul2::render::skeleton::g2_get_bone_matrix_low` (`pub fn`).
// - `G2_RagGetBoneBasePoseMatrixLow` ->
//   `mp_engine_ghoul2::render::skeleton::g2_rag_get_bone_base_pose_matrix_low`
//   (`pub fn`).
// - `UnCompressBone` ->
//   `mp_engine_ghoul2::render::bone_transform::uncompress_bone` (`pub fn`).
//
// None of this wave's other assigned fns call any of the four above (checked
// against the packet's own "in-module callees" digests), so no re-export was
// needed to keep this wave's live call graph closed.
// ---------------------------------------------------------------------------

/// Raven `static int G2_ComputeLOD(trRefEntity_t *ent, const model_t
/// *currentModel, int lodBias)`.
///
/// `ent->e.modelScale`/`ent->e.radius` are threaded as explicit
/// `model_scale`/`radius` parameters rather than read off `RefEntity`: that
/// placeholder (`render_state::placeholders`, out of this file's edit scope)
/// carries only the subset of fields the `tr_light`/`tr_scene` wave-0 slices
/// needed (`origin`, `renderfx`, the lighting fields, ...) and does not yet
/// have `modelScale`/`radius`. A state home this packet marks
/// mapped-but-not-yet-populated is an escalation, not an invention (preamble
/// "state home ... ESCALATION"); threading the two missing scalars in
/// directly avoids inventing a field on a struct outside this file. `ent`
/// itself is still threaded for `.origin`, which the placeholder does carry.
///
/// `r_lodbias`/`r_lodscale`/`r_autolodscalevalue` read through
/// `Common::cvar` (the `RendererCvars`-handle + live-engine-table pattern
/// `tr_light.rs`'s `R_SetupEntityLightingGrid` already established).
/// `ProjectRadius`/`myftol` are the wave-0 in-module callees (cross-file,
/// signatures are LAW); `ProjectRadius` is itself still a wave-0
/// `todo!()` (blocked on `FrameState::view`/`ori`, per its own doc comment)
/// so this fn's call into it inherits that same, already-declared block
/// rather than introducing a new one.
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:967-1041`
pub fn g2_compute_lod(
    ent: &RefEntity,
    model_scale: vec3_t,
    radius: f32,
    current_model: &model_t,
    lod_bias: i32,
    frame: &FrameState,
    common: &Common,
    cvars: &RendererCvars,
) -> i32 {
    // model has only 1 LOD level, skip computations and bias
    if current_model.numLods < 2 {
        return 0;
    }

    let mut lod_bias = lod_bias;
    if common.cvar(cvars.r_lodbias).integer > lod_bias {
        lod_bias = common.cvar(cvars.r_lodbias).integer;
    }

    // scale the radius if need be
    let mut largest_scale = model_scale[0];
    if model_scale[1] > largest_scale {
        largest_scale = model_scale[1];
    }
    if model_scale[2] > largest_scale {
        largest_scale = model_scale[2];
    }
    if largest_scale == 0.0 {
        largest_scale = 1.0;
    }

    // Raven `0.75*largestScale*ent->e.radius` — the unsuffixed `0.75` double
    // literal promotes the whole expression to `double`, narrowed to `float`
    // once at the `ProjectRadius(float, ...)` call boundary (wave-0 ruling
    // 12).
    let scaled_radius = (0.75_f64 * largest_scale as f64 * radius as f64) as f32;
    // we reduce the radius to make the LOD match other model types which use
    // the actual bound box size
    let projected_radius = project_radius(scaled_radius, ent.origin, frame);
    let mut flod;
    if projected_radius != 0.0 {
        let mut lodscale =
            common.cvar(cvars.r_lodscale).value + common.cvar(cvars.r_autolodscalevalue).value;
        if lodscale > 20.0 {
            lodscale = 20.0;
        } else if lodscale < 0.0 {
            lodscale = 0.0;
        }
        flod = 1.0f32 - projected_radius * lodscale;
    } else {
        // object intersects near view plane, e.g. view weapon
        flod = 0.0;
    }
    flod *= current_model.numLods as f32;
    let mut lod = myftol(flod);

    if lod < 0 {
        lod = 0;
    } else if lod >= current_model.numLods {
        lod = current_model.numLods - 1;
    }

    lod += lod_bias;

    if lod >= current_model.numLods {
        lod = current_model.numLods - 1;
    }
    if lod < 0 {
        lod = 0;
    }

    lod
}

/// Raven `void G2_SetUpBolts(mdxaHeader_t *header, CGhoul2Info &ghoul2,
/// mdxaBone_v &bonePtr, boltInfo_v &boltList)`. `ghoul2` is unread by the
/// oracle body (kept in the signature for call-site parity, matching the
/// same-file precedent `g2_find_bone_by_num` set for its own unused `mod`
/// parameter). `header` -> [`MdxaView`] (the safe byte view over the raw
/// `mdxaHeader_t*` this exact walk already needs — no `EngineHost`/parsed
/// sidecar required, matching `g2_find_surface_bc`'s equivalent choice for
/// `mdxmHeader_t*`).
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:2053-2068`
pub fn g2_set_up_bolts(
    header: MdxaView,
    _ghoul2: &CGhoul2Info,
    bone_ptr: &[(i32, mdxaBone_t)],
    bolt_list: &mut [boltInfo_t],
) {
    for bolt in bolt_list.iter_mut() {
        if bolt.boneNumber != -1 {
            // figure out where the bone hirearchy info is
            let skel = header.skel(bolt.boneNumber);
            multiply_3x4_matrix(
                &mut bolt.position,
                &bone_ptr[bolt.boneNumber as usize].1,
                &skel.base_pose_mat(),
            );
        }
    }
}

/// Raven `void G2_ProcessGeneratedSurfaceBolts(CGhoul2Info &ghoul2,
/// mdxaBone_v &bonePtr, model_t *mod_t)`. The `G2_PERFORMANCE_ANALYSIS`
/// timer touches this fn's oracle body brackets its loop in are dropped per
/// this wave's STATE HOMES table (DEC-37 A13.5, dead surface — see this
/// file's module doc comment).
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:2495-2518`
pub fn g2_process_generated_surface_bolts(
    ghoul2: &mut CGhoul2Info,
    bone_ptr: &[(i32, mdxaBone_t)],
    mod_t: &model_t,
) {
    // look through the surfaces off the end of the pre-defined model
    // surfaces
    for i in 0..ghoul2.slist.len() {
        // only look for bolts if we are actually a generated surface, and
        // not just an overriden one
        if ghoul2.slist[i].offFlags & G2SURFACEFLAG_GENERATED != 0 {
            // well alrighty then. Lets see if there is a bolt that is
            // attempting to use it
            let bolt_num =
                g2_find_bolt_surface_num(&ghoul2.bltlist, i as i32, G2SURFACEFLAG_GENERATED);
            // yes - ok, processing time.
            if bolt_num != -1 {
                // Raven passes `NULL` for `surface` here: `surfInfo` (=
                // `&ghoul2.mSlist[i]`) always has `G2SURFACEFLAG_GENERATED`
                // set on this call path, so `g2_process_surface_bolt`'s
                // generated-flag branch never dereferences its `surface`
                // parameter — this throwaway view (surface 0 of `mod_t`,
                // read but not written) is behaviorally inert, matching
                // Raven's null.
                let placeholder_surface = mdxm_view_of(mod_t).find_surface(0, 0);
                g2_process_surface_bolt(
                    bone_ptr,
                    placeholder_surface,
                    bolt_num,
                    &mut ghoul2.bltlist,
                    Some(&ghoul2.slist[i]),
                    mod_t,
                );
            }
        }
    }
}

/// Raven `void RenderSurfaces(CRenderSurface &RS)` — "also ended up just
/// ripping right from SP." The recursive offFlags/child-walk skeleton is
/// live; the surface-visible body (`if (!offFlags) { ... }`,
/// `tr_ghoul2.cpp:2554-2717`) is DEFERRED — four independent, out-of-this-
/// file's-edit-scope blockers stack in that one block:
/// - the default-shader-resolution arm needs `surfInfo->shaderIndex`, which
///   `MdxmSurfHierarchyView` (`crates/mp/host-interface/src/mdx/mdxm.rs`)
///   does not expose (only `shader_first_byte`/`flags`/`parent_index`/
///   `num_children`/`child`);
/// - the skin-shader-match arm needs `SkinAsset`'s `surfaces`/`name`/
///   `shader` fields, and `render_state::skin_asset::SkinAsset` is still the
///   empty `{}` client-rendering placeholder;
/// - the shadow-surface and third-person arms both build a
///   `CRenderableSurface` through [`alloc_rs`] — itself already `todo!()` in
///   this file (the `RSStorage` ring has no state carrier) — and populate
///   its tier-2 raw-pointer fields (`surfaceData: *mut mdxmSurface_t`,
///   `boneCache: *mut c_void`) from a safe `MdxmSurfaceView`/`BoneCacheId`,
///   which needs new unsafe plumbing this wave does not own (R2's own
///   Group-4 table marks `CRenderableSurface`'s replacement shape
///   "re-verify when the ghoul2 render-side integration wave lands" — not
///   this wave);
/// - the `_G2_GORE` arm needs `FindGoreRecord`/`G2API_GetTime` (this
///   packet's own call-surface section flags both "NOT RESOLVED ... confirm
///   before use; escalate, never stub") and `CGoreSet::mGoreRecords`, which
///   `CRenderSurface::gore_set: Option<u32>` (already an opaque
///   placeholder, this file's own earlier doc comment) cannot express.
///
/// `RS.currentModel`/`RS.currentModel->mdxm` non-null asserts are dropped —
/// compiled-out under this build's `-DNDEBUG` (house convention, e.g.
/// `mp_engine_ghoul2::bolts`'s module doc comment).
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:2521-2735`
pub fn render_surfaces(
    rs: &mut CRenderSurface,
    current_model: &model_t,
    assets: &RenderAssets,
    cvars: &RendererCvars,
    common: &Common,
) {
    // back track and get the surfinfo struct for this surface
    let mdxm = mdxm_view_of(current_model);
    let surface = mdxm.find_surface(rs.surface_num, rs.lod);
    let surf_info = mdxm.surf_hierarchy(surface.this_surface_index());

    // see if we have an override surface in the surface list
    let surf_override = g2_find_override_surface(rs.surface_num, rs.root_s_list);

    // really, we should use the default flags for this surface unless it's
    // been overriden
    let off_flags = match surf_override {
        Some(o) => o.offFlags,
        None => surf_info.flags(),
    };

    // if this surface is not off, add it to the shader render list
    if off_flags == 0 {
        let _ = (assets, cvars, common);
        // DEFERRED: RenderSurfaces surface-visible body (see doc comment
        // above).
        // Source: oracle/codemp/renderer/tr_ghoul2.cpp:2554-2717
        todo!(
            "Port RenderSurfaces surface-visible body — shaderIndex accessor / SkinAsset fields / CRenderableSurface tier-2 fields (AllocRS) / _G2_GORE subsystem — oracle/codemp/renderer/tr_ghoul2.cpp:2554-2717"
        );
    }

    // if we are turning off all descendants, then stop this recursion now
    if off_flags & G2SURFACEFLAG_NODESCENDANTS != 0 {
        return;
    }

    // now recursively call for the children
    for i in 0..surf_info.num_children() {
        rs.surface_num = surf_info.child(i);
        render_surfaces(rs, current_model, assets, cvars, common);
    }
}

/// Raven `void ProcessModelBoltSurfaces(int surfaceNum, surfaceInfo_v
/// &rootSList, mdxaBone_v &bonePtr, model_t *currentModel, int lod,
/// boltInfo_v &boltList)`.
///
/// `lod` is a dead parameter in the oracle body too (`G2_FindSurface` is
/// always called with a hardcoded `0`, not `lod`); kept only so the
/// recursive call keeps passing it down, matching Raven.
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:2739-2792`
pub fn process_model_bolt_surfaces(
    surface_num: i32,
    root_s_list: &[surfaceInfo_t],
    bone_ptr: &[(i32, mdxaBone_t)],
    current_model: &model_t,
    lod: i32,
    bolt_list: &mut [boltInfo_t],
) {
    // back track and get the surfinfo struct for this surface
    let mdxm = mdxm_view_of(current_model);
    let surface = mdxm.find_surface(surface_num, 0);
    let surf_info = mdxm.surf_hierarchy(surface.this_surface_index());

    // see if we have an override surface in the surface list
    let surf_override = g2_find_override_surface(surface_num, root_s_list);

    // really, we should use the default flags for this surface unless it's
    // been overriden
    let off_flags = match surf_override {
        Some(o) => o.offFlags,
        None => surf_info.flags(),
    };

    // is this surface considered a bolt surface?
    if surf_info.flags() & G2SURFACEFLAG_ISBOLT != 0 {
        // well alrighty then. Lets see if there is a bolt that is
        // attempting to use it
        let bolt_num = g2_find_bolt_surface_num(bolt_list, surface_num, 0);
        // yes - ok, processing time.
        if bolt_num != -1 {
            g2_process_surface_bolt(
                bone_ptr,
                surface,
                bolt_num,
                bolt_list,
                surf_override,
                current_model,
            );
        }
    }

    // if we are turning off all descendants, then stop this recursion now
    if off_flags & G2SURFACEFLAG_NODESCENDANTS != 0 {
        return;
    }

    // now recursively call for the children
    for i in 0..surf_info.num_children() {
        process_model_bolt_surfaces(
            surf_info.child(i),
            root_s_list,
            bone_ptr,
            current_model,
            lod,
            bolt_list,
        );
    }
}

/// Raven `void G2_ConstructUsedBoneList(CConstructBoneList &CBL)`. The
/// offFlags/child-walk skeleton is live; the bone-marking body
/// (`if (!(offFlags & G2SURFACEFLAG_OFF)) { ... }`,
/// `tr_ghoul2.cpp:2821-2860`) is DEFERRED — three stacked, out-of-scope
/// blockers:
/// - `surface->numBoneReferences` — `MdxmSurfaceView`
///   (`crates/mp/host-interface/src/mdx/mdxm.rs`) exposes `bone_ref(i)` but
///   no bone-reference *count* accessor;
/// - `G2BONEFLAG_ALWAYSXFORM` — not in this wave's packet, not
///   cross-verifiable anywhere in-crate (same "never guess a numeric
///   constant" rule as `G2SURFACEFLAG_ISBOLT` above);
/// - `mod_a = R_GetModelByHandle(currentModel->mdxm->animIndex)` — needs the
///   `RenderModels`/`ModelData` registry (`crate::tr_model::render_models`)
///   threaded in; not part of this fn's oracle signature and not
///   reconstructible from `CConstructBoneList` alone.
///
/// `CBL.currentModel: Option<ModelHandle>` (this file's pre-existing
/// `CConstructBoneList` field) resolves to the empty `ModelAsset`
/// client-rendering placeholder (`render_state::model_asset`), not the live
/// server-side `model_t` this fn actually walks — so, matching the wave-0
/// precedent `g2_find_surface_bc`/`g2_process_surface_bolt` already set
/// (both take `&model_t` directly), `current_model` is threaded as an
/// explicit parameter instead; `CBL.current_model` is left unread by this
/// port.
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:2796-2874`
pub fn g2_construct_used_bone_list(cbl: &mut CConstructBoneList, current_model: &model_t) {
    // back track and get the surfinfo struct for this surface
    let mdxm = mdxm_view_of(current_model);
    let surface = mdxm.find_surface(cbl.surface_num, 0);
    let surf_info = mdxm.surf_hierarchy(surface.this_surface_index());

    // see if we have an override surface in the surface list
    let surf_override = g2_find_override_surface(cbl.surface_num, cbl.root_s_list);

    // really, we should use the default flags for this surface unless it's
    // been overriden
    let off_flags = match surf_override {
        Some(o) => o.offFlags,
        None => surf_info.flags(),
    };

    // if this surface is not off, add it to the shader render list
    if off_flags & G2SURFACEFLAG_OFF == 0 {
        // DEFERRED: G2_ConstructUsedBoneList bone-marking body (see doc
        // comment above).
        // Source: oracle/codemp/renderer/tr_ghoul2.cpp:2821-2860
        todo!(
            "Port G2_ConstructUsedBoneList bone-marking body — MdxmSurfaceView bone-reference count accessor / G2BONEFLAG_ALWAYSXFORM / R_GetModelByHandle threading — oracle/codemp/renderer/tr_ghoul2.cpp:2821-2860"
        );
    } else if off_flags & G2SURFACEFLAG_NODESCENDANTS != 0 {
        // if we are turning off all descendants, then stop this recursion
        // now
        return;
    }

    // now recursively call for the children
    for i in 0..surf_info.num_children() {
        cbl.surface_num = surf_info.child(i);
        g2_construct_used_bone_list(cbl, current_model);
    }
}

/// Transform one `mdxmVertex_t` into model space by its weighted bones,
/// evaluated through an already-built `CBoneCache` (`boneCache.Eval`, not
/// `EvalRender` — `G2EVALRENDER` is undefined) —
/// [`g2_process_surface_bolt2`]'s per-vertex accumulation loop, run at each
/// of its four call sites (`tr_ghoul2.cpp:3026-3052,3057-3082,3086-3112,
/// 3175-3203`). Distinct from this file's wave-0
/// `g2_process_surface_bolt_transform` (which evaluates bones off a flat
/// `bonePtr` pair list, not a `CBoneCache`) — the two callers read bones
/// from genuinely different sources, so the helpers don't unify.
fn g2_process_surface_bolt2_transform(
    cache: &mut CBoneCache,
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
        let bone_ref = surface.bone_ref(bone_index);
        let bone = cache.eval(bone_ref);
        p[0] += bone_weight * (DotProductRow(&bone.matrix[0], vert_coords) + bone.matrix[0][3]);
        p[1] += bone_weight * (DotProductRow(&bone.matrix[1], vert_coords) + bone.matrix[1][3]);
        p[2] += bone_weight * (DotProductRow(&bone.matrix[2], vert_coords) + bone.matrix[2][3]);
    }
    p
}

/// Raven `void G2_ProcessSurfaceBolt2(CBoneCache &boneCache, const
/// mdxmSurface_t *surface, int boltNum, boltInfo_v &boltList, const
/// surfaceInfo_t *surfInfo, const model_t *mod, mdxaBone_t &retMatrix)` —
/// computes a surface-attached bolt's matrix straight off an already-built
/// `CBoneCache`, unlike this file's wave-0 [`g2_process_surface_bolt`]
/// (which threads a flat `bonePtr` pair list instead — a different, earlier
/// evaluation strategy used by the generated-surface-bolt path).
/// `boltNum`/`boltList` are unread by the oracle body, so kept out of this
/// port's parameter list (the caller already holds them — same §C7 dead-param
/// drop `mp_engine_ghoul2::render::skeleton`'s already-ported, module-private
/// twin of this exact function documents). Always writes all twelve matrix
/// entries, so returned by value.
///
/// PORT-NOTE: `mp_engine_ghoul2::render::skeleton.rs` already carries this
/// exact oracle function's logic as a module-private helper
/// (`g2_process_surface_bolt2`, serving `G2_GetBoltMatrixLow`'s needs) —
/// not reusable here (private, and `crates/mp/engine/ghoul2` is out of this
/// wave's edit scope per the workflow instructions). Duplicated across the
/// wave boundary rather than silently diverging; flagged for a future dedup
/// pass (make one of the two copies `pub(crate)`/`pub` and delegate).
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:2983-3251`
pub fn g2_process_surface_bolt2(
    bone_cache: &mut CBoneCache,
    surface: Option<MdxmSurfaceView>,
    surf_info: Option<&surfaceInfo_t>,
    mod_: &model_t,
) -> mdxaBone_t {
    let mut ret_matrix = mdxaBone_t {
        matrix: [[0.0; 4]; 3],
    };

    // now there are two types of tag surface - model ones and procedural
    // generated types - lets decide which one we have here.
    if let Some(surf_info) = surf_info.filter(|s| s.offFlags == G2SURFACEFLAG_GENERATED) {
        let surf_number = surf_info.genPolySurfaceIndex & 0x0ffff;
        let poly_number = (surf_info.genPolySurfaceIndex >> 16) & 0x0ffff;

        // find original surface our original poly was in.
        let original_surf = g2_find_surface_bc(mod_, surf_number, surf_info.genLod);

        // get the original polys indexes
        let [index0, index1, index2] = original_surf.triangle(poly_number);

        // now go and transform just the points we need from the surface
        // that was hit originally
        let p_tri = [
            g2_process_surface_bolt2_transform(
                bone_cache,
                original_surf.vert(index0),
                original_surf,
            ),
            g2_process_surface_bolt2_transform(
                bone_cache,
                original_surf.vert(index1),
                original_surf,
            ),
            g2_process_surface_bolt2_transform(
                bone_cache,
                original_surf.vert(index2),
                original_surf,
            ),
        ];

        // work out baryCentricK. Raven `float baryCentricK = 1.0 - (...)`
        // — the `1.0` double literal makes this one subtraction a double
        // intermediate before the store to `float` (wave-0 ruling 12).
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

        // up will be towards point 0 of the original triangle.
        // so lets work it out. Vector is hit point - point 0
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
        // that's the up vector
        ret_matrix.matrix[0][2] = right[0];
        ret_matrix.matrix[1][2] = right[1];
        ret_matrix.matrix[2][2] = right[2];
    } else {
        // no, we are looking at a normal model tag
        //
        // Divergence (§19): oracle derefs `surface` unconditionally on this
        // arm; a null `surface` here is an unreachable oracle null-deref
        // (UB) — pick the defined identity-matrix fallback instead.
        let Some(surface) = surface else {
            return ret_matrix;
        };

        // whip through and actually transform each vertex
        let mut p_tri = [[0.0f32; 3]; 3];
        for (j, slot) in p_tri.iter_mut().enumerate() {
            *slot = g2_process_surface_bolt2_transform(bone_cache, surface.vert(j as i32), surface);
        }

        // clear out used arrays (`memset`, folded into the fresh
        // zero-initialized arrays below)
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

    ret_matrix
}

// ---------------------------------------------------------------------------
// R3 wave 2 (`tr_ghoul2.wave2.md`).
//
// RECONCILED, NOT RE-PORTED (marker law: "never re-port an already-ported
// fn" — preamble). Two of this wave's five assigned fns are already
// canonically ported in `mp_engine_ghoul2` (the same DEC-32 one-home surface
// this file's wave-1 header comment established for `CBoneCache` and
// friends) and are consumed from there rather than re-declared here:
//
// - `G2_TransformBone` ->
//   `mp_engine_ghoul2::render::bone_transform::g2_transform_bone` (`pub fn`).
// - `G2_GetBoltMatrixLow` ->
//   `mp_engine_ghoul2::render::skeleton::g2_get_bolt_matrix_low` (`pub fn`).
//
// Neither is called by this wave's other three fns (checked against the
// packet's own "in-module callees" digests), so no re-export was needed to
// keep this wave's live call graph closed.
// ---------------------------------------------------------------------------

/// Raven's file-scope `const static mdxaBone_t identityMatrix`
/// (`tr_ghoul2.cpp:128-133`) — cross-verified against the byte-identical
/// private copy `mp_engine_ghoul2::render::skeleton` already carries for this
/// exact oracle constant. A `static` (not `const`) item, matching that copy's
/// own rationale: the "yikes"/no-cache fallback paths below hand out a stable
/// `*mut mdxaBone_t` into it, mirroring Raven's own
/// `const_cast<mdxaBone_t *>(&identityMatrix)`.
static IDENTITY_MATRIX: mdxaBone_t = mdxaBone_t {
    matrix: [
        [0.0, -1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
    ],
};

/// Zero `mdxaBone_t` — the defined fallback [`g2_rag_get_anim_matrix`] below
/// returns on the (NDEBUG-stripped-assert) missing-bone-cache/mdxa paths a
/// §19 UB pick backs (oracle would null-deref there). Matches
/// `mp_engine_ghoul2::ragdoll`'s own private `ZERO_BONE` constant for the
/// same oracle function.
const ZERO_BONE: mdxaBone_t = mdxaBone_t {
    matrix: [[0.0; 4]; 3],
};

/// Raven `int G2_GetParentBoneMatrixLow(CGhoul2Info &ghoul2, int boneNum,
/// const vec3_t scale, mdxaBone_t &retMatrix, mdxaBone_t *&retBasepose,
/// mdxaBone_t *&retBaseposeInv)`. Out-params -> return value: every path but
/// the no-bone-cache one writes all three, so the write-or-not is expressed
/// as `Option` rather than inventing a value Raven never computes (its
/// caller only reads `retMatrix`/`retBasepose`/`retBaseposeInv` when the
/// returned `parent != -1` combined with a live cache — matching the
/// `(parent, Option<...>)` shape below one-for-one). `world_matrix` is not
/// in the oracle signature: it threads Raven's file-scope `worldMatrix`
/// global through to the wave-1-ported [`g2_get_bone_matrix_low`] this fn
/// calls, which already made that same global an explicit parameter
/// (`render/skeleton.rs` module-doc note) — porting-rules §B4, this
/// parameter grows the same threading choice up one call level rather than
/// inventing a new one.
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:780-803`
pub fn g2_get_parent_bone_matrix_low(
    g2: &mut Ghoul2System,
    ghoul2: &CGhoul2Info,
    bone_num: i32,
    scale: vec3_t,
    world_matrix: &mdxaBone_t,
) -> (i32, Option<(mdxaBone_t, *mut mdxaBone_t, *mut mdxaBone_t)>) {
    let mut parent = -1;
    let mut out = None;

    // Read the parent index + mdxa header off the cache first, then drop the
    // borrow before the possible `g2_get_bone_matrix_low(g2, ...)` call below
    // (which needs `g2` free for its own `&mut` bone-cache lookup).
    let cache_state = ghoul2
        .bone_cache
        .and_then(|id| g2.bone_caches.get(id))
        .map(|cache| (cache.get_parent(bone_num), cache.mdxa));

    if let Some((p, mdxa)) = cache_state {
        parent = p;
        let num_bones = mdxa
            .expect("G2_GetParentBoneMatrixLow: bone cache has no mdxa header")
            .num_bones();
        if parent < 0 || parent >= num_bones {
            parent = -1;
            // yikes
            let id_ptr = &IDENTITY_MATRIX as *const mdxaBone_t as *mut mdxaBone_t;
            out = Some((IDENTITY_MATRIX, id_ptr, id_ptr));
        } else {
            out = Some(g2_get_bone_matrix_low(
                g2,
                ghoul2,
                parent,
                scale,
                world_matrix,
            ));
        }
    }

    (parent, out)
}

/// Raven's private `G2_Find_Bone`-then-`G2_Add_Bone` idiom
/// `G2_RagGetAnimMatrix` uses twice (`tr_ghoul2.cpp:1441-1450,1481-1486`) — a
/// blank bone name never resolves (Raven: `if (!skel->name || !skel->name[0])
/// bListIndex=-1;`).
fn resolve_or_add_bone(ghoul2: &mut CGhoul2Info, name: &str) -> Option<usize> {
    if name.is_empty() {
        return None;
    }
    let mut idx = g2_find_bone(ghoul2.anim_model, &ghoul2.blist, name);
    if idx == -1 {
        idx = g2_add_bone(ghoul2.anim_model, &mut ghoul2.blist, name);
    }
    if idx == -1 {
        None
    } else {
        Some(idx as usize)
    }
}

/// Raven `void G2_RagGetAnimMatrix(CGhoul2Info &ghoul2, const int boneNum,
/// mdxaBone_t &matrix, const int frame)`. Out-param `matrix` -> return value
/// (§C7; every path writes it).
///
/// PORT-NOTE: `mp_engine_ghoul2::ragdoll::g2_rag_get_anim_matrix` already
/// carries this exact oracle function's logic as a private "stopgap" helper
/// serving the ragdoll solver — not reusable here (private, and
/// `crates/mp/engine/ghoul2` is out of this wave's edit scope per the
/// workflow instructions, the same reconciliation this file's
/// `g2_process_surface_bolt2` (wave 1) already documents). Duplicated across
/// the wave boundary rather than silently diverging; flagged for a future
/// dedup pass (make one of the two copies `pub(crate)`/`pub` and delegate).
/// The state-carrier parameter is `bone_caches: &BoneCacheArena` rather than
/// the whole `Ghoul2System` (unlike the `ragdoll.rs` twin), matching this
/// file's own established convention ([`g2_get_bone_name_from_skel`],
/// [`g2_needs_recalc`], wave 0/1).
///
/// `assert(ghoul2.mBoneCache)`/`assert(ghoul2.animModel)`/
/// `assert(bListIndex != -1)`/`assert(parentBlistIndex != -1)`/
/// `assert(pbone.hasAnimFrameMatrix == frame)` are compiled out under
/// `-DNDEBUG` (house convention); the no-cache/no-mdxa/unresolved-bone paths
/// that assert would have guarded instead return [`ZERO_BONE`] — a §19 UB
/// pick (the oracle would null-deref/read-uninit there), not invented. That
/// covers the empty-skeleton-name path too: `tr_ghoul2.cpp:1437-1452` sets
/// `bListIndex = -1` for a blank `skel->name`, then in a release build indexes
/// `ghoul2.mBlist[-1]` past the failed assert; [`resolve_or_add_bone`] returns
/// `None` there and this fn returns [`ZERO_BONE`] instead.
///
/// Oracle UB, second site: the oracle binds `boneInfo_t &bone =
/// ghoul2.mBlist[bListIndex]` at `:1455`, *before* the recursive call at
/// `:1470` whose `G2_Add_Bone` can reallocate `mBlist` and dangle that
/// reference; the port re-indexes `ghoul2.blist[bli]` after the recursion
/// returns, which is the same result whenever no reallocation happened.
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:1417-1539`
pub fn g2_rag_get_anim_matrix(
    bone_caches: &BoneCacheArena,
    ghoul2: &mut CGhoul2Info,
    bone_num: i32,
    frame: i32,
) -> mdxaBone_t {
    let Some(cache) = ghoul2.bone_cache.and_then(|id| bone_caches.get(id)) else {
        return ZERO_BONE;
    };
    let root_matrix = cache.root_matrix;
    let Some(mdxa) = cache.mdxa else {
        return ZERO_BONE;
    };

    // find/add the bone in the list
    let skel = mdxa.skel(bone_num);
    let name = skel.name.clone();
    let Some(bli) = resolve_or_add_bone(ghoul2, &name) else {
        return ZERO_BONE;
    };

    if ghoul2.blist[bli].hasAnimFrameMatrix == frame {
        // already calculated so just grab it
        return ghoul2.blist[bli].animFrameMatrix;
    }

    // get the base matrix for the specified frame
    let mut anim_matrix = ZERO_BONE;
    uncompress_bone(&mut anim_matrix.matrix, bone_num, mdxa, frame);

    let parent = skel.parent;
    let mut result = ZERO_BONE;
    if bone_num > 0 && parent > -1 {
        // recursively call to assure all parent matrices are set up
        let _ = g2_rag_get_anim_matrix(bone_caches, ghoul2, parent, frame);

        // assign the new skel ptr for our parent
        let pname = mdxa.skel(parent).name.clone();

        // taking bone matrix for the skeleton frame and parent's
        // animFrameMatrix into account, determine our final animFrameMatrix
        let Some(pbli) = resolve_or_add_bone(ghoul2, &pname) else {
            return ZERO_BONE;
        };
        let parent_anim_matrix = ghoul2.blist[pbli].animFrameMatrix;
        multiply_3x4_matrix(&mut result, &parent_anim_matrix, &anim_matrix);
    } else {
        // root
        multiply_3x4_matrix(&mut result, &root_matrix, &anim_matrix);
    }

    // never need to figure it out again
    let bone = &mut ghoul2.blist[bli];
    bone.animFrameMatrix = result;
    bone.hasAnimFrameMatrix = frame;
    result
}

/// Raven `static int R_GCullModel(trRefEntity_t *ent)` — culls `ent`'s
/// bounding sphere against the view frustum, bumping the matching
/// `tr.pc.c_sphere_cull_md3_{out,in,clip}` perf counter.
///
/// `ent->e.modelScale`/`ent->e.radius` are threaded as explicit
/// `model_scale`/`radius` parameters rather than read off `RefEntity` — same
/// rationale [`g2_compute_lod`] (this file, wave 1) already documents for the
/// same two fields, and `ent->e.origin` is unread here (oracle culls around
/// `vec3_origin`, not the entity's world position). The three `tr.pc.*`
/// counters have no state carrier yet (`FrameState::counters: BackEndCounters`
/// is still the R4-backend-wave empty placeholder, `render_state::
/// placeholders`, out of this file's edit scope) — threaded as explicit
/// `&mut i32` outs instead of reaching into that empty struct, matching the
/// established precedent `tr_world.rs`'s `R_DlightFace`/`R_DlightGrid` set
/// for the identical `BackEndCounters`-is-empty situation (PORT-NOTE there:
/// "state is threaded, not reached", porting-rules §4). `ori`/
/// `r_nocull_integer`/`frustum` are [`R_CullLocalPointAndRadius`]'s own
/// (wave 1) already-threaded parameters, forwarded through unchanged.
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:896-930`
#[allow(clippy::too_many_arguments)]
pub fn r_g_cull_model(
    model_scale: vec3_t,
    radius: f32,
    ori: &orientationr_t,
    r_nocull_integer: i32,
    frustum: &[cplane_t; 4],
    c_sphere_cull_md3_out: &mut i32,
    c_sphere_cull_md3_in: &mut i32,
    c_sphere_cull_md3_clip: &mut i32,
) -> i32 {
    // scale the radius if need be
    let mut largest_scale = model_scale[0];
    if model_scale[1] > largest_scale {
        largest_scale = model_scale[1];
    }
    if model_scale[2] > largest_scale {
        largest_scale = model_scale[2];
    }
    if largest_scale == 0.0 {
        largest_scale = 1.0;
    }

    // cull bounding sphere
    match R_CullLocalPointAndRadius(
        vec3_origin,
        radius * largest_scale,
        ori,
        r_nocull_integer,
        frustum,
    ) {
        CULL_OUT => {
            *c_sphere_cull_md3_out += 1;
            CULL_OUT
        }
        CULL_IN => {
            *c_sphere_cull_md3_in += 1;
            CULL_IN
        }
        CULL_CLIP => {
            *c_sphere_cull_md3_clip += 1;
            CULL_IN
        }
        _ => CULL_IN,
    }
}
