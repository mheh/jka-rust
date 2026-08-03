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

use mp_qshared::common::mp::qcommon::tags::memtag_t;
use mp_qshared::shared::com_parse::QSharedScratch;
use mp_qshared::shared::q_color::S_COLOR_YELLOW;
use mp_qshared::shared::q_math::{
    _DotProduct, _VectorMA, _VectorSubtract, vec3_origin, CrossProduct, DotProductRow,
    VectorNormalize2,
};
use mp_qshared::shared::swap::LittleLong;
use mp_qshared::shared::{cplane_t, errorParm_t, mdxaBone_t, qhandle_t, vec3_t, VectorNormalize};

use mp_host_interface::mdx::mdxa::{MdxaRef, MdxaView};
use mp_host_interface::mdx::mdxm::{MdxmSurfaceView, MdxmVertView, MdxmView};
use mp_host_interface::EngineHost;

// USER RULING (DEC-32 one-home): every bone-evaluation type/function below is
// consumed from `mp_engine_ghoul2` (the DEC-35 canonical port of the very same
// `tr_ghoul2.cpp` definitions), never re-declared in this crate.
use mp_engine_ghoul2::bolts::g2_find_bolt_surface_num;
use mp_engine_ghoul2::bones::{g2_add_bone, g2_find_bone};
use mp_engine_ghoul2::api_collision::g2api_get_time;
use mp_engine_ghoul2::ghoul2_system::{BoneCacheArena, BoneCacheId, Ghoul2System};
use mp_engine_ghoul2::info_array::Ghoul2Handle;
use mp_engine_ghoul2::misc::{g2_setup_model_pointers, g2_setup_model_pointers_v};
use mp_engine_ghoul2::shared::cghoul2_info_v::CGhoul2Info_v;
use mp_engine_ghoul2::render::bone_cache::CBoneCache;
use mp_engine_ghoul2::render::bone_transform::{multiply_3x4_matrix, uncompress_bone};
use mp_engine_ghoul2::render::skeleton::{
    g2_construct_render_skeleton, g2_get_bone_matrix_low,
};
use mp_engine_ghoul2::shared::bolt_info_t::boltInfo_t;
use mp_engine_ghoul2::shared::bone_info_t::boneInfo_t;
use mp_engine_ghoul2::shared::cghoul2_info::CGhoul2Info;
use mp_engine_ghoul2::shared::surface_info_t::surfaceInfo_t;
use mp_engine_ghoul2::surfaces::g2_find_override_surface;

use mp_engine_qcommon::common::{com_error, com_printf, EngineHostView};
use mp_engine_qcommon::qfiles::shader_limits::{SHADER_MAX_INDEXES, SHADER_MAX_VERTEXES};
use native_string::q_string::Q_strlwr;

use crate::mdx_format::mdxa_header_t::mdxaHeader_t;
use crate::mdx_format::mdxm_header_t::mdxmHeader_t;
use crate::mdx_format::mdxm_lod_t::mdxmLOD_t;
use crate::mdx_format::mdxm_lodsurf_offset_t::mdxmLODSurfOffset_t;
use crate::mdx_format::mdxm_surf_hierarchy_t::mdxmSurfHierarchy_t;
use crate::mdx_format::mdxm_surface_t::mdxmSurface_t;
use crate::mdx_format::mdxm_vertex_t::mdxmVertex_t;
use crate::render_state::frame_state::FrameState;
use crate::render_state::placeholders::RefEntity;
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::render_cvar_snapshot::RenderCvarSnapshot;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::render_state::walk_warnings::WalkWarnings;
use crate::render_state::world_load_state::WorldLoadState;
use crate::render_state::shader_asset::ShaderHandle;
use crate::render_state::skin_asset::SkinHandle;
use crate::tr_image::{TrImageState, R_GetSkinByHandle};
use crate::tr_light::R_SetupEntityLighting;
use crate::tr_local::dlight_s::dlight_t;
use crate::tr_local::fog_t::fog_t;
use crate::tr_local::model_s::model_t;
use crate::tr_local::modtype_t::modtype_t;
use crate::tr_local::orientationr_t::orientationr_t;
use crate::tr_local::surface_type_t::surfaceType_t;
use crate::tr_local::view_parms_t::viewParms_t;
use crate::tr_main::{
    DrawSurf, G2SurfaceRef, R_AddDrawSurf, R_CullLocalPointAndRadius, SurfaceGeometry, CULL_CLIP,
    CULL_IN, CULL_OUT,
};
use crate::tr_mesh::project_radius;
use crate::tr_public::ref_flags::{RDF_NOFOG, RDF_NOWORLDMODEL};
use crate::tr_model::frontend::{mdxm_view_of, re_register_models_malloc, RE_RegisterModel};
use crate::tr_model::model_pool::ModelHandle;
use crate::tr_model::render_models::RenderModels;
use crate::tr_model::server_load::read_qpath;
use crate::tr_shade_calc::myftol;
use crate::tr_shader::{lightmapsNone, stylesDefault, R_FindShader, R_GetShaderByHandleQuiet};
use mp_qshared::common::mp::cgame::tr_types::RF_THIRD_PERSON;
use crate::tr_worldeffects::world_effects::WorldEffectsState;

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

/// Raven `int OldToNewRemapTable[72]` — the JK2->JKA bone remap table
/// [`RenderModels::r_load_mdxm`] runs over an old (`numBones == 72`,
/// `_humanoid`) `.glm`'s bone references. File-scope in this very TU, right
/// above the `_humanoid` skeleton commentary that documents each entry's
/// source bone; Raven's mixed-case name is preserved (same
/// `non_upper_case_globals` allowance `tr_shader.rs`'s `lightmapsNone` uses).
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:4469-4542`
#[allow(non_upper_case_globals)]
const OldToNewRemapTable: [i32; 72] = [
    0, 1, 2, 3, 4, 5, 6, 6, 7, 8, 9, 10, //
    10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, //
    22, 23, 24, 25, 26, 27, 28, 29, 29, 34, 35, 35, //
    30, 31, 31, 32, 33, 33, 32, 33, 33, 34, 35, 35, //
    36, 37, 38, 39, 40, 41, 42, 42, 43, 44, 44, 43, //
    44, 44, 45, 46, 46, 45, 46, 46, 47, 48, 48, 52,
];

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

/// Raven `static int R_GComputeFogNum(trRefEntity_t *ent)` — the fog volume
/// `ent`'s bounding sphere falls inside, if any.
///
/// `fogs` is `tr.world->fogs` (index 0 is the reserved "no fog" slot, matching
/// the oracle's `for (i=1; i<numfogs; i++)`); `refdef_rdflags` is
/// `tr.refdef.rdflags`. The entity's own `origin`/`radius` bound the test, not
/// a frame-array read as the MD3 twin ([`r_compute_fog_num`], `tr_mesh.rs`)
/// does.
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:939-964`
pub fn r_g_compute_fog_num(ent: &RefEntity, fogs: &[fog_t], refdef_rdflags: i32) -> i32 {
    if refdef_rdflags & RDF_NOWORLDMODEL != 0 {
        return 0;
    }

    for i in 1..fogs.len() {
        let fog = &fogs[i];
        let mut j = 0usize;
        while j < 3 {
            if ent.origin[j] - ent.radius >= fog.bounds[1][j] {
                break;
            }
            if ent.origin[j] + ent.radius <= fog.bounds[0][j] {
                break;
            }
            j += 1;
        }
        if j == 3 {
            return i as i32;
        }
    }

    0
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
/// `r_lodbias`/`r_lodscale`/`r_autolodscalevalue` read through
/// `Common::cvar` (the `RendererCvars`-handle + live-engine-table pattern
/// `tr_light.rs`'s `R_SetupEntityLightingGrid` already established).
/// `ProjectRadius`/`myftol` are the cross-file in-module callees.
/// `project_radius` takes the live `viewParms_t` (E2), so `view` threads
/// straight through to it.
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:967-1041`
pub fn g2_compute_lod(
    ent: &RefEntity,
    current_model: &model_t,
    lod_bias: i32,
    view: &viewParms_t,
    cvars: RenderCvarSnapshot,
) -> i32 {
    // model has only 1 LOD level, skip computations and bias
    if current_model.numLods < 2 {
        return 0;
    }

    let model_scale = ent.model_scale;

    let mut lod_bias = lod_bias;
    if cvars.lodbias > lod_bias {
        lod_bias = cvars.lodbias;
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
    let scaled_radius = (0.75_f64 * largest_scale as f64 * ent.radius as f64) as f32;
    // we reduce the radius to make the LOD match other model types which use
    // the actual bound box size
    let projected_radius = project_radius(scaled_radius, ent.origin, view);
    let mut flod;
    if projected_radius != 0.0 {
        let mut lodscale = cvars.lodscale + cvars.autolodscalevalue;
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
/// ripping right from SP." Walks the surface tree, resolves each visible
/// surface's shader, and pushes a Ghoul2 draw surf.
///
/// The shader resolve follows the oracle priority: the custom shader first,
/// else a skin-name match against [`SkinAsset::surfaces`], else the surface's
/// own default shader. DEC-42.2 stores that default in `surfInfo->shaderIndex`
/// as the shader arena slot number, so the handle reads back through
/// `Arena::handle_at_slot`.
///
/// The draw surf is a `Copy` [`G2SurfaceRef`] rather than Raven's raw-pointer
/// `CRenderableSurface` (R2 Group-4 table): it carries the model handle,
/// the LOD, the surface index, and the bone-cache id, so the backend re-locates
/// the surface and reads the cache from the arena.
///
/// DEFERRED in this arm: the stencil- and projection-shadow pushes
/// (`r_shadows == 2`/`3`) and the `_G2_GORE` overlay chain. Both build extra
/// draw surfs from tier-2 fields and land with the shadow/gore backend waves.
/// The gore fields (`scale`/`fade`/`impactTime`) stay off [`G2SurfaceRef`].
///
/// `RS.currentModel`/`RS.currentModel->mdxm` non-null asserts are dropped —
/// compiled-out under this build's `-DNDEBUG` (house convention, e.g.
/// `mp_engine_ghoul2::bolts`'s module doc comment).
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:2521-2735`
pub fn render_surfaces<'a>(
    rs: &mut CRenderSurface,
    current_model: &model_t,
    assets: &RenderAssets,
    shifted_entity_num: i32,
    rdf_nofog: bool,
    draw_surfs: &mut Vec<DrawSurf<SurfaceGeometry<'a>>>,
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
        // figure out whether we should be using a custom shader for this
        // surface, else the skin match, else the surface's default shader.
        let shader = if let Some(cust) = rs.cust_shader {
            cust
        } else if let Some(skin_handle) = rs.skin {
            // match the surface name to something in the skin file
            let surf_name = surf_info.name_lossy();
            match assets.skins.get(skin_handle) {
                Some(skin) => {
                    let mut resolved = ShaderHandle::slot_zero();
                    for skin_surface in &skin.surfaces {
                        // the names have both been lowercased
                        if skin_surface.name == surf_name {
                            resolved = skin_surface.shader;
                            break;
                        }
                    }
                    resolved
                }
                None => ShaderHandle::slot_zero(),
            }
        } else {
            // Raven: `R_GetShaderByHandle(surfInfo->shaderIndex)`. DEC-42.2
            // stores the shader arena slot in `shaderIndex`.
            assets
                .shaders
                .handle_at_slot(surf_info.shader_index() as u32)
                .unwrap_or_else(ShaderHandle::slot_zero)
        };

        // DEFERRED: the stencil-shadow (`r_shadows == 2`), projection-shadow
        // (`r_shadows == 3`), and `_G2_GORE` overlay pushes — see the doc
        // comment above.
        // Source: oracle/codemp/renderer/tr_ghoul2.cpp:2586-2715

        // don't add third_person objects if not viewing through a portal
        if !rs.personal_model {
            // A live render surface always has a built bone cache
            // (`G2_TransformGhoulBones` ran first). A missing cache means the
            // surface is not renderable, so it is dropped.
            if let Some(bone_cache) = rs.bone_cache {
                let sorted_index = assets
                    .shaders
                    .get(shader)
                    .map(|s| s.sorted_index)
                    .unwrap_or(0);

                // Raven's `RB_SurfaceGhoul` tess body
                // (`oracle/codemp/renderer/tr_ghoul2.cpp:4060-4451`) has no
                // Rust twin. Its bone deform dissolves into the R4 vertex
                // pipeline, which reads this `G2SurfaceRef`. The gore chain,
                // the `alternateTex` overlay, and the dynamic-glow arms of
                // that body stay unported.
                R_AddDrawSurf(
                    SurfaceGeometry::Ghoul2(G2SurfaceRef {
                        model: current_model.index,
                        lod: rs.lod,
                        surface_index: rs.surface_num,
                        bone_cache,
                    }),
                    sorted_index,
                    shifted_entity_num,
                    rdf_nofog,
                    rs.fog_num,
                    0,
                    draw_surfs,
                );
            }
        }
    }

    // if we are turning off all descendants, then stop this recursion now
    if off_flags & G2SURFACEFLAG_NODESCENDANTS != 0 {
        return;
    }

    // now recursively call for the children
    for i in 0..surf_info.num_children() {
        rs.surface_num = surf_info.child(i);
        render_surfaces(
            rs,
            current_model,
            assets,
            shifted_entity_num,
            rdf_nofog,
            draw_surfs,
        );
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
/// `CConstructBoneList` field) is a `ModelPool` handle
/// (`crate::tr_model::model_pool`), which this fn has no pool receiver to
/// resolve against — so, matching the wave-0
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
/// `ent->e.origin` is unread here, because the oracle culls around
/// `vec3_origin`, not the entity's world position. The three `tr.pc.*`
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
    ent: &RefEntity,
    ori: &orientationr_t,
    r_nocull_integer: i32,
    frustum: &[cplane_t; 4],
    c_sphere_cull_md3_out: &mut i32,
    c_sphere_cull_md3_in: &mut i32,
    c_sphere_cull_md3_clip: &mut i32,
) -> i32 {
    let model_scale = ent.model_scale;

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
        ent.radius * largest_scale,
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

// ---------------------------------------------------------------------------
// R3 wave 3 (`tr_ghoul2.wave3.md`).
//
// RECONCILED, NOT RE-PORTED (marker law: "never re-port an already-ported
// fn" — preamble). All three of this wave's assigned fns are already
// canonically ported in `mp_engine_ghoul2` (the same DEC-32 one-home surface
// this file's wave-1/wave-2 header comments already established for
// `CBoneCache` and friends) and are consumed from there rather than
// re-declared here:
//
// - `EvalLow` -> `mp_engine_ghoul2::render::bone_cache::CBoneCache::
//   eval_low` — a *private* method (the memoized-recursion core
//   `CBoneCache::Eval`/`EvalUnsmooth`/`EvalRender` call internally, same
//   already-ported-elsewhere status this file's wave-1 header documents for
//   `SmoothLow`); nothing in this file needs to call it directly.
// - `RootMatrix` -> `mp_engine_ghoul2::render::skeleton::root_matrix` — also
//   private there (module-doc gap note: not named in the doc's own
//   method-transcription roster, ported anyway), mutually recursive with
//   `g2_construct_ghoul_skeleton` matching this wave's own SCC 330 grouping.
// - `G2_ConstructGhoulSkeleton` ->
//   `mp_engine_ghoul2::render::skeleton::g2_construct_ghoul_skeleton`
//   (`pub fn`).
//
// `identityMatrix` — this wave's STATE HOMES row for both `RootMatrix` and
// `G2_ConstructGhoulSkeleton` (DEC-37 A13.3: "genuinely-const tables ->
// const") — is already named in this very file as [`IDENTITY_MATRIX`] (wave
// 2, above); no new state carrier is needed even had these bodies been
// re-transcribed here. The `G2_PERFORMANCE_ANALYSIS`-only touches this
// wave's STATE HOMES table flags (`G2PerformanceTimer_
// G2_ConstructGhoulSkeleton` read, `G2Time_G2_ConstructGhoulSkeleton` write)
// are dead surface under this build (DEC-37 A13.5, matching this file's own
// module doc comment, which already states the same drop applies to every
// ported function in this file).
//
// None of this wave's three fns are called by anything else already ported
// in this file (checked against the packet's own SCC 330/in-module-callee
// digests and a grep of this file for `RootMatrix`/`root_matrix`/
// `ConstructGhoulSkeleton`/`EvalLow`), so no re-export was needed to keep
// this wave's live call graph closed.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// R3 wave 9 (`tr_ghoul2.wave9.md`) — final tail wave.
// ---------------------------------------------------------------------------

/// Raven `MDXA_VERSION` — cross-verified (never guessed, porting-rules §A2)
/// against the already-ported copy `tr_model::server_load` carries for the
/// exact same oracle `#define` (this wave's packet does not list it in its
/// own `## FILE-SCOPE CONSTANTS` section — it lives in `mdx_format.h`, not
/// `tr_ghoul2.cpp` — but the never-guess rule permits reuse of an
/// already-verified copy).
///
/// Source: `oracle/codemp/renderer/mdx_format.h:29`
const MDXA_VERSION: i32 = 6;

impl RenderModels {
    /// Raven `qboolean R_LoadMDXA(model_t *mod, void *buffer, const char
    /// *mod_name, qboolean &bAlreadyCached)` — the CLIENT model-registration
    /// path's Ghoul 2 animation-file (`.gla`) loader. Distinct oracle
    /// function from the already-ported dedicated-server twin
    /// `ServerLoadMDXA` (`tr_model.cpp:683`, ported as
    /// `RenderModels::server_load_mdxa` in `tr_model/server_load.rs`) — this
    /// one is `tr_ghoul2.cpp:5256`, reached from `R_RegisterModel`'s client
    /// path, not `RE_RegisterServerModel`'s. Not a re-port (marker law
    /// "never re-port an already-ported fn" — the two are genuinely separate
    /// Raven functions, porting-rules §20 "duplicate, don't unify"), but its
    /// body mirrors `server_load_mdxa`'s already-established idiom one for
    /// one wherever the two functions' oracle bodies agree.
    ///
    /// `mod` -> `model: qhandle_t` re-resolved into `self.models` inside the
    /// method body (not a `&mut model_t` sibling parameter), matching
    /// `server_load_mdxa`'s own established split-borrow rationale (this
    /// file's `RenderModels` methods borrow `&mut self`, so a second live
    /// `&mut ModelData` parameter would alias the receiver — `server_load.rs`
    /// module doc). Out-param `bAlreadyCached` -> `&mut bool` (kept by-ref:
    /// the caller reads both the load result and the already-cached flag
    /// independently, so folding it into the return would lose information a
    /// plain out-param-to-return translation keeps).
    ///
    /// `CREATE_LIMB_HIERARCHY` (`tr_ghoul2.cpp:5051`, `//#define
    /// CREATE_LIMB_HIERARCHY`) is commented out in the oracle source itself
    /// — dead surface at the source level (porting-rules §20), dropped
    /// without a per-site note (its four gated blocks, `:5261-5449`, never
    /// compile in retail). The `#ifndef _M_IX86` skeletal/frame byte-swap
    /// loop (`:5461-5500`) is likewise dropped — dead on this port's LE
    /// x86-64 target, matching `server_load_mdxa`'s own `TRM-D3`/ruling 54
    /// disposition for the identical arm in its own oracle twin.
    ///
    /// Carrier shape: this file's client model loaders take the same bundle
    /// as their `tr_model/frontend.rs` family (that file's top-of-module
    /// DEC-42.3 note) — `view: &mut EngineHostView` for engine services,
    /// `common` reached as `view.common`, and the renderer state
    /// `re_register_models_malloc`/`R_FindShader` need threaded beside it.
    ///
    /// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:5256-5502`
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn r_load_mdxa(
        &mut self,
        qs: &mut QSharedScratch,
        world_load: &mut WorldLoadState,
        assets: &mut RenderAssets,
        view: &mut EngineHostView,
        cvars: &RendererCvars,
        img_state: &mut TrImageState,
        sky_view: &mut viewParms_t,
        model: qhandle_t,
        buffer: &[u8],
        mod_name: &str,
        already_cached: &mut bool,
    ) -> bool {
        // read some fields from the binary, but only LittleLong() them when
        // we know this wasn't an already-cached model...
        let mut version = i32::from_le_bytes(buffer[4..8].try_into().unwrap()); // mdxaHeader_t::version
        let mut size = i32::from_le_bytes(buffer[96..100].try_into().unwrap()); // mdxaHeader_t::ofsEnd

        if !*already_cached {
            version = LittleLong(version);
            size = LittleLong(size);
        }

        let warn = S_COLOR_YELLOW.to_str().expect("S_COLOR_YELLOW is ASCII");

        if version != MDXA_VERSION {
            com_printf(
                view.common,
                &format!(
                    "{}R_LoadMDXA: {} has wrong version ({} should be {})\n",
                    warn, mod_name, version, MDXA_VERSION
                ),
            );
            return false;
        }

        let idx = model as usize;
        self.models.slot_mut(idx).r#type = modtype_t::MOD_MDXA;
        self.models.slot_mut(idx).dataSize += size;

        let (ptr, already_found) = re_register_models_malloc(
            qs,
            world_load,
            assets,
            view,
            cvars,
            self,
            img_state,
            sky_view,
            size,
            Some(buffer),
            mod_name,
            memtag_t::TAG_MODEL_GLA,
        );
        // Raven: `assert(bAlreadyCached == bAlreadyFound); // I should
        // probably eliminate 'bAlreadyFound', but wtf?` — compiled out under
        // `-DNDEBUG` (house convention), kept as a debug assert.
        debug_assert_eq!(
            *already_cached, already_found,
            "bAlreadyCached == bAlreadyFound"
        );
        // `server_load_mdxa`'s own precedent assert for the identical
        // `AlignedBytes`-backed cast below.
        debug_assert_eq!(
            ptr as usize % 16,
            0,
            "AlignedBytes base must be 16-byte aligned"
        );

        let mdxa = ptr as *mut mdxaHeader_t;
        self.models.slot_mut(idx).mdxa = mdxa;

        if !already_found {
            // horrible new hackery, if !bAlreadyFound then we've just done a
            // tag-morph, so we need to set the bool reference passed into
            // this function to true, to tell the caller NOT to do an
            // FS_Freefile since we've hijacked that memory block...
            // Aaaargh. Kill me now...
            //
            // `assert( mdxa == buffer )` doesn't hold under the
            // `re_register_models_malloc` ingest-copy divergence
            // `server_load_mdxa` already documents (`TRM-D4`(a)/ruling 58)
            // and is dropped, not ported (§19).
            *already_cached = true;

            // SAFETY: `mdxa` is the just-copy-constructed 16-byte-aligned
            // `AlignedBytes` base (`TRM-D4`/ruling 58); the debug alignment
            // assert above covers the cast, per §D11 — matching
            // `server_load_mdxa`'s identical quarantine.
            unsafe {
                (*mdxa).ident = LittleLong((*mdxa).ident);
                (*mdxa).version = LittleLong((*mdxa).version);
                (*mdxa).numFrames = LittleLong((*mdxa).numFrames);
                (*mdxa).numBones = LittleLong((*mdxa).numBones);
                (*mdxa).ofsFrames = LittleLong((*mdxa).ofsFrames);
                (*mdxa).ofsEnd = LittleLong((*mdxa).ofsEnd);
            }
        }

        // SAFETY: see above — `mdxa` is the aligned, live block either way.
        let num_frames = unsafe { (*mdxa).numFrames };
        if num_frames < 1 {
            com_printf(
                view.common,
                &format!("{}R_LoadMDXA: {} has no frames\n", warn, mod_name),
            );
            return false;
        }

        if already_found {
            // All done, stop here, do not LittleLong() etc. Do not pass
            // go...
            return true;
        }

        // `#ifndef _M_IX86` skeletal/frame swaps (`:5461-5500`) — dropped,
        // see doc comment above.

        // DEC-35: build the parse-once `MdxaParsed` sidecar over the now
        // swap-completed block (fresh-load path only; a cache hit returned
        // above and keeps its already-built sidecar) — matching
        // `server_load_mdxa`'s identical final step.
        self.store_parsed_mdxa(mod_name);
        true
    }
}

// ---------------------------------------------------------------------------
// R3 wave 12 (`tr_ghoul2.wave12.md`) — final tail wave of the R3 renderer
// port.
// ---------------------------------------------------------------------------

/// Raven `MDXM_VERSION` — cross-verified (never guessed, porting-rules §A2)
/// against the already-ported copy `tr_model::server_load` carries for the
/// exact same oracle `#define` (this wave's packet does not list it in its
/// own `## FILE-SCOPE CONSTANTS` section — it lives in `mdx_format.h`, not
/// `tr_ghoul2.cpp` — but the never-guess rule permits reuse of an
/// already-verified copy, matching [`MDXA_VERSION`]'s (wave 9, above)
/// identical precedent).
///
/// Source: `oracle/codemp/renderer/mdx_format.h:28`
const MDXM_VERSION: i32 = 6;

impl RenderModels {
    /// Raven `qboolean R_LoadMDXM(model_t *mod, void *buffer, const char
    /// *mod_name, qboolean &bAlreadyCached)` — the CLIENT model-registration
    /// path's Ghoul 2 mesh-file (`.glm`) loader. Distinct oracle function
    /// from the already-ported dedicated-server twin `ServerLoadMDXM`
    /// (`tr_model.cpp:799`, ported as `RenderModels::server_load_mdxm` in
    /// `tr_model/server_load.rs`) — this one is `tr_ghoul2.cpp:4816`, reached
    /// from `RE_RegisterModel_Actual`'s client path, not
    /// `RE_RegisterServerModel`'s (matching [`Self::r_load_mdxa`]'s (wave 9,
    /// above) identical "not a re-port" rationale for its own server twin).
    /// Its body mirrors `server_load_mdxm`'s already-established
    /// version-peek/malloc/`LL()`-swap/anim-registration/surface-hierarchy/
    /// LOD-swap idiom one for one wherever the two oracle bodies agree;
    /// differences are called out per divergent site below.
    ///
    /// `mod` -> `model: qhandle_t` re-resolved into `self.models` inside the
    /// method body (not a `&mut model_t` sibling parameter) — same
    /// split-borrow rationale `server_load_mdxm`/`r_load_mdxa` already
    /// establish (`RenderModels` methods borrow `&mut self`, so a second live
    /// `&mut ModelData` parameter would alias the receiver). Out-param
    /// `bAlreadyCached` -> `&mut bool`, kept by-ref for the same reason
    /// `r_load_mdxa` already gives.
    ///
    /// `mdxm->animIndex = RE_RegisterModel(va("%s.gla", mdxm->animName))`
    /// (`:4884-4891`) is a genuine SCC-345 mutual-recursion edge back into
    /// `tr_model/frontend.rs`'s [`RE_RegisterModel`], called here with this
    /// family's shared carrier bundle (that file's top-of-module DEC-42.3
    /// note): `view: &mut EngineHostView` for engine services, `common`
    /// reached as `view.common`, and the renderer state `R_FindShader`/
    /// `RE_LoadWorldMap_Actual` need threaded beside it.
    ///
    /// The surface-hierarchy walk's `Q_strlwr`/trailing-`"_off"`-strip
    /// (`:4912-4916`) has no twin in `server_load_mdxm` (the dedicated server
    /// never touches surface names) — transcribed here as new client-path
    /// behavior, guarding the 4-byte tail read against a short name (§19: the
    /// oracle's unguarded `&surfInfo->name[strlen(name)-4]` underflows on a
    /// name shorter than 4 bytes; the defined choice is "too short to end in
    /// `_off`, don't strip").
    ///
    /// The `#ifndef DEDICATED` shader lookup (`:4926-4938`, `R_FindShader(
    /// surfInfo->shader, lightmapsNone, stylesDefault, qtrue)` ->
    /// `surfInfo->shaderIndex`) is the client leg (DEC-40) and is transcribed
    /// live: a default shader pokes `0`, else the resolved handle's arena
    /// slot number, which IS Raven's `shader_t::index` (DEC-42.2).
    /// `RE_RegisterModels_StoreShaderRequest` (`:4939`) still runs
    /// unconditionally, outside both `#ifdef` arms, exactly as Raven has it.
    ///
    /// `SHADER_MAX_VERTEXES`/`SHADER_MAX_INDEXES` bound overflows raise
    /// `Com_Error(ERR_DROP, ...)` here (`:4968-4975`) — unlike
    /// `server_load_mdxm`'s own plain `return qfalse` for the identical
    /// check, a genuine divergence between the two sibling oracle functions
    /// (faithfully kept, not normalized, §A2), matching `r_load_md3`'s own
    /// already-ported `Com_Error` transcription of the same pattern.
    ///
    /// `if (isAnOldModelFile) { ... OldToNewRemapTable[boneRef[j]] ... }`
    /// (`:5026-5041`) is transcribed live, against the
    /// [`OldToNewRemapTable`] this file now carries — the table is file-scope
    /// in this same oracle TU (`:4469-4542`), not in `G2_bones.cpp`. Raven's
    /// `assert(boneRef[j] >= 0 && boneRef[j] < 72)` is a debug assert (house
    /// convention, compiled out under `-DNDEBUG`); the guarded `else
    /// boneRef[j]=0` arm is kept as Raven has it. Its bracketing
    /// `isAnOldModelFile` detection is `:4900-4904` (`numBones == 72 &&
    /// strstr(animName, "_humanoid")`).
    ///
    /// Every remaining `LL()` swap, the `SF_MDX` ident stamp, and the
    /// `#ifndef _M_IX86` bone-ref/triangle/vertex byte-swap block
    /// (`:4980-5024`, dead on this port's LE x86-64 target, `TRM-D3`/ruling
    /// 54, matching `server_load_mdxm`'s identical disposition for the exact
    /// same nested block) transcribe one for one with `server_load_mdxm`.
    ///
    /// The `*mut mdxmHeader_t` cast and every in-place surface/LOD field
    /// read+swap operate on the 16-byte-aligned `AlignedBytes` base
    /// (`TRM-D4`/ruling 58); `unsafe`-confined at this seam (§D11) with a
    /// debug alignment assert at each cast site, matching
    /// `server_load_mdxm`/`r_load_mdxa`.
    ///
    /// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:4816-5049`
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn r_load_mdxm(
        &mut self,
        qs: &mut QSharedScratch,
        world_load: &mut WorldLoadState,
        assets: &mut RenderAssets,
        view: &mut EngineHostView,
        cvars: &RendererCvars,
        img_state: &mut TrImageState,
        sky_view: &mut viewParms_t,
        world_effects: &mut WorldEffectsState,
        model: qhandle_t,
        buffer: &[u8],
        mod_name: &str,
        already_cached: &mut bool,
    ) -> bool {
        // read some fields from the binary, but only LittleLong() them when
        // we know this wasn't an already-cached model...
        let mut version = i32::from_le_bytes(buffer[4..8].try_into().unwrap()); // mdxmHeader_t::version
        let mut size = i32::from_le_bytes(buffer[160..164].try_into().unwrap()); // mdxmHeader_t::ofsEnd

        if !*already_cached {
            version = LittleLong(version);
            size = LittleLong(size);
        }

        let warn = S_COLOR_YELLOW.to_str().expect("S_COLOR_YELLOW is ASCII");

        if version != MDXM_VERSION {
            com_printf(
                view.common,
                &format!(
                    "{}R_LoadMDXM: {} has wrong version ({} should be {})\n",
                    warn, mod_name, version, MDXM_VERSION
                ),
            );
            return false;
        }

        let idx = model as usize;
        self.models.slot_mut(idx).r#type = modtype_t::MOD_MDXM;
        self.models.slot_mut(idx).dataSize += size;

        let (ptr, already_found) = re_register_models_malloc(
            qs,
            world_load,
            assets,
            view,
            cvars,
            self,
            img_state,
            sky_view,
            size,
            Some(buffer),
            mod_name,
            memtag_t::TAG_MODEL_GLM,
        );
        // Raven: `assert(bAlreadyCached == bAlreadyFound);` — compiled out
        // under `-DNDEBUG` (house convention), kept as a debug assert.
        debug_assert_eq!(
            *already_cached, already_found,
            "bAlreadyCached == bAlreadyFound"
        );
        debug_assert_eq!(
            ptr as usize % 16,
            0,
            "AlignedBytes base must be 16-byte aligned"
        );

        let mdxm = ptr as *mut mdxmHeader_t;
        self.models.slot_mut(idx).mdxm = mdxm;

        if !already_found {
            // "horrible new hackery" — the one-time ingest copy (`TRM-D4`
            // (a)/ruling 58); `assert(mdxm == buffer)` doesn't hold under
            // that divergence and is dropped, not ported (§19).
            *already_cached = true;

            // SAFETY: `mdxm` is the just-copy-constructed 16-byte-aligned
            // `AlignedBytes` base (`TRM-D4`/ruling 58); the debug alignment
            // assert above covers the cast, per §D11 — matching
            // `server_load_mdxm`'s identical quarantine.
            unsafe {
                (*mdxm).ident = LittleLong((*mdxm).ident);
                (*mdxm).version = LittleLong((*mdxm).version);
                (*mdxm).numLODs = LittleLong((*mdxm).numLODs);
                (*mdxm).ofsLODs = LittleLong((*mdxm).ofsLODs);
                (*mdxm).numSurfaces = LittleLong((*mdxm).numSurfaces);
                (*mdxm).ofsSurfHierarchy = LittleLong((*mdxm).ofsSurfHierarchy);
                (*mdxm).ofsEnd = LittleLong((*mdxm).ofsEnd);
            }
        }

        // first up, go load in the animation file we need that has the
        // skeletal animation info for this model.
        // SAFETY: `mdxm` is the aligned, live block either way; `animName`
        // is never itself byte-swapped (it's a char array).
        let anim_name = unsafe { read_qpath(&(*mdxm).animName) };
        let anim_filename = format!("{}.gla", anim_name);
        // See the doc comment above: the SCC-345 mutual-recursion edge back
        // into `tr_model/frontend.rs`.
        let anim_index = RE_RegisterModel(
            qs,
            world_load,
            assets,
            view,
            cvars,
            self,
            img_state,
            sky_view,
            world_effects,
            &anim_filename,
        );
        // SAFETY: as above.
        unsafe {
            (*mdxm).animIndex = anim_index;
        }

        if anim_index == 0 {
            // SAFETY: as above.
            let mesh_name = unsafe { read_qpath(&(*mdxm).name) };
            com_printf(
                view.common,
                &format!(
                    "{}R_LoadMDXM: missing animation file {} for mesh {}\n",
                    warn, anim_name, mesh_name
                ),
            );
            return false;
        }

        // copy this up to the model for ease of use - it wil get inced
        // after this.
        // SAFETY: as above.
        let num_lods = unsafe { (*mdxm).numLODs };
        self.models.slot_mut(idx).numLods = num_lods - 1;

        if already_found {
            // All done. Stop, go no further, do not LittleLong(), do not
            // pass Go...
            return true;
        }

        // SAFETY: as above.
        let is_an_old_model_file =
            unsafe { (*mdxm).numBones == 72 && anim_name.contains("_humanoid") };

        // SAFETY: every pointer walk below stays inside the `AlignedBytes`
        // block the cache entry owns (its size is the file's `ofsEnd`), off
        // the 16-byte-aligned base asserted above (§D11).
        unsafe {
            let base = ptr;
            let num_surfaces = (*mdxm).numSurfaces;

            let mut surf_info =
                base.add((*mdxm).ofsSurfHierarchy as usize) as *mut mdxmSurfHierarchy_t;
            for _ in 0..num_surfaces {
                (*surf_info).numChildren = LittleLong((*surf_info).numChildren);
                (*surf_info).parentIndex = LittleLong((*surf_info).parentIndex);

                // just in case
                Q_strlwr(&mut (*surf_info).name);
                // remove "_off" from name (§19: guard the 4-byte tail read
                // against a short name — see doc comment above). `name_len`
                // is C `strlen` — bytes before the NUL — not the decoded
                // `String`'s `.len()`: `read_qpath` widens every byte >= 0x80
                // to a 2-byte UTF-8 char, so the two diverge on a non-ASCII
                // surface name.
                let (name_len, ends_in_off) = {
                    let name = &(*surf_info).name;
                    let len = name.iter().position(|&c| c == 0).unwrap_or(name.len());
                    (
                        len,
                        len >= 4
                            && name[len - 4..len]
                                .iter()
                                .zip(b"_off")
                                .all(|(&c, &b)| c as u8 == b),
                    )
                };
                if ends_in_off {
                    (*surf_info).name[name_len - 4] = 0;
                }

                // do all the children indexs
                let num_children = (*surf_info).numChildren;
                let child_indexes = core::ptr::addr_of_mut!((*surf_info).childIndexes) as *mut i32;
                for j in 0..num_children as usize {
                    let child = child_indexes.add(j);
                    *child = LittleLong(*child);
                }

                // get the shader name
                // Source: oracle/codemp/renderer/tr_ghoul2.cpp:4926-4938
                let shader_name = read_qpath(&(*surf_info).shader);
                let sh = R_FindShader(
                    &shader_name,
                    &lightmapsNone,
                    &stylesDefault,
                    true,
                    qs,
                    world_load,
                    assets,
                    view,
                    cvars,
                    self,
                    img_state,
                    sky_view,
                );
                // insert it in the surface list
                let is_default_shader = assets
                    .shaders
                    .get(sh)
                    .map(|shader| shader.default_shader)
                    .unwrap_or(false);
                (*surf_info).shaderIndex = if is_default_shader {
                    0
                } else {
                    // DEC-42.2: the arena slot number IS `shader_t::index`.
                    sh.index() as i32
                };

                let name_offset =
                    (core::ptr::addr_of!((*surf_info).shader) as usize - base as usize) as i32;
                let poke_offset =
                    (core::ptr::addr_of!((*surf_info).shaderIndex) as usize - base as usize) as i32;
                self.re_register_models_store_shader_request(mod_name, name_offset, poke_offset);

                // find the next surface
                let surf_info_size = core::mem::offset_of!(mdxmSurfHierarchy_t, childIndexes)
                    + (num_children as usize) * core::mem::size_of::<i32>();
                surf_info = (surf_info as *mut u8).add(surf_info_size) as *mut mdxmSurfHierarchy_t;
            }

            // swap all the LOD's (we need to do the middle part of this even
            // for intel, because of shader reg and err-check)
            let mdxm = ptr as *mut mdxmHeader_t;
            let mut lod = base.add((*mdxm).ofsLODs as usize) as *mut mdxmLOD_t;
            for _ in 0..(*mdxm).numLODs {
                (*lod).ofsEnd = LittleLong((*lod).ofsEnd);

                // swap all the surfaces
                let mut surf = (lod as *mut u8).add(
                    core::mem::size_of::<mdxmLOD_t>()
                        + (num_surfaces as usize) * core::mem::size_of::<mdxmLODSurfOffset_t>(),
                ) as *mut mdxmSurface_t;
                for _ in 0..num_surfaces {
                    (*surf).numTriangles = LittleLong((*surf).numTriangles);
                    (*surf).ofsTriangles = LittleLong((*surf).ofsTriangles);
                    (*surf).numVerts = LittleLong((*surf).numVerts);
                    (*surf).ofsVerts = LittleLong((*surf).ofsVerts);
                    (*surf).ofsEnd = LittleLong((*surf).ofsEnd);
                    (*surf).ofsHeader = LittleLong((*surf).ofsHeader);
                    (*surf).numBoneReferences = LittleLong((*surf).numBoneReferences);
                    (*surf).ofsBoneReferences = LittleLong((*surf).ofsBoneReferences);

                    if (*surf).numVerts > SHADER_MAX_VERTEXES as i32 {
                        com_error(
                            errorParm_t::ERR_DROP,
                            format!(
                                "R_LoadMDXM: {} has more than {} verts on a surface ({})",
                                mod_name,
                                SHADER_MAX_VERTEXES,
                                (*surf).numVerts
                            ),
                        );
                    }
                    if (*surf).numTriangles * 3 > SHADER_MAX_INDEXES as i32 {
                        com_error(
                            errorParm_t::ERR_DROP,
                            format!(
                                "R_LoadMDXM: {} has more than {} triangles on a surface ({})",
                                mod_name,
                                SHADER_MAX_INDEXES / 3,
                                (*surf).numTriangles
                            ),
                        );
                    }

                    // change to surface identifier
                    (*surf).ident = surfaceType_t::SF_MDX as i32;

                    // `#ifndef _M_IX86` bone-ref/triangle/vertex swaps
                    // (`:4980-5024`) — §20-dropped (`TRM-D3`/ruling 54).

                    if is_an_old_model_file {
                        let bone_ref =
                            (surf as *mut u8).add((*surf).ofsBoneReferences as usize) as *mut i32;
                        for j in 0..(*surf).numBoneReferences as usize {
                            let slot = bone_ref.add(j);
                            // Raven: `assert(boneRef[j] >= 0 && boneRef[j] <
                            // 72);` — compiled out under `-DNDEBUG` (house
                            // convention), kept as a debug assert.
                            debug_assert!(
                                (0..72).contains(&*slot),
                                "boneRef[j] >= 0 && boneRef[j] < 72"
                            );
                            *slot = if (0..72).contains(&*slot) {
                                OldToNewRemapTable[*slot as usize]
                            } else {
                                0
                            };
                        }
                    }

                    // find the next surface
                    surf = (surf as *mut u8).add((*surf).ofsEnd as usize) as *mut mdxmSurface_t;
                }

                // find the next LOD
                lod = (lod as *mut u8).add((*lod).ofsEnd as usize) as *mut mdxmLOD_t;
            }
        }

        self.store_parsed_mdxm(mod_name);
        true
    }
}

/// Raven `#define GHOUL2_NORENDER 0x002` (`ghoul2_shared.h:230`) — skip this
/// model's render pass.
const GHOUL2_NORENDER: i32 = 0x002;
/// Raven `#define GHOUL2_NOMODEL 0x004` (`ghoul2_shared.h:231`) — this model
/// slot carries no model.
const GHOUL2_NOMODEL: i32 = 0x004;

/// Raven `void R_AddGhoulSurfaces( trRefEntity_t *ent )` — the per-frame
/// entry point that culls, sorts, transforms and renders every Ghoul2
/// construct bolted to `ent`.
///
/// `ent->e.ghoul2` is decoded into `handle` at the entry seam
/// (`R_AddEntitySurfaces`, `tr_main.rs`), so this body reaches the instance
/// list through `CGhoul2Info_v { mItem: handle }` and the threaded
/// `&mut Ghoul2System` (design point 1/2). The oracle's inline per-model
/// transform loop is `g2_construct_render_skeleton` (`mp_engine_ghoul2`): it
/// computes the root matrix (`RootMatrix`), sorts the models, and transforms
/// each render-visible model off its bolt or the root matrix, exactly as this
/// oracle body does before it renders. It returns the sorted model list, so the
/// render loop reads the built caches in the same order with no second sort.
///
/// This body calls `R_SetupEntityLighting` (`:3438-3443`) and the caller folds
/// the lit fields onto `entities[n]` through `write_back_lighting`. The backend
/// deform that reads the lit color is a later wave.
///
/// DEFERRED in this body:
/// - The `bInShadowRange` shadow-plane adjust (`:3525-3528`) — `bInShadowRange`
///   is still a marked stub in this file (needs `r_shadowRange` plus the shadow
///   backend).
/// - `_G2_GORE` (`gore`/`gore_shader`) — gore rendering is a later wave.
///
/// Source: `oracle/codemp/renderer/tr_ghoul2.cpp:3383-3538`
#[doc(alias = "R_AddGhoulSurfaces")]
#[allow(clippy::too_many_arguments)]
pub fn r_add_ghoul_surfaces<'a>(
    ent: &mut RefEntity,
    handle: Ghoul2Handle,
    host: &mut EngineHostView,
    assets: &RenderAssets,
    models: &RenderModels,
    view: &viewParms_t,
    ori: &orientationr_t,
    cvars: RenderCvarSnapshot,
    warnings: &mut WalkWarnings,
    world_load: &WorldLoadState,
    frame: &FrameState,
    g2: &mut Ghoul2System,
    fogs: &[fog_t],
    refdef_rdflags: i32,
    shifted_entity_num: i32,
    dlights: &[dlight_t],
    draw_surfs: &mut Vec<DrawSurf<SurfaceGeometry<'a>>>,
) {
    let mut ghoul2 = CGhoul2Info_v { mItem: handle.0 };

    if !ghoul2.is_valid(g2) {
        return;
    }

    // if we don't want server ghoul2 models and this is one, or we just don't
    // want ghoul2 models at all, then return
    if cvars.no_server_ghoul2 != 0 {
        return;
    }

    if !g2_setup_model_pointers_v(g2, host, &ghoul2) {
        return;
    }

    let current_time = g2api_get_time(g2, frame.refdef.time);

    // cull the entire model if the merged bounding box is outside the frustum
    let r_nocull_integer = cvars.nocull;
    let mut cull_out = 0;
    let mut cull_in = 0;
    let mut cull_clip = 0;
    let cull = r_g_cull_model(
        ent,
        ori,
        r_nocull_integer,
        &view.frustum,
        &mut cull_out,
        &mut cull_in,
        &mut cull_clip,
    );
    if cull == CULL_OUT {
        return;
    }

    // don't add third_person objects if not in a portal
    let personal_model = (ent.renderfx & RF_THIRD_PERSON) != 0 && view.isPortal == 0;

    // set up lighting now that we know we aren't culled. The oracle guards the
    // call with the non-`VV_LIGHTING` arm `!personalModel || r_shadows->integer
    // > 1`. The caller folds the lit fields back onto `entities[n]`.
    // Source: oracle/codemp/renderer/tr_ghoul2.cpp:3438-3443
    if !personal_model || cvars.shadows > 1 {
        R_SetupEntityLighting(cvars, assets, world_load, frame, refdef_rdflags, dlights, ent);
    }

    // see if we are in a fog volume
    let fog_num = r_g_compute_fog_num(ent, fogs, refdef_rdflags);

    // Transform every render-visible model's skeleton (root matrix + sort +
    // per-model bolt or root transform, flags-gated). This is the oracle's
    // inline transform loop. It returns the sort order so bolt-ons render
    // against the right parent model without a second sort.
    let model_list = g2_construct_render_skeleton(g2, host, &mut ghoul2, current_time, ent.model_scale);
    let rdf_nofog = refdef_rdflags & RDF_NOFOG != 0;

    // walk each model for this entity and render it out
    for &i in &model_list {
        let item = ghoul2.mItem;
        let inst = &g2.info_array.get(item)[i as usize];

        if !inst.valid
            || (inst.flags & GHOUL2_NOMODEL) != 0
            || (inst.flags & GHOUL2_NORENDER) != 0
        {
            continue;
        }

        // figure out the custom shader or the custom skin for this model
        let (cust_shader, skin): (Option<ShaderHandle>, Option<SkinHandle>) =
            if ent.custom_shader != 0 {
                (
                    Some(R_GetShaderByHandleQuiet(assets, ent.custom_shader, warnings)),
                    None,
                )
            } else if inst.custom_skin != 0 {
                (None, Some(R_GetSkinByHandle(assets, inst.custom_skin)))
            } else if ent.custom_skin != 0 {
                (None, Some(R_GetSkinByHandle(assets, ent.custom_skin)))
            } else if inst.skin > 0
                && u32::try_from(inst.skin)
                    .ok()
                    .and_then(|slot| assets.skins.handle_at_slot(slot))
                    .is_some()
            {
                // Raven guards this arm with `mSkin > 0 && mSkin < tr.numSkins`
                // (`:3489`). An out-of-range `mSkin` leaves `skin` NULL and
                // falls through to the surface's own shader, so the arm entry
                // needs the registered-slot probe, not just `mSkin > 0`.
                (None, Some(R_GetSkinByHandle(assets, inst.skin)))
            } else {
                (None, None)
            };

        let current_model = models.get_model(inst.model);
        let which_lod = g2_compute_lod(ent, current_model, inst.lod_bias, view, cvars);

        // The bone transforms already ran through `g2_construct_render_skeleton`
        // above, so the render only reads the built cache. Clone the surface
        // and bolt lists into owned locals so the render surface borrows them
        // instead of the arena (which the loop still reads by index).
        let root_s_list = inst.slist.clone();
        let mut bolt_list = inst.bltlist.clone();
        let bone_cache = inst.bone_cache;
        let surface_root = inst.surface_root;
        // `render_surfaces` reads the model through its own `current_model`
        // parameter, so the render surface's own `current_model` field is unread
        // and stays `None`.
        let model_handle = None;

        // DEFERRED: the `RF_SHADOW_PLANE`/`bInShadowRange` `RF_NOSHADOW` adjust
        // (`:3525-3528`) — `bInShadowRange` needs the shadow backend. The
        // `RF_SHADOW_PLANE`/`RF_NOSHADOW` imports return with that wave.
        // Source: oracle/codemp/renderer/tr_ghoul2.cpp:3525-3528
        let renderfx = ent.renderfx;

        let mut rs = CRenderSurface::new(
            surface_root,
            &root_s_list,
            cust_shader,
            fog_num,
            personal_model,
            bone_cache,
            renderfx,
            skin,
            model_handle,
            which_lod,
            &mut bolt_list,
            None,
            None,
        );

        render_surfaces(
            &mut rs,
            current_model,
            assets,
            shifted_entity_num,
            rdf_nofog,
            draw_surfs,
        );
    }
}
