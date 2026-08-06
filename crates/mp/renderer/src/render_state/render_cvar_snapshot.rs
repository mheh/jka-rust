//! `RenderCvarSnapshot` — the render cvar values one frame reads (DEC-37
//! A13.1, R4 render-thread snapshot).

use mp_engine_qcommon::common::common::Common;

use crate::render_state::renderer_cvars::RendererCvars;

/// The resolved render cvar values that one executor frame reads.
///
/// `RendererCvars` holds the live `cvar_t*` handles. A13.1 deferred the
/// render-thread snapshot the backend needs. This struct is that snapshot: the
/// executor caller resolves each handle against the live engine cvar table
/// once, then passes this small copy by value into `FrameExecutor::
/// execute_frame` and `Pipeline3d::draw`. A caller without a live cvar table
/// (a harness or a golden test) uses [`RenderCvarSnapshot::default`], which
/// carries the retail cvar defaults.
///
/// W2-F1 reverses A13.1 for the world set. The world walk used to read the
/// live table through `EngineHostView::common`, which pinned the walk to the
/// sim thread. Every cvar the walk reads is now a field here, filled at
/// `RE_EndFrame`, so the walk runs render-side against a frozen copy.
///
/// The fields keep Raven's `->integer`/`->value` widths, so a gate compares the
/// same way the oracle does.
///
/// Source: `oracle/codemp/renderer/tr_cmds.cpp:105-109` (`r_skipBackEnd`),
/// `oracle/codemp/renderer/tr_backend.cpp:523-529` (`r_fastsky`),
/// `oracle/codemp/renderer/tr_surface.cpp:1555-1563` (`r_lodCurveError`),
/// `oracle/codemp/renderer/tr_init.cpp:746-874` (the registrations the
/// defaults below copy)
#[derive(Clone, Copy)]
pub struct RenderCvarSnapshot {
    /// `r_skipBackEnd->integer`. A non-zero value skips the whole backend
    /// replay, so the executor draws nothing this frame. Retail default `0`.
    pub skip_back_end: i32,
    /// `r_fastsky->integer`. A non-zero value skips the sky draw and clears the
    /// world to black. Retail default `0`.
    pub fastsky: i32,
    /// `r_lodCurveError->value`. The grid LOD arm divides the view-space error
    /// by this to pick the row and column step. Retail default `250`.
    pub lod_curve_error: f32,
    /// The shader backend the world pass draws with (DEC-37 ruling 5). A zero
    /// value picks the faithful backend, the parity reference. A non-zero value
    /// picks the PBR uber-shader. Retail default `0`. The harness rides an F9
    /// toggle and a `--pbr` boot flag on this field.
    pub pbr: i32,

    // ---- the world walk set (W2-F1) ------------------------------------
    /// `r_drawworld->integer`. Zero drops the whole BSP walk. Default `1`.
    pub drawworld: i32,
    /// `r_novis->integer`. Non-zero marks every non-solid node visible instead
    /// of reading the PVS. Default `0`.
    pub novis: i32,
    /// `r_lockpvs->integer`. Non-zero freezes the marked leaf set. Default `0`.
    pub lockpvs: i32,
    /// `r_nocull->integer`. `1` drops frustum culling, `2` drops the dlight
    /// side test as well. Default `0`.
    pub nocull: i32,
    /// `r_nocurves->integer`. Non-zero culls every bezier patch. Default `0`.
    pub nocurves: i32,
    /// `r_znear->value`. The near clip plane. Default `4`.
    pub znear: f32,
    /// `r_noportals->integer`. Non-zero refuses every mirror and portal view.
    /// Default `0`.
    pub noportals: i32,
    /// `r_facePlaneCull->integer`. Zero keeps back-facing planar surfaces.
    /// Default `1`.
    pub face_plane_cull: i32,
    /// `r_cullRoofFaces->integer`. The automap-shot roof cull. Default `0`.
    /// The render side runs the feature inert (W2-F2).
    pub cull_roof_faces: i32,
    /// `r_roofCullCeilDist->value`. Default `256`.
    pub roof_cull_ceil_dist: f32,
    /// `r_drawentities->integer`. Zero drops the whole entity walk. Default
    /// `1`.
    pub drawentities: i32,
    /// `r_drawTerrain->integer`. Default `1`. The landscape stays null in this
    /// wave, so the terrain walk returns early either way (W2-F6).
    pub draw_terrain: i32,
    /// `r_debugSurface->integer`. Picks the `R_DebugGraphics` overlay. Default
    /// `0`.
    pub debug_surface: i32,
    /// `r_portalOnly->integer`. Non-zero draws the mirrored view alone.
    /// Default `0`.
    pub portal_only: i32,
    /// `cg_shadows->integer`, which the renderer registers as `r_shadows`.
    /// Default `1`.
    pub shadows: i32,
    /// `r_lodbias->integer`. Shifts every model LOD choice. Default `0`.
    pub lodbias: i32,
    /// `r_lodscale->value`. Default `5`.
    pub lodscale: f32,
    /// `r_autolodscalevalue->value`. Default `0`.
    pub autolodscalevalue: f32,
    /// `r_ambientScale->value`. Scales the grid ambient term. Default `0.6`.
    pub ambient_scale: f32,
    /// `r_directedScale->value`. Scales the grid directed term. Default `1`.
    pub directed_scale: f32,
    /// `r_debuglight->integer`, registered as `r_debugLight`. Non-zero prints
    /// one line per first-person entity light. Default `0`.
    pub debug_light: i32,
    /// `r_fullbright->integer`. Non-zero flattens entity lighting. Default `0`.
    pub fullbright: i32,
    /// `r_flares->integer`. Non-zero keeps flare surfaces. Default `1`.
    pub flares: i32,
    /// `r_noserverghoul2->integer`, registered as `r_noServerGhoul2`. Default
    /// `0`.
    pub no_server_ghoul2: i32,
    /// `r_drawSun->integer`. Default `0`.
    pub draw_sun: i32,
    /// `r_dlightStyle->integer` - default `"1"`.
    /// A style above zero runs the `ProjectDlightTexture2` pass, and zero runs `ProjectDlightTexture`.
    ///
    /// Source: `oracle/codemp/renderer/tr_shade.cpp:2330-2336`
    pub dlight_style: i32,
    /// `r_swapInterval->integer` - default `"0"`.
    /// Zero asks for an unsynchronized present, and a nonzero value asks for vsync.
    ///
    /// Source: `oracle/codemp/renderer/tr_init.cpp:1068`
    pub swap_interval: i32,
}

impl RenderCvarSnapshot {
    /// Reads the live cvar table once, at the frame boundary.
    ///
    /// `pbr` has no cvar handle yet, so it keeps its retail default. The
    /// harness drives that field from its own F9 toggle instead.
    pub fn from_cvars(cvars: &RendererCvars, common: &Common) -> RenderCvarSnapshot {
        RenderCvarSnapshot {
            skip_back_end: common.cvar(cvars.r_skipBackEnd).integer,
            fastsky: common.cvar(cvars.r_fastsky).integer,
            lod_curve_error: common.cvar(cvars.r_lodCurveError).value,
            pbr: 0,

            drawworld: common.cvar(cvars.r_drawworld).integer,
            novis: common.cvar(cvars.r_novis).integer,
            lockpvs: common.cvar(cvars.r_lockpvs).integer,
            nocull: common.cvar(cvars.r_nocull).integer,
            nocurves: common.cvar(cvars.r_nocurves).integer,
            znear: common.cvar(cvars.r_znear).value,
            noportals: common.cvar(cvars.r_noportals).integer,
            face_plane_cull: common.cvar(cvars.r_facePlaneCull).integer,
            cull_roof_faces: common.cvar(cvars.r_cullRoofFaces).integer,
            roof_cull_ceil_dist: common.cvar(cvars.r_roofCullCeilDist).value,
            drawentities: common.cvar(cvars.r_drawentities).integer,
            draw_terrain: common.cvar(cvars.r_drawTerrain).integer,
            debug_surface: common.cvar(cvars.r_debugSurface).integer,
            portal_only: common.cvar(cvars.r_portalOnly).integer,
            shadows: common.cvar(cvars.r_shadows).integer,
            lodbias: common.cvar(cvars.r_lodbias).integer,
            lodscale: common.cvar(cvars.r_lodscale).value,
            autolodscalevalue: common.cvar(cvars.r_autolodscalevalue).value,
            ambient_scale: common.cvar(cvars.r_ambientScale).value,
            directed_scale: common.cvar(cvars.r_directedScale).value,
            debug_light: common.cvar(cvars.r_debugLight).integer,
            fullbright: common.cvar(cvars.r_fullbright).integer,
            flares: common.cvar(cvars.r_flares).integer,
            no_server_ghoul2: common.cvar(cvars.r_noServerGhoul2).integer,
            draw_sun: common.cvar(cvars.r_drawSun).integer,
            dlight_style: common.cvar(cvars.r_dlightStyle).integer,
            swap_interval: common.cvar(cvars.r_swapInterval).integer,
        }
    }
}

impl Default for RenderCvarSnapshot {
    /// The retail cvar defaults, for a caller without a live cvar table.
    fn default() -> RenderCvarSnapshot {
        RenderCvarSnapshot {
            skip_back_end: 0,
            fastsky: 0,
            lod_curve_error: 250.0,
            pbr: 0,

            drawworld: 1,
            novis: 0,
            lockpvs: 0,
            nocull: 0,
            nocurves: 0,
            znear: 4.0,
            noportals: 0,
            face_plane_cull: 1,
            cull_roof_faces: 0,
            roof_cull_ceil_dist: 256.0,
            drawentities: 1,
            draw_terrain: 1,
            debug_surface: 0,
            portal_only: 0,
            shadows: 1,
            lodbias: 0,
            lodscale: 5.0,
            autolodscalevalue: 0.0,
            ambient_scale: 0.6,
            directed_scale: 1.0,
            debug_light: 0,
            fullbright: 0,
            flares: 1,
            no_server_ghoul2: 0,
            draw_sun: 0,
            dlight_style: 1,
            swap_interval: 0,
        }
    }
}
