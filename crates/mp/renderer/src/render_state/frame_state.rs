//! `FrameState` — render-thread-local frame scratch (`R2-D1`).

use mp_engine_qcommon::qfiles::light_style_limits::MAX_LIGHT_STYLES;

use crate::render_state::placeholders::{
    BackEndCounters, OrientationR, RefEntity, TrRefdef, Vec3, ViewParms,
};

/// Render-thread-local scratch, replacing `backEndState_t`'s role in full —
/// all 11 fields accounted for (B5) — plus the frontend scratch/counters
/// `trGlobals_t` used to hold. `backEndData_t`'s double buffer is NOT
/// reproduced here (`### A1 disposition table`, `R2-D8`).
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
    /// `tr.frameCount` — Raven: incremented every frame.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1313`
    pub frame_count: i32,
    /// `tr.viewCount` — the frontend scratch counter `R_BoxSurfaces_r` stamps
    /// surfaces with to avoid revisiting them (`## State ownership`'s "tr
    /// frontend scratch/counters" row).
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1315`
    pub view_count: i32,
    /// `tr.visCount` — Raven: "incremented every time a new vis cluster is
    /// entered". `R_MarkLeaves` bumps it and stamps every visible node's
    /// `visframe` with it; `R_RecursiveWorldNode` gates the walk on the
    /// match. Distinct from `view_count` (`tr.viewCount`, per-surface).
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1316`
    pub vis_count: i32,
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
    /// `tr.identityLight` — `1.0 / ( 1 << overbrightBits )`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1374`
    pub identity_light: f32,
    /// `tr.identityLightByte` — `identityLight * 255`, truncated to `int` by
    /// the oracle's assignment.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1375`
    pub identity_light_byte: i32,
    /// `tr.overbrightBits` — the lightmap/vertex-color shift
    /// `R_ColorShiftLightingBytes` applies at BSP load.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1376`
    pub overbright_bits: i32,
    /// `tr.sunDirection`.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1385`
    pub sun_direction: Vec3,
    /// `tr.sunAmbient` — Raven: "from the sky shader (only used for John's
    /// terrain system)" (wave-10 field merge; see `tr_bsp.rs`'s WAVE 10
    /// ADDITIONS note).
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1387`
    pub sun_ambient: Vec3,
    /// `tr.externalVisData` — Raven: "from `RE_SetWorldVisData`, shared with
    /// `CM_Load`". Owned here rather than aliasing the collision world's
    /// buffer (interior-safety law); `None` until `CM_LoadMap` hands one over.
    ///
    /// Source: `oracle/codemp/renderer/tr_local.h:1326`
    pub external_vis_data: Option<Vec<u8>>,
}
