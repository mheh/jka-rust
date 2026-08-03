//! Raven `tr_terrain.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_terrain.cpp`

#![allow(non_snake_case)]

// PORT-NOTE (wave 0, `tr_terrain.wave0.md`): mirrors `tr_main.wave0`'s own
// PORT-NOTE. `FrameState::view`/`::refdef` and `RenderAssets::world` are
// still empty landing placeholders (`render_state/placeholders.rs`'s
// `ViewParms`/`TrRefdef`/`WorldAsset`) or not yet populated, so the
// functions below thread the concrete, already-ported tier-2 shapes
// (`viewParms_t`, `trRefdef_t`, `srfTerrain_t`) as explicit parameters
// instead of the not-yet-populated R2 carrier types, per the
// interior-safety law's own carve-out ("Tier-2 fields may be *read*
// through their existing shapes until their owning wave replaces them")
// and porting-rules §B4. Flagged for the integrator: once those carrier
// fields land with real shapes, call sites here take carrier slices
// instead.
//
// PORT-NOTE (collision-side landscape): Raven's `CCMLandScape`/`CCMPatch` are
// ported as `mp_engine_qcommon::cm_terrain::CmLandScape` /
// `cm_patch::CmPatch`. The renderer reaches them through the `CTRLandScape::
// common` / `CTRPatch::owner`/`::common` back-pointers; those fields stay as
// `*mut c_void` for ABI layout only and are never dereferenced here — the
// carriers are threaded in as `&CmLandScape` / `&CmPatch` parameters (§B4/
// §B5), the same way `refdef`/`assets`/`cvars` are. Raven's three back-pointer
// writes in `InitRendererPatches` (`SetCommon`/`SetOwner`/`SetLocalOwner`) are
// what that threading replaces, so they are not re-created as pointer stores.
//
// Still genuinely deferred (nothing in this crate provides them yet):
// `CalculateShaders` (`CTRLandScape::GetBlendedShader`, wave 11, no longer
// blocks it — see that fn's own doc; the remaining blocker is the
// `mSortedPatches`/`mSortedCount` fill + closing `qsort`); `WorldAsset
// ::globalFog` (tr_bsp/tr_world R3 wave) and the zero-arg `CTRLandScape
// ::Render()`, and with them `RB_SurfaceTerrain`; the `CTRLandScape(const
// char *)` ctor, and with it `RE_InitRendererTerrain` and
// `R_TerrainShutdown`'s teardown arm. `tess` writes (`RenderCorner`/`Render`/
// `RenderWaterVert`) are DISSOLVED per R2 `## State ownership` (an R4
// concern) and carry no body at all.

use core::ffi::c_int;

use native_math::qmath::Com_Clampi;
use native_string::atoi::atoi;
use native_types::thandle_t;

use mp_engine_qcommon::cm_load::CM_ShutdownTerrain;
use mp_engine_qcommon::cm_patch::CmPatch;
use mp_engine_qcommon::cm_terrain::CmLandScape;
use mp_engine_qcommon::cm_trace::CM_CullWorldBox;
use mp_engine_qcommon::collision_world::CollisionWorld;
use mp_engine_qcommon::common::common::{com_printf, Common};
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common_fns::{Com_ParseTextFile, Com_ParseTextFileDestroy};
use mp_engine_qcommon::cvar_fns::Cvar_Get;
use mp_engine_qcommon::gp2::generic_parser2::GenericParser2;
use mp_engine_qcommon::z_memman_pc::Z_Free;
use mp_qshared::shared::com_parse::QSharedScratch;
use mp_qshared::shared::cvar::CVAR_CHEAT;
use mp_qshared::shared::q_math::{
    _DotProduct, _VectorAdd, _VectorMA, _VectorScale, _VectorSubtract, CrossProduct,
    DistanceSquared, VectorNormalize,
};
use mp_qshared::shared::{qhandle_t, vec3_t};

use crate::render_state::frame_state::FrameState;
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::render_cvar_snapshot::RenderCvarSnapshot;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::tr_image::TrImageState;
use crate::tr_landscape::ctrland_scape::CTRLandScape;
use crate::tr_landscape::ctrpatch::CTRPatch;
use crate::tr_landscape::spatch_info::TPatchInfo;
use crate::tr_light::R_LightForPoint;
use crate::tr_local::srf_terrain_s::srfTerrain_t;
use crate::tr_local::surface_type_t::surfaceType_t;
use crate::tr_local::tr_refdef_t::trRefdef_t;
use crate::tr_local::view_parms_t::viewParms_t;
use crate::tr_main::{DrawSurf, R_AddDrawSurf, SurfaceGeometry};
use crate::tr_model::render_models::RenderModels;
use crate::tr_public::ref_flags::{RDF_NOFOG, RDF_NOWORLDMODEL};
use crate::tr_shader::{RE_RegisterShader, R_CreateBlendedShader, R_GetShaderByHandle};
use crate::tr_sky::SkyState;
use crate::tr_surface::RB_CheckOverflow;

/// Raven `HEIGHT_RESOLUTION` — the size of `CTRLandScape::mHeightDetails[]`.
/// Already defined (module-private) in
/// `crates/mp/renderer/src/tr_landscape/ctrland_scape.rs`; repeated here
/// (same crate, but that constant is not `pub`) rather than widening that
/// file's visibility from this wave — mirrors `mp_engine_qcommon::cm_terrain`'s
/// own repeated-locally treatment of the same oracle constant.
///
/// Source: `oracle/codemp/qcommon/cm_landscape.h:13`
const HEIGHT_RESOLUTION: usize = 256;

/// Raven `PI_TOP` — the `SPatchInfo::mPart` top-triangle flag.
///
/// Source: `oracle/codemp/renderer/tr_landscape.h:104`
const PI_TOP: c_int = 1;

/// Per-terrain-subsystem render-thread state (DEC-37 A13.3 — NAMED BY THIS
/// WAVE): `TerrainDistanceCull`/`TerrainFog` are file-scope statics in
/// `tr_terrain.cpp` with no R2 row of their own. Both are render-side only
/// (no sim/render boundary crossing — `SetVisibility`/`Reset` read/write
/// `TerrainDistanceCull` and `RB_SurfaceTerrain` writes `TerrainFog`, all on
/// the render thread), so a plain owned field pair suffices; no `static`.
///
/// Source: `oracle/codemp/renderer/tr_terrain.cpp` (file-scope statics, not
/// enumerated in `tr_local.h`)
#[derive(Default)]
pub struct TerrainState {
    /// Raven `float TerrainDistanceCull` — the squared cull distance for the
    /// landscape's current reset pass.
    pub distance_cull: f32,
    /// Raven `fog_t *TerrainFog` — the world's global fog for whichever
    /// terrain surface is currently drawing. An index into the eventual
    /// `WorldAsset` fog list (interior-safety law: no raw pointer), left
    /// unpopulated this wave — `RB_SurfaceTerrain`, its only writer, is
    /// itself deferred below (`WorldAsset` has no `globalFog`-equivalent
    /// field yet; `tr_bsp`/`tr_world` R3 wave).
    pub fog: Option<usize>,
}

// DEFERRED: R4 — `CTRPatch::RenderCorner` writes only into `tess`, DISSOLVED
// into R4's tessellation/vertex-building pipeline (R2 `## State ownership`
// row `tess`: "no single global scratch buffer survives the new topology").
// No CPU logic survives independent of that buffer, so no stub body is
// written here (mirrors this crate's existing pure-GL `tr_init.rs`
// `GL_CheckErrors` treatment).
// Source: `oracle/codemp/renderer/tr_terrain.cpp:31-56`

// DEFERRED: R4 — `CTRPatch::Render` funnels entirely into `tess` (DISSOLVED,
// same `## State ownership` row as `RenderCorner`) via `RecurseRender`, which
// is itself not among this wave's 22 packet functions (absent from the
// RESOLVED CALL SURFACE — a wave-planning gap, not guessed here). No CPU
// logic survives independent of either blocker, so no stub body is written.
// Source: `oracle/codemp/renderer/tr_terrain.cpp:121-153`

// DEFERRED: R4 — `CTRPatch::RenderWaterVert` writes only into `tess`
// (DISSOLVED, same `## State ownership` row). No stub body is written.
// Source: `oracle/codemp/renderer/tr_terrain.cpp:158-182`

impl CTRPatch {
    /// Raven `CTRPatch::HasWater`. `owner` (`CCMLandScape`) and `common`
    /// (`CCMPatch`) are threaded in (§B4); `r_terrainWaterOffset->integer` is
    /// the live read through the cached handle.
    ///
    /// Source: `oracle/codemp/renderer/tr_terrain.cpp:205-209`
    pub fn has_water(
        &self,
        owner: &mut CmLandScape,
        common: &CmPatch,
        engine: &Common,
        cvars: &RendererCvars,
    ) -> bool {
        let offset = engine.cvar(cvars.r_terrainWaterOffset).integer;
        owner.set_real_water_height(owner.base_water_height() + offset);
        common.bounds[0][2] < owner.water_height()
    }

    /// Raven `CTRPatch::SetVisibility`.
    ///
    /// Source: `oracle/codemp/renderer/tr_terrain.cpp:213-231`
    pub fn set_visibility(
        &mut self,
        vis_check: bool,
        common: &CmPatch,
        refdef: &trRefdef_t,
        view: &viewParms_t,
        terrain: &TerrainState,
    ) {
        if vis_check {
            if DistanceSquared(self.mCenter, refdef.vieworg) > terrain.distance_cull {
                self.misVisible = false;
            } else {
                // Set the visibility of the patch
                self.misVisible = !CM_CullWorldBox(view.frustum.as_ptr(), common.bounds);
            }
        } else {
            self.misVisible = true;
        }
    }

    /// Raven `CTRPatch::RecurseRender` — recursively subdivides a
    /// right-triangle span of the patch (`left`/`right`/`apex`) down to leaf
    /// triangles, emitting one leaf triangle per bottomed-out recursion.
    ///
    /// `ivec5_t` (`int[5]`: xy grid indices in `[0]`/`[1]`, four packed alpha
    /// bytes reinterpreted through `[2]`, the vert index in `[3]`, the
    /// tessellation-registration flag in `[4]`) has no existing alias in this
    /// crate — represented inline as `[i32; 5]`, matching this file's own
    /// `vec3_t = [f32; 3]` treatment of other oracle fixed-size arrays.
    ///
    /// Raven reinterprets `&left[2]`/`&right[2]`/`&center[2]` as `byte*` to
    /// average the four packed alpha bytes in place; transcribed via
    /// `to_ne_bytes`/`from_ne_bytes` (native-endian, matching the host-order
    /// pointer aliasing Raven relies on) rather than a raw pointer
    /// reinterpret cast, which the interior-safety law forbids.
    ///
    /// `RB_CheckOverflow`'s ported signature threads `&mut FrameState`
    /// (§B4) in place of the `tess`-adjacent globals it reads.
    ///
    /// Panics via `RB_CheckOverflow`'s loud stub until its owning wave lands
    /// — that callee is still a `todo!()` in `tr_surface.rs` for the `tess`
    /// gap its own doc names. The subdivision/alpha-averaging logic above it
    /// has no `tess` dependency and is transcribed in full.
    ///
    /// Source: `oracle/codemp/renderer/tr_terrain.cpp:58-114`
    pub fn RecurseRender(
        &self,
        mut depth: i32,
        left: [i32; 5],
        right: [i32; 5],
        apex: [i32; 5],
        frame: &mut FrameState,
    ) {
        // All non-leaf nodes have both children, so just check for one
        if depth >= 0 {
            let mut center: [i32; 5] = [0; 5];

            // Work out the centre of the hypoteneuse
            center[0] = (left[0] + right[0]) >> 1;
            center[1] = (left[1] + right[1]) >> 1;

            // Work out the relevant texture coefficients at that point
            let left_alphas = left[2].to_ne_bytes();
            let right_alphas = right[2].to_ne_bytes();
            let mut center_alphas = [0u8; 4];

            center_alphas[0] = ((left_alphas[0] as i32 + right_alphas[0] as i32) >> 1) as u8;
            center_alphas[1] = ((left_alphas[1] as i32 + right_alphas[1] as i32) >> 1) as u8;
            center_alphas[2] = ((left_alphas[2] as i32 + right_alphas[2] as i32) >> 1) as u8;
            center_alphas[3] = ((left_alphas[3] as i32 + right_alphas[3] as i32) >> 1) as u8;
            center[2] = i32::from_ne_bytes(center_alphas);

            // Make sure the vert index and tesselation registration are not set
            center[3] = -1;
            center[4] = 0;

            if apex[0] == left[0] && apex[0] == center[0] {
                depth = 0;
            }

            self.RecurseRender(depth - 1, apex, left, center, frame);
            self.RecurseRender(depth - 1, right, apex, center, frame);
        } else {
            if left[0] == right[0] && left[0] == apex[0] {
                return;
            }
            if left[1] == right[1] && left[1] == apex[1] {
                return;
            }
            // A leaf node!  Output a triangle to be rendered.
            RB_CheckOverflow(4, 4, frame);

            // assert(left[0] != right[0] || left[1] != right[1]);
            // assert(left[0] != apex[0] || left[1] != apex[1]);

            // DEFERRED: R4 — `RenderCorner` writes only into the DISSOLVED
            // `tess` global (R2 `## State ownership` row `tess`); its
            // Rust body is deliberately unwritten (this file's `RenderCorner`
            // note above), so these three calls have no target to invoke.
            // The `RB_CheckOverflow(4, 4)` call above is transcribed as-is;
            // it panics through that fn's own loud `todo!()` stub
            // (`tr_surface.rs`) until the `tess` wave lands it.
            // Source: oracle/codemp/renderer/tr_terrain.cpp:110-112
            let _ = (left, right, apex);
        }
    }

    /// Raven `CTRPatch::RenderWater` — emits the two triangles covering this
    /// patch's water surface.
    // DEFERRED: R4 — `RenderWaterVert` (this file's `RenderWaterVert`
    // DEFERRED note above) has no Rust body to call, and the four
    // TL/TR/BL/BR indexes it would return feed directly into the
    // `tess.indexes` writes, DISSOLVED (R2 `## State ownership` row `tess`).
    // No CPU logic survives independent of either blocker, so this lands as
    // a whole-fn loud `todo!()`. The leading `RB_CheckOverflow(4, 6)`
    // (`oracle/codemp/renderer/tr_terrain.cpp:186`) is NOT transcribed ahead
    // of it: a call whose only successor is an unconditional panic is dead,
    // and it re-lands with the real body when this fn's wave writes it.
    // Source: `oracle/codemp/renderer/tr_terrain.cpp:184-203`
    pub fn RenderWater(&self, frame: &mut FrameState) {
        let _ = frame;
        todo!("Port CTRPatch::RenderWater — oracle/codemp/renderer/tr_terrain.cpp:189-202")
    }
}

impl CTRLandScape {
    /// Raven `CTRLandScape::CTRLandScape` — constructs the renderer-side
    /// landscape from a `.terrain`/RMG config string: registers the
    /// collision-side `CCMLandScape`, links the two back-pointers, allocates
    /// and fills `mRenderMap`/`mTRPatches`/`mSortedPatches`, walks the
    /// collision patches to build the matching renderer patches, and
    /// resolves the terrain's contents shader.
    ///
    /// Source: `oracle/codemp/renderer/tr_terrain.cpp:875-938`
    // DEFERRED: whole-fn — the ctor has no meaningful partial transcription;
    // five independent, individually-verified blockers (wave-10 grep, not
    // assumed) close it end-to-end between them. Blocker 1 denies `common`
    // itself (needed from `:889` on); blocker 5 denies `this` — needed even
    // by the *leading* `memset(this, 0, sizeof(*this))`/
    // `tr.landScape.landscape = this` writes (`:881,887`, neither of which
    // reads `common`) and by the already-landed `LoadTerrainDef`'s `&mut
    // self` method call (`:890`, likewise no `common` read). No step in the
    // body has both an available receiver and an available `common`.
    //   1. `CM_RegisterTerrain(configstring, false)` (`:884`) returns
    //      `const CCMLandScape *` in oracle. The workspace's
    //      `mp_engine_qcommon::cm_terrain::register_terrain` folds Raven's
    //      ctor+`CM_InitTerrain` into one get-or-create call, but returns a
    //      `TerrainHandle` (always `0`) and needs `&mut CollisionWorld` +
    //      `&mut impl EngineHost` threaded in — neither of which this ctor's
    //      only call site, `RE_InitRendererTerrain` (below), carries today.
    //      Retrieving the `&CmLandScape` back from the handle is possible in
    //      principle (`CollisionWorld::land_scape` is the single global slot
    //      the always-`0` handle names) but widening
    //      `RE_InitRendererTerrain`'s signature reaches every one of its
    //      external callers, out of this single-file wave's scope.
    //   2. `common->GetSize()` (`:893`, `CCMLandScape::GetSize`/`mSize`) has
    //      no live Rust accessor: `cm_terrain.rs`'s `CmLandScape::size` field
    //      is `#[allow(dead_code)]` with its own doc note that "its live
    //      reader ... lands with a later terrain-consuming slice"
    //      (`crates/mp/engine/qcommon/src/cm_terrain.rs`).
    //   3. `CM_TerrainPatchIterate(common, InitRendererPatches, this)`
    //      (`:923`) — the iterator that drives this file's own
    //      already-ported [`InitRendererPatches`] callback — has no Rust
    //      definition anywhere in the workspace.
    //      `crates/mp/engine/qcommon/src/cm_terrain.rs`'s own module doc
    //      lists `CM_TerrainPatchIterate`/`TerrainPatchIterate` under
    //      "§20-dropped ... renderer + dead generation-path callers only":
    //      judged to have no live caller and intentionally not ported. This
    //      ctor is in fact that live renderer caller, so that ruling is
    //      stale — reconciling it is an escalation for the state-ownership
    //      docs, not an invention at this call site.
    //   4. `CalculateShaders()` (`:926`) is this file's own whole-fn
    //      `todo!()` below (`GetBlendedShader` itself landed wave 11, but
    //      the `mSortedPatches`/`mSortedCount` fill + closing `qsort` are
    //      still unwritten). `mShader = R_GetShaderByHandle(R_GetShaderByNum(
    //      shaderNum, *tr.world))` (`:930`) needs the world registry this
    //      file's header PORT-NOTE already flags as "still empty landing
    //      placeholders" pending the `tr_bsp`/`tr_world` R3 wave.
    //   5. `this` is heap-constructed at the `new CCMLandScape(...)` call
    //      site (`RE_InitRendererTerrain`, not this ctor), not
    //      stack-returned — this file's `R_TerrainShutdown` DEFERRED note
    //      above already records that "`CTRLandScape`'s owning-allocation
    //      strategy is unsettled while its ctor ... is deferred".
    // `configstring` is threaded to match the oracle signature; unused since
    // the body never runs.
    pub fn new(configstring: &str) -> Self {
        let _ = configstring;
        todo!("Port CTRLandScape::CTRLandScape — oracle/codemp/renderer/tr_terrain.cpp:875-938")
    }

    /// Raven `CTRLandScape::Reset` — resets all patches, recomputing variance
    /// if needed.
    ///
    /// Source: `oracle/codemp/renderer/tr_terrain.cpp:267-284`
    pub fn reset(
        &mut self,
        vis_check: bool,
        land: &CmLandScape,
        assets: &RenderAssets,
        refdef: &trRefdef_t,
        view: &viewParms_t,
        terrain: &mut TerrainState,
    ) {
        let mut distance_cull = assets.distance_cull + self.mPatchSize;
        distance_cull *= distance_cull;
        terrain.distance_cull = distance_cull;

        let block_width = land.block_width();

        // Go through the patches performing resets, compute variances, and
        // linking.
        // Raven's `x++, patch++` increment steps a `patch` the loop body
        // immediately overwrites with `GetPatch(x, y)` — dead, dropped (§C10).
        for y in self.mPatchMiny..self.mPatchMaxy {
            for x in self.mPatchMinx..self.mPatchMaxx {
                let common = land.patch(x, y);
                let patch = self.patch_mut(x, y, block_width);
                patch.set_visibility(vis_check, common, refdef, view, terrain);
            }
        }
    }

    /// Raven `CTRLandScape::Render` — renders every visible patch, then all
    /// water for visible patches with water.
    // DEFERRED: R4/tr_bsp — blocked on four independent unresolved
    // dependencies, none satisfiable from this file alone:
    //   1. `tess.shader != current->mShader` reads the DISSOLVED `tess`
    //      global (R2 `## State ownership` row `tess`: "no single global
    //      scratch buffer survives the new topology" — an R4 concern, not
    //      an R3 field).
    //   2. `current->mPatch->Render(current->mPart)` is `CTRPatch::Render`,
    //      already DEFERRED R4 earlier in this file (funnels into `tess`
    //      via `RecurseRender`, itself absent from this wave's packet).
    //   3. Walking `mSortedPatches` (a raw `TPatchInfo *`) to reach
    //      `current->mPatch->isVisible()` has no existing quarantined
    //      accessor on `CTRLandScape` (unlike `mTRPatches`/`mRenderMap`,
    //      which have `patch_mut`/`render_map_mut`) — adding one is a
    //      `ctrland_scape.rs` change, out of this wave's single-file scope,
    //      and a raw deref here would be unsafe, banned in this crate.
    //   4. The water pass's `patch->RenderWater()` is `CTRPatch::RenderWater`
    //      (verbatim `RenderWaterVert` sibling), also DEFERRED R4 above —
    //      writes only into the same DISSOLVED `tess`.
    //   5. `RB_EndSurface`, this fn's only in-module callee besides
    //      `RB_BeginSurface`, is itself an unimplemented `todo!()` stub
    //      upstream (`tr_shade.rs:448`).
    // No CPU logic survives independent of these blockers — matches this
    // file's existing `RenderCorner`/`CTRPatch::Render`/`RenderWaterVert`
    // treatment above, and the header PORT-NOTE's own forward reference to
    // "the zero-arg `CTRLandScape::Render()`".
    // Source: `oracle/codemp/renderer/tr_terrain.cpp:291-334`
    pub fn render(&mut self) {
        todo!("Port CTRLandScape::Render — oracle/codemp/renderer/tr_terrain.cpp:291-334")
    }

    /// Raven `CTRLandScape::CalculateRegion`.
    ///
    /// Raven's `#if _DEBUG mCycleCount++` has no field to increment: the
    /// asserted `CTRLandScape` layout is the release shape.
    ///
    /// Source: `oracle/codemp/renderer/tr_terrain.cpp:336-357`
    pub fn calculate_region(
        &mut self,
        land: &CmLandScape,
        assets: &RenderAssets,
        refdef: &trRefdef_t,
    ) {
        let size = land.patch_size();
        let offset = land.mins();
        let mut mins: vec3_t = [0.0; 3];
        let mut maxs: vec3_t = [0.0; 3];

        mins[0] = refdef.vieworg[0] - assets.distance_cull - (size[0] * 2.0) - offset[0];
        mins[1] = refdef.vieworg[1] - assets.distance_cull - (size[1] * 2.0) - offset[1];

        maxs[0] = refdef.vieworg[0] + assets.distance_cull + (size[0] * 2.0) - offset[0];
        maxs[1] = refdef.vieworg[1] + assets.distance_cull + (size[1] * 2.0) - offset[1];

        self.mPatchMinx = Com_Clampi(0, land.block_width(), (mins[0] / size[0]).floor() as c_int);
        self.mPatchMaxx = Com_Clampi(0, land.block_width(), (maxs[0] / size[0]).ceil() as c_int);

        self.mPatchMiny = Com_Clampi(0, land.block_height(), (mins[1] / size[1]).floor() as c_int);
        self.mPatchMaxy = Com_Clampi(0, land.block_height(), (maxs[1] / size[1]).ceil() as c_int);
    }

    /// Raven `CTRLandScape::CalculateRealCoords`.
    ///
    /// Source: `oracle/codemp/renderer/tr_terrain.cpp:359-377`
    pub fn calculate_real_coords(&mut self, land: &CmLandScape) {
        let real_width = land.real_width();
        let real_height = land.real_height();
        let mins = land.mins();
        let terxel_size = land.terxel_size();
        let real_area = land.real_area() as usize;
        let render_map = self.render_map_mut(real_area);

        // Work out the real world coordinates of each heightmap entry
        for y in 0..real_height {
            for x in 0..real_width {
                let offset = ((y * real_width) + x) as usize;

                // VectorSet(icoords, x, y, mRenderMap[offset].height) then
                // VectorScaleVectorAdd(GetMins(), icoords, GetTerxelSize(), …).
                let icoords: vec3_t = [x as f32, y as f32, render_map[offset].height as f32];
                render_map[offset].coords = [
                    mins[0] + (icoords[0] * terxel_size[0]),
                    mins[1] + (icoords[1] * terxel_size[1]),
                    mins[2] + (icoords[2] * terxel_size[2]),
                ];
            }
        }
    }

    /// Raven `CTRLandScape::CalculateNormals`.
    ///
    /// Source: `oracle/codemp/renderer/tr_terrain.cpp:379-407`
    pub fn calculate_normals(&mut self, land: &CmLandScape) {
        let width = land.width();
        let height = land.height();
        let real_width = land.real_width();
        let real_area = land.real_area() as usize;
        let render_map = self.render_map_mut(real_area);

        let mut offset: c_int = 0;

        // Work out the normals for every face
        for y in 0..height {
            for x in 0..width {
                let mut vcenter: vec3_t = [0.0; 3];
                let mut vleft: vec3_t = [0.0; 3];

                offset = (y * real_width) + x;
                let o = offset as usize;

                _VectorSubtract(render_map[o].coords, render_map[o + 1].coords, &mut vcenter);
                _VectorSubtract(
                    render_map[o].coords,
                    render_map[o + real_width as usize].coords,
                    &mut vleft,
                );

                CrossProduct(vcenter, vleft, &mut render_map[o].normal);
                VectorNormalize(&mut render_map[o].normal);
            }
            // Duplicate right edge condition
            let normal = render_map[offset as usize].normal;
            render_map[offset as usize + 1].normal = normal;
        }
        // Duplicate bottom line
        offset = height * real_width;
        for x in 0..real_width {
            let normal = render_map[(offset - real_width + x) as usize].normal;
            render_map[(offset + x) as usize].normal = normal;
        }
    }

    /// Raven `CTRLandScape::CalculateLighting` — computes each terxel's
    /// vertex normal (averaged from its four attached face normals) and the
    /// resulting lit tint.
    ///
    /// Raven reaches `tr.overbrightBits` through the frontend-scratch global
    /// and `common->GetBaseWaterHeight()` through the `CCMLandScape`
    /// back-pointer; both are threaded explicitly (`frame`/`land`, §B4),
    /// matching this file's established parameter shape.
    /// `R_LightForPoint`'s already-ported idiomatic signature returns
    /// `Option<(ambient, directed, direction)>` in place of the oracle's
    /// `int` return plus three out-params (dictionary: out-params->returns);
    /// `!R_LightForPoint(...)` becomes the `None` arm.
    ///
    /// `Com_Clampi(0.0f, 1.0f, DotProduct(...))` / `Com_Clampi(0.0f, 255.0f,
    /// tint[N])` pass float literals/values to the `int`-typed
    /// `Com_Clampi(c_int, c_int, c_int) -> c_int`; Raven's C compiles this
    /// via an implicit float->int truncating conversion at the call
    /// boundary, preserved faithfully here as an explicit `as c_int` cast
    /// (porting-rules §A2 — no speculative "should be a float clamp"
    /// cleanup). `(byte)Com_Clampi(...) >> tr.overbrightBits` integer-
    /// promotes the byte to `i32` for the shift before truncating back on
    /// assignment, matching this crate's established
    /// `R_ColorShiftLightingBytes` idiom (`tr_bsp.rs`).
    ///
    /// `(1.0 - dp) * 0.5` uses unsuffixed C double literals against a float
    /// `dp`, promoting the whole expression to `f64` before `VectorScale`'s
    /// `f32` parameter narrows it back once — wave-0 ruling 12.
    ///
    /// Source: `oracle/codemp/renderer/tr_terrain.cpp:409-477`
    pub fn calculate_lighting(
        &mut self,
        land: &CmLandScape,
        common: &Common,
        cvars: &RendererCvars,
        assets: &RenderAssets,
        frame: &FrameState,
    ) {
        let width = land.width();
        let height = land.height();
        let real_width = land.real_width();
        let real_area = land.real_area() as usize;
        let base_water_height = land.base_water_height();
        let overbright_bits = frame.overbright_bits;
        // One cvar read for the whole grid: `R_LightForPoint` takes the frame
        // snapshot since W2-F1, and this loop runs per vertex.
        let cvar_snapshot = RenderCvarSnapshot::from_cvars(cvars, common);

        let render_map = self.render_map_mut(real_area);

        let mut offset: c_int = 0;

        // Work out the vertex normal (average of every attached face normal) and apply to the direction of the light
        for y in 0..height {
            for x in 0..width {
                offset = (y * real_width) + x;
                let o = offset as usize;

                // Work out average normal
                let mut total = render_map[o].normal;
                _VectorAdd(total, render_map[o + 1].normal, &mut total);
                _VectorAdd(
                    total,
                    render_map[o + real_width as usize + 1].normal,
                    &mut total,
                );
                _VectorAdd(
                    total,
                    render_map[o + real_width as usize].normal,
                    &mut total,
                );
                VectorNormalize(&mut total);

                let coords = render_map[o].coords;
                let Some((mut ambient, directed, direction)) =
                    R_LightForPoint(cvar_snapshot, assets, frame, coords)
                else {
                    let v = (255i32 >> overbright_bits) as u8;
                    render_map[o].tint[0] = v;
                    render_map[o].tint[1] = v;
                    render_map[o].tint[2] = v;
                    render_map[o].tint[3] = 255;
                    continue;
                };

                if coords[2] < base_water_height as f32 {
                    _VectorScale(ambient, 0.75, &mut ambient);
                }

                // Both normalised, so -1.0 < dp < 1.0
                let dot = _DotProduct(direction, total);
                let mut dp = Com_Clampi(0, 1, dot as c_int) as f32;
                dp = dp.powf(3.0);
                let scale = ((1.0f64 - dp as f64) * 0.5) as f32;
                _VectorScale(ambient, scale, &mut ambient);
                let mut tint: vec3_t = [0.0; 3];
                _VectorMA(ambient, dp, directed, &mut tint);

                let r = ((Com_Clampi(0, 255, tint[0] as c_int) as i32) >> overbright_bits) as u8;
                let g = ((Com_Clampi(0, 255, tint[1] as c_int) as i32) >> overbright_bits) as u8;
                let b = ((Com_Clampi(0, 255, tint[2] as c_int) as i32) >> overbright_bits) as u8;
                render_map[o].tint[0] = r;
                render_map[o].tint[1] = g;
                render_map[o].tint[2] = b;
                render_map[o].tint[3] = 0xff;

                // Raven:
                // mRenderMap[offset].tint[0] += tr.identityLight * 32;
                // mRenderMap[offset].tint[1] += tr.identityLight * 32;
                // mRenderMap[offset].tint[2] += tr.identityLight * 32;
            }
            render_map[offset as usize + 1].tint[0] = render_map[offset as usize].tint[0];
            render_map[offset as usize + 1].tint[1] = render_map[offset as usize].tint[1];
            render_map[offset as usize + 1].tint[2] = render_map[offset as usize].tint[2];
            render_map[offset as usize + 1].tint[3] = 0xff;
        }
        // Duplicate bottom line
        offset = height * real_width;
        for x in 0..real_width {
            render_map[(offset + x) as usize].tint[0] =
                render_map[(offset - real_width + x) as usize].tint[0];
            render_map[(offset + x) as usize].tint[1] =
                render_map[(offset - real_width + x) as usize].tint[1];
            render_map[(offset + x) as usize].tint[2] =
                render_map[(offset - real_width + x) as usize].tint[2];
            render_map[(offset + x) as usize].tint[3] = 0xff;
        }
    }

    /// Raven `CTRLandScape::CalculateTextureCoords`.
    ///
    /// Source: `oracle/codemp/renderer/tr_terrain.cpp:479-493`
    pub fn calculate_texture_coords(&mut self, land: &CmLandScape) {
        let real_width = land.real_width();
        let real_height = land.real_height();
        let terxel_size = land.terxel_size();
        let texture_scale = self.mTextureScale;
        let real_area = land.real_area() as usize;
        let render_map = self.render_map_mut(real_area);

        for y in 0..real_height {
            for x in 0..real_width {
                let offset = ((y * real_width) + x) as usize;

                render_map[offset].tex[0] = x as f32 * texture_scale * terxel_size[0];
                render_map[offset].tex[1] = y as f32 * texture_scale * terxel_size[1];
            }
        }
    }

    /// Raven `CTRLandScape::SetShaders`.
    ///
    /// Source: `oracle/codemp/renderer/tr_terrain.cpp:495-506`
    pub fn set_shaders(&mut self, height: i32, shader: qhandle_t) {
        let mut i = height;
        while shader != 0 && (i as usize) < HEIGHT_RESOLUTION {
            let idx = i as usize;
            if self.mHeightDetails[idx].GetShader() == 0 {
                self.mHeightDetails[idx].SetShader(shader);
            }
            i += 1;
        }
    }

    /// Raven `CTRLandScape::LoadTerrainDef` — wave 9 (`snake_case`, matching
    /// every other landed `CTRLandScape` method in this file). GP2-parses
    /// `ext_data/RMG/<td>.terrain` (falling back to `ext_data/arioche/<td>
    /// .terrain` on a first miss, printing `Could not open %s` and returning
    /// non-fatally on a double miss), then walks the root group's subgroups'
    /// subgroups for `altitudetexture`/`water`/`flattexture` blocks.
    ///
    /// `Com_sprintf`/`stricmp` -> `format!`/`eq_ignore_ascii_case` per the
    /// translation dictionary (mirrors this crate's `ParseSkyParms`
    /// treatment, `tr_shader.rs`, of the same raw-pointer-`Com_sprintf`
    /// substitution). `atol` -> `native_string::atoi::atoi` (identical on
    /// the 32-bit retail target — same one-line ruling `cm_terrain.rs`'s
    /// `CmLandScape::load_terrain_def` states for its own `atol` call).
    ///
    /// `#ifndef PRE_RELEASE_DEMO` wraps the whole retail body; the demo-only
    /// empty-fn arm is dead surface, dropped (§C10).
    ///
    /// The `water`/`flattexture` cases' `RE_RegisterShader` calls thread
    /// through the whole shader-registration surface (`QSharedScratch`,
    /// `FrameState`, `RenderAssets`, `EngineHostView`, `RendererCvars`,
    /// `RenderModels`, `TrImageState`, `viewParms_t`,
    /// `SkyState`) per its own wave-8 LAW signature. That
    /// callee is itself still a whole-fn loud `todo!()` stub pending its
    /// `lightmaps2d`/`stylesDefault` gap (`tr_shader.rs`); any non-empty
    /// `shader` name reaching this fn panics through it until that wave
    /// lands (marker law: "Panics via `<callee>`'s loud stub").
    ///
    /// `mFlatShader` (a plain `qhandle_t`) takes `RE_RegisterShader`'s
    /// return directly. `mWaterShader` (`*mut shader_t`, tier-2 raw
    /// pointer — `renderer-r2-design.md`'s tier-2 transition audit lists it
    /// "Coordinate (soft)" alongside `mTLShader`/`mBRShader`) cannot receive
    /// `R_GetShaderByHandle`'s idiomatic `ShaderHandle` return without
    /// unsafe pointer synthesis, banned by the interior-safety law; the
    /// registration + lookup calls are still issued for their real
    /// side effects (mirrors this crate's `ParseSkyParms` "calls issued,
    /// handle discarded" treatment), but the store into `mWaterShader`
    /// itself is DEFERRED to that coordinated slice.
    ///
    /// Source: `oracle/codemp/renderer/tr_terrain.cpp:508-578`
    #[allow(clippy::too_many_arguments)]
    pub fn load_terrain_def(
        &mut self,
        td: &str,
        view: &mut EngineHostView,
        qs: &mut QSharedScratch,
        frame: &mut FrameState,
        assets: &mut RenderAssets,
        cvars: &RendererCvars,
        models: &RenderModels,
        img_state: &mut TrImageState,
        sky_view: &mut viewParms_t,
        sky: &mut SkyState,
    ) {
        let mut path = format!("ext_data/RMG/{td}.terrain");
        com_printf(
            view.common,
            &format!("R_Terrain: Loading and parsing terrainDef {td}.....\n"),
        );

        self.mWaterShader = core::ptr::null_mut();
        self.mFlatShader = 0;

        let mut parse = GenericParser2::new();
        if !Com_ParseTextFile(view, &path, &mut parse, true) {
            path = format!("ext_data/arioche/{td}.terrain");
            if !Com_ParseTextFile(view, &path, &mut parse, true) {
                com_printf(view.common, &format!("Could not open {path}\n"));
                return;
            }
        }

        // The whole file → the root `{ }` struct → its subgroups.
        let basegroup = parse.top_level();
        for classes in basegroup.subgroups() {
            for items in classes.subgroups() {
                let type_name = items.name();
                if type_name.eq_ignore_ascii_case("altitudetexture") {
                    // Height must exist - the rest are optional
                    let height = atoi(items.find_pair_value("height").unwrap_or("0"));

                    // Shader for this height
                    let shader_name = items.find_pair_value("shader").unwrap_or("");
                    if !shader_name.is_empty() {
                        let shader = RE_RegisterShader(
                            shader_name,
                            qs,
                            frame,
                            assets,
                            view,
                            cvars,
                            models,
                            img_state,
                            sky_view,
                            sky,
                        );
                        if shader != 0 {
                            self.set_shaders(height, shader);
                        }
                    }
                } else if type_name.eq_ignore_ascii_case("water") {
                    let shader_name = items.find_pair_value("shader").unwrap_or("");
                    let shader = RE_RegisterShader(
                        shader_name,
                        qs,
                        frame,
                        assets,
                        view,
                        cvars,
                        models,
                        img_state,
                        sky_view,
                        sky,
                    );
                    // DEFERRED: mWaterShader store — raw-pointer tier-2
                    // field, no Handle<Shader> replacement landed yet
                    // (renderer-r2-design.md tier-2 transition audit, Group
                    // 2 `shader_t` row, "Coordinate (soft)").
                    // Source: oracle/codemp/renderer/tr_terrain.cpp:564
                    let _ = R_GetShaderByHandle(assets, view.common, shader);
                } else if type_name.eq_ignore_ascii_case("flattexture") {
                    let shader_name = items.find_pair_value("shader").unwrap_or("");
                    self.mFlatShader = RE_RegisterShader(
                        shader_name,
                        qs,
                        frame,
                        assets,
                        view,
                        cvars,
                        models,
                        img_state,
                        sky_view,
                        sky,
                    );
                }
            }
        }

        Com_ParseTextFileDestroy(&mut parse);
    }

    /// Raven `CTRLandScape::GetBlendedShader` — wave 11. Body doesn't touch
    /// `self`; kept as an impl method (not a free fn) to mirror `CTRLandScape
    /// ::` class membership, matching this file's `has_water`/`set_visibility`
    /// precedent for member fns whose bodies happen not to read `self`.
    ///
    /// `R_CreateBlendedShader`'s already-ported wave-10 signature returns a
    /// `ShaderHandle` where the oracle returns `qhandle_t` directly; the
    /// round-trip is `.index() as i32`, the established seam convention for
    /// this exact conversion (`tr_scene.rs:483`'s `R_GetShaderByHandle` call,
    /// `tr_shader.rs:5073`'s `RE_RegisterShaderLightMap` return arm).
    /// `assets`/`common`/`cvars` are threaded (§B4) to satisfy that callee's
    /// own signature.
    ///
    /// Panics via `R_MergeShaders`'s loud stub (`tr_shader.rs:5354`) until
    /// its owning wave lands — reached through `R_CreateBlendedShader`, whose
    /// own doc comment names the same transitive path, whenever no existing
    /// blended shader is found.
    ///
    /// Source: `oracle/codemp/renderer/tr_terrain.cpp:580-592`
    pub fn get_blended_shader(
        &self,
        a: qhandle_t,
        b: qhandle_t,
        c: qhandle_t,
        surface_sprites: bool,
        assets: &mut RenderAssets,
        common: &mut Common,
        cvars: &RendererCvars,
    ) -> qhandle_t {
        // Special case single pass shader
        if a == b && a == c {
            return a;
        }

        let blended = R_CreateBlendedShader(a, b, c, surface_sprites, assets, common, cvars);
        blended.index() as i32
    }

    /// Raven `CTRLandScape::CalculateShaders`.
    // DEFERRED: `GetBlendedShader` itself landed this wave
    // (`get_blended_shader` above), but `CalculateShaders`'s own body is
    // still blocked on independent gaps: the whole retail-compiled body
    // (`#ifndef PRE_RELEASE_DEMO`) walks `mTRPatches` to fill
    // `mSortedPatches`/`mSortedCount`, then closes with `qsort(…,
    // ComparePatchInfo)` — neither the sorted-patch buffer's fill loop nor a
    // `qsort` call site exist yet in this file, out of this single-fn wave's
    // scope. Lands with a future `tr_terrain` slice.
    // Source: `oracle/codemp/renderer/tr_terrain.cpp:628-804`
    pub fn calculate_shaders(&mut self) {
        todo!("Port CTRLandScape::CalculateShaders — oracle/codemp/renderer/tr_terrain.cpp:628-804")
    }

    /// Raven `CTRLandScape::~CTRLandScape`.
    ///
    /// Rust has no destructor-with-context: Raven's `Z_Free` needs `&mut
    /// Common`, unavailable to `Drop::drop(&mut self)`. Ported as an
    /// explicit cleanup method the owning subsystem calls before the
    /// landscape is dropped (porting-rules §C10: control flow preserved,
    /// not shape).
    ///
    /// Source: `oracle/codemp/renderer/tr_terrain.cpp:854-871`
    pub fn destroy(&mut self, common: &mut Common) {
        if !self.mTRPatches.is_null() {
            Z_Free(common, self.mTRPatches as *mut ());
            self.mTRPatches = core::ptr::null_mut();
        }
        if !self.mSortedPatches.is_null() {
            Z_Free(common, self.mSortedPatches as *mut ());
            self.mSortedPatches = core::ptr::null_mut();
        }
        if !self.mRenderMap.is_null() {
            Z_Free(common, self.mRenderMap as *mut ());
            self.mRenderMap = core::ptr::null_mut();
        }
    }
}

/// Raven `ComparePatchInfo` — the `qsort` comparator over `mSortedPatches`.
///
/// Raven reads each half's shader through `arg->mPatch` (a raw `CTRPatch *`);
/// the two patches are resolved by the caller and passed in (§B5), which keeps
/// this file free of raw-pointer derefs. Its only call site,
/// [`CTRLandScape::calculate_shaders`], is deferred.
///
/// Source: `oracle/codemp/renderer/tr_terrain.cpp:594-626`
pub fn ComparePatchInfo(
    arg1: &TPatchInfo,
    patch1: &CTRPatch,
    arg2: &TPatchInfo,
    patch2: &CTRPatch,
) -> c_int {
    let s1 = if (arg1.mPart & PI_TOP) != 0 {
        patch1.mTLShader
    } else {
        patch1.mBRShader
    };
    let s2 = if (arg2.mPart & PI_TOP) != 0 {
        patch2.mTLShader
    } else {
        patch2.mBRShader
    };

    if s1 < s2 {
        -1
    } else if s1 > s2 {
        1
    } else {
        0
    }
}

/// Raven `InitRendererPatches` — the `CM_TerrainPatchIterate` callback that
/// links one collision patch to its renderer patch. Raven's `void *userdata`
/// landscape pair is threaded explicitly (§B4), and with it the three
/// back-pointer writes (`SetCommon`/`SetOwner`/`SetLocalOwner`) that threading
/// replaces.
///
/// Source: `oracle/codemp/renderer/tr_terrain.cpp:811-837`
pub fn InitRendererPatches(patch: &CmPatch, land_scape: &mut CTRLandScape, land: &CmLandScape) {
    // Get TRPatch pointer
    let tx = patch.hx;
    let ty = patch.hy;
    let bx = tx / land.terxels();
    let by = ty / land.terxels();

    let real_width = land.real_width();
    let block_width = land.block_width();

    land_scape.patch_mut(bx, by, block_width).clear();

    // `CTRPatch::SetRenderMap` lives on `CTRLandScape` (`ctrland_scape.rs`),
    // where the raw `mRenderMap` walk it performs is quarantined (§D11).
    land_scape.set_patch_render_map(bx, by, block_width, tx, ty, real_width);
    land_scape.patch_mut(bx, by, block_width).set_center(patch);
}

/// Raven `CTRLandScape::CopyHeightMap` — copies the byte heightmap into the
/// render map to speed up calcs.
///
/// Source: `oracle/codemp/renderer/tr_terrain.cpp:839-852`
pub fn CopyHeightMap(land_scape: &mut CTRLandScape, land: &CmLandScape) {
    let real_area = land.real_area() as usize;
    let height_map = land.height_map();
    let render_map = land_scape.render_map_mut(real_area);

    for i in 0..real_area {
        render_map[i].height = height_map[i] as c_int;
    }
}

/// Raven `RB_SurfaceTerrain`.
// DEFERRED: `TerrainFog = tr.world->globalFog` — `WorldAsset` has no
// `globalFog`-equivalent field yet (`tr_bsp`/`tr_world` R3 wave) — and the
// zero-arg `CTRLandScape::Render()` overload funnels entirely into `tess`,
// DISSOLVED into R4 (see this file's `Render` note above).
// Source: `oracle/codemp/renderer/tr_terrain.cpp:942-959`
pub fn RB_SurfaceTerrain(_surf: &mut srfTerrain_t, _terrain: &mut TerrainState) {
    todo!("Port RB_SurfaceTerrain — oracle/codemp/renderer/tr_terrain.cpp:942-959")
}

/// Raven `R_CalcTerrainVisBounds` — sets up the visbounds using terrain data.
///
/// Raven takes the `CTRLandScape` and immediately narrows to
/// `landscape->GetCommon()`; the `CCMLandScape` is threaded in directly (§B4),
/// so the renderer landscape is not a parameter. Raven's six per-axis `if`s
/// become the equivalent axis loop (§C10).
///
/// Source: `oracle/codemp/renderer/tr_terrain.cpp:961-991`
pub fn R_CalcTerrainVisBounds(land: &CmLandScape, view: &mut viewParms_t) {
    let mins = land.mins();
    let maxs = land.maxs();

    for i in 0..3 {
        if mins[i] < view.visBounds[0][i] {
            view.visBounds[0][i] = mins[i];
        }
        if maxs[i] > view.visBounds[1][i] {
            view.visBounds[1][i] = maxs[i];
        }
    }
}

/// Raven `RE_InitRendererTerrain`.
// DEFERRED: `CTRLandScape::CTRLandScape(const char *)` — wave-10 ported
// as [`CTRLandScape::new`] above, itself a whole-fn `todo!()` (see that
// method's own doc for the five verified blockers: the `CM_RegisterTerrain`
// shape/threading mismatch, the missing `GetSize` accessor, the dropped
// `CM_TerrainPatchIterate`, `CalculateShaders`'s existing block, and the
// unsettled `CTRLandScape` heap-ownership strategy). `R_TerrainShutdown`
// below is blocked on that same allocation strategy.
// Source: `oracle/codemp/renderer/tr_terrain.cpp:1010-1024`
pub fn RE_InitRendererTerrain(common: &mut Common, info: &str) {
    if info.is_empty() {
        com_printf(common, "RE_RegisterTerrain: NULL name\n");
        return;
    }

    com_printf(common, "R_Terrain: Creating RENDERER data.....\n");

    // Create and register a new landscape structure
    let _landscape = CTRLandScape::new(info);
}

/// Raven `R_TerrainInit`.
///
/// Source: `oracle/codemp/renderer/tr_terrain.cpp:1026-1038`
pub fn R_TerrainInit(
    view: &mut EngineHostView,
    cvars: &mut RendererCvars,
    assets: &mut RenderAssets,
    land_scape: &mut srfTerrain_t,
) {
    land_scape.surfaceType = surfaceType_t::SF_TERRAIN;
    land_scape.landscape = core::ptr::null_mut();

    cvars.r_terrainTessellate = Some(Cvar_Get(view, "r_terrainTessellate", "3", CVAR_CHEAT));
    cvars.r_drawTerrain = Some(Cvar_Get(view, "r_drawTerrain", "1", CVAR_CHEAT));
    cvars.r_showFrameVariance = Some(Cvar_Get(view, "r_showFrameVariance", "0", 0));
    cvars.r_terrainWaterOffset = Some(Cvar_Get(view, "r_terrainWaterOffset", "0", 0));

    assets.distance_cull = 6000.0;
    assets.distance_cull_squared = assets.distance_cull * assets.distance_cull;
}

/// Raven `R_TerrainShutdown`.
///
/// Source: `oracle/codemp/renderer/tr_terrain.cpp:1042-1054`
pub fn R_TerrainShutdown(cm: &mut CollisionWorld, land_scape: &mut srfTerrain_t) {
    // Raven passes the literal `0`, not the landscape it just fetched
    // (`tr_terrain.cpp:1049`).
    let terrain_id: thandle_t = 0;
    if !land_scape.landscape.is_null() {
        CM_ShutdownTerrain(cm, terrain_id);
        // DEFERRED: `delete ls` + `tr.landScape.landscape = NULL` —
        // `CTRLandScape`'s owning-allocation strategy is unsettled while its
        // ctor (`RE_InitRendererTerrain` above) is deferred, so there is no
        // established pointer provenance to reclaim here (`destroy`, this
        // file's ported `~CTRLandScape` body, frees the landscape's *internal*
        // buffers but not the `CTRLandScape` allocation itself).
        // Source: `oracle/codemp/renderer/tr_terrain.cpp:1046-1053`
        todo!("Port R_TerrainShutdown teardown — oracle/codemp/renderer/tr_terrain.cpp:1046-1053")
    }
}

// ---------------------------------------------------------------------
// wave 1
// ---------------------------------------------------------------------

/// Raven `R_AddTerrainSurfaces`.
///
/// `r_drawTerrain`/`tr.refdef` are threaded (`cvars`/`engine`/`refdef`, §B4).
/// `tr.landScape` (the `srfTerrain_t`) and `landscape->GetCommon()`'s
/// `CmLandScape` are threaded separately per this file's own header PORT-NOTE
/// ("the renderer reaches [`common`] through ... back-pointers ... the
/// carriers are threaded in as `&CmLandScape` ... parameters") — matching
/// [`R_CalcTerrainVisBounds`]'s own established shape below, which this fn
/// calls.
///
/// `shader_sorted_index` (`landscape->GetShader()->sortedIndex`) is read
/// through the tier-2 accessors `srfTerrain_t::landscape` and
/// `CTRLandScape::shader_sorted_index`, which quarantine the two raw-pointer
/// derefs in the owning types' own files (§D11) alongside this file's
/// `mTRPatches`/`mRenderMap` accessors.
///
/// `shifted_entity_num` is `tr.shiftedEntityNum`: unlike `R_AddPolygonSurfaces`
/// (`tr_scene.rs`), which assigns it inline in its own oracle body, this fn's
/// oracle body never writes it — it is read as ambient state set by whichever
/// caller precedes this call (`R_AddWorldSurfaces`, not in this wave's
/// packet). Threaded explicitly rather than guessed at a fixed value (no
/// speculative behavior, porting-rules §A2); the caller supplies the current
/// `tr.shiftedEntityNum`.
///
/// The surface payload has no dedicated `SurfaceGeometry` variant yet (world/
/// terrain surface arenas land with `tr_bsp`/`tr_world`, tier-2 transition
/// audit) — `SurfaceGeometry::Other` is the file's own established catch-all
/// for not-yet-modeled surface kinds (`tr_main.rs`'s `R_CullPointAndRadius`
/// family default-plane arm).
///
/// Source: `oracle/codemp/renderer/tr_terrain.cpp:993-1008`
pub fn R_AddTerrainSurfaces<'a>(
    cvars: RenderCvarSnapshot,
    refdef: &trRefdef_t,
    land_scape: &srfTerrain_t,
    land: &CmLandScape,
    view: &mut viewParms_t,
    shifted_entity_num: i32,
    draw_surfs: &mut Vec<DrawSurf<SurfaceGeometry<'a>>>,
) {
    if cvars.draw_terrain == 0 || (refdef.rdflags & RDF_NOWORLDMODEL) != 0 {
        return;
    }

    if land_scape.landscape.is_null() {
        return;
    }
    let shader_sorted_index = land_scape.landscape().shader_sorted_index();
    let rdf_nofog = (refdef.rdflags & RDF_NOFOG) != 0;

    R_AddDrawSurf(
        SurfaceGeometry::Other,
        shader_sorted_index,
        shifted_entity_num,
        rdf_nofog,
        0,
        0,
        draw_surfs,
    );
    R_CalcTerrainVisBounds(land, view);
}
