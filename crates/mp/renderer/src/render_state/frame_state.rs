//! `FrameState` — render-thread-local frame scratch (`R2-D1`).

use mp_engine_qcommon::qfiles::light_style_limits::MAX_LIGHT_STYLES;

use crate::render_state::placeholders::{
    BackEndCounters, OrientationR, RefEntity, TrRefdef, ViewParms,
};
use crate::tr_local::srf_terrain_s::srfTerrain_t;

/// Render-thread-local scratch, replacing `backEndState_t`'s role in full —
/// all 11 fields accounted for (B5) — plus the frontend scratch/counters
/// `trGlobals_t` used to hold. `backEndData_t`'s double buffer is NOT
/// reproduced here (`### A1 disposition table`, `R2-D8`).
///
/// W2-F3 split this struct: the sim-written half is `WorldLoadState` and what
/// stays is the walk-and-view half, which `FrameExecutor` owns. The `ViewState`
/// rename that name change implies is cosmetic, and the user parked it.
///
/// Source: `oracle/codemp/renderer/tr_local.h:1279-1292`
pub struct FrameState {
    pub refdef: TrRefdef,
    pub view: ViewParms,
    pub ori: OrientationR,
    pub counters: BackEndCounters,
    pub is_hyperspace: bool,
    /// `trRefEntity_t *currentEntity` — whichever entity the backend is
    /// currently drawing, by value rather than by pointer (ruling 1: the
    /// renderer interior is oracle-match-free).
    pub current_entity: Option<RefEntity>,
    pub sky_rendered_this_view: bool,
    pub projection_2d: bool,
    pub color_2d: [u8; 4],
    pub vertexes_2d: bool,
    /// `trRefEntity_t entity2D` — a value field in the oracle
    /// (`currentEntity` points here during 2D rendering).
    pub entity_2d: RefEntity,
    /// The A11 snapshot carrier's consumer-side landing field: filled from
    /// `FrameEvent::RenderScene.light_styles` when the render thread processes
    /// that event, then read by the R4 tessellation/vertex-building consumers
    /// for the rest of that scene's surfaces
    /// (`oracle/codemp/renderer/tr_surface.cpp:279,324`,
    /// `oracle/codemp/renderer/tr_shade.cpp:1401,1685`,
    /// `oracle/codemp/renderer/tr_light.cpp:234-274`). A per-frame copy, not
    /// sim-owned `LightStyleTable` itself (`R2-D5`).
    pub scene_light_styles: [[u8; 4]; MAX_LIGHT_STYLES],
    // W2-F3 moved `tr.frameCount`, `tr.identityLight`,
    // `tr.identityLightByte`, `tr.overbrightBits`, `tr.sunDirection` and
    // `tr.sunAmbient` to `WorldLoadState`, and `tr.externalVisData` to
    // `RenderAssets`. The sim writes all seven, and this struct keeps only
    // what the render thread owns.
    // W2-F4 moved `tr.viewCount` and `tr.visCount` to
    // `WorldWalkScratch::view_count`/`vis_count`, beside the mark arrays they
    // stamp. `R_MarkFragments` keeps its own counter on `MarkState`, so the
    // decal path no longer shares the world walk's generation.
    // Source: `oracle/codemp/renderer/tr_local.h:1315-1316`
    /// `tr.sceneCount` — Raven: incremented every scene. The light-flare code
    /// distinguishes per-scene surface visibility by this count.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1314`
    pub scene_count: i32,
    /// `tr.frameSceneNum` — Raven: zeroed at `RE_BeginFrame`, bumped per
    /// scene. `R_RenderView` stamps the view's `frameSceneNum` from it.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1318`
    pub frame_scene_num: i32,
    /// `tr.viewCluster` — the PVS cluster the current view origin sits in,
    /// reset to `-1` by `RE_BeginRegistration` and by `R_MarkLeaves`'s
    /// novis path.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1382`
    pub view_cluster: i32,
    /// `skyboxportal` — cross-TU scene/backend state, DEC-37 A13.3 home:
    /// written sim-side by `RE_RenderScene`/`RE_LoadWorldMap`, read
    /// render-side by `tr_sky`/`tr_shade`/`tr_backend`. Raven's `int`.
    ///
    /// Source: `oracle/codemp/renderer/tr_scene.cpp:35`
    pub skyboxportal: i32,
    /// `drawskyboxportal` — cross-TU scene/backend state, DEC-37 A13.3 home;
    /// same write/read split as `skyboxportal`. Raven's `int`.
    ///
    /// Source: `oracle/codemp/renderer/tr_scene.cpp:36`
    pub drawskyboxportal: i32,
    /// `g_bRenderGlowingObjects` — cross-TU scene/backend state, DEC-37
    /// A13.3 home: Raven: "Whether we are currently rendering only glowing
    /// objects or not." Written by `RB_DrawSurfs`'s dynamic-glow pass, read
    /// by `tr_sky`/`tr_shade`/`tr_backend`.
    ///
    /// Source: `oracle/codemp/renderer/tr_backend.cpp:32`
    pub render_glowing_objects: bool,
    /// `tr.landScape` on the sim instance, seeded by `R_TerrainInit`.
    /// The executor keeps its own render-side seed (W2-F6), so this copy never crosses threads.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1422`, `tr_terrain.cpp:1028-1029`
    pub land_scape: srfTerrain_t,
}
