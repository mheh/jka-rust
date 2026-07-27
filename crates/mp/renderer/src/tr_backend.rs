//! Raven `tr_backend.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_backend.cpp`

// Raven-named functions/types keep their original casing across this
// transcription, matching the rest of the renderer/engine crates.
#![allow(non_snake_case)]

use mp_engine_qcommon::common::com_error;
use mp_qshared::shared::error_parm::errorParm_t;

use crate::render_state::frame_state::FrameState;
use crate::render_state::gpu_resources::GpuResources;
use crate::render_state::image_asset::ImageHandle;
use crate::render_state::placeholders::Vec3;
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::render_state::shader_asset::ShaderHandle;
use crate::tr_image::{R_Images_GetNextIteration, R_Images_StartIteration};
use crate::tr_local::cull_type_t::cullType_t;

// `R_WorldCoordToScreenCoordFloat` threads `RenderAssets::glconfig`
// (`crate::render_state::placeholders::GlConfig`) and `FrameState::refdef`
// (`crate::render_state::placeholders::TrRefdef`) per the R2 `## State
// ownership` rows for `glConfig`/`tr` frontend scratch. Both are still
// skeleton stubs owned by other waves this wave may not touch (porting-rules
// process); the fields it reads — `GlConfig`: `vid_width`/`vid_height: i32`;
// `TrRefdef`: `view_axis: [Vec3; 3]`, `view_origin: Vec3`,
// `fov_x`/`fov_y: f32` — are `glconfig_t`/`trRefdef_t`'s licensed shapes and
// land on those structs with the field-merge step of this wave's
// integration (tr_bsp.rs precedent).

// Official OpenGL 1.0/1.1 registry enum values (not a Raven `#define` — these
// are the fixed spec constants `GL_TexEnv`'s switch compares `env` against).
const GL_MODULATE: u32 = 0x2100;
const GL_DECAL: u32 = 0x2101;
const GL_ADD: u32 = 0x0104;
const GL_REPLACE: u32 = 0x1E01;

// Official ARB/NV extension-registry enum values (not a Raven `#define`) —
// the two pixel-shader path selectors `BeginPixelShader`/`EndPixelShader`
// switch on.
const GL_REGISTER_COMBINERS_NV: u32 = 0x8523;
const GL_FRAGMENT_PROGRAM_ARB: u32 = 0x8804;

/// Per-subsystem render-thread state for the pixel-shader (register-combiner
/// / ARB fragment-program) path `BeginPixelShader`/`EndPixelShader` track
/// between calls.
///
/// Raven `GLuint g_uiCurrentPixelShaderType` — a file-scope static crossing
/// between the two fns; both consumers land in this file at this wave, so it
/// is named here per DEC-37 A13.3.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp` (`g_uiCurrentPixelShaderType`)
#[derive(Default)]
pub struct PixelShaderState {
    /// `0x0` (unset) in the oracle; `None` here.
    pub current_type: Option<u32>,
}

/// Raven `GL_Bind` — binds a 2D image as the active texture on the current
/// TMU: resolves the `NULL image` fallback to `tr.defaultImage`, the
/// `r_nobind` performance-evaluation override to `tr.dlightImage`, then
/// compares against `glState.currenttextures[glState.currenttmu]` before
/// issuing `qglBindTexture` and stamping `image->frameUsed = tr.frameCount`.
///
/// DEFERRED: R4 — every touched field lives on a placeholder still owned by
/// a later wave: `RenderAssets`' `default_image`/`dlight_image`/`frame_count`
/// registry state and `ImageAsset::texnum`/`frame_used` (tr_image wave), and
/// `GpuResources::gl_state`'s `currenttextures`/`currenttmu` cache (a named
/// placeholder until R4 defines the real pipeline/bind-group cache). The
/// bind decision and the `qglBindTexture` call are GL-only regardless
/// (DEC-37 A13.2).
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:61-82`
pub fn GL_Bind(_gpu: &mut GpuResources, _image: Option<ImageHandle>) {
    // DEFERRED: R4 — GL_Bind (see doc comment above) (DEC-37 A13.2)
    // Source: oracle/codemp/renderer/tr_backend.cpp:61-82
}

/// Raven `GL_Bind3D` — `GL_Bind`'s `GL_TEXTURE_3D` twin; identical texnum
/// resolution, same `glState.currenttextures[currenttmu]` cache compare.
///
/// DEFERRED: R4 — same dependency set as `GL_Bind` (DEC-37 A13.2).
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:85-107`
pub fn GL_Bind3D(_gpu: &mut GpuResources, _image: Option<ImageHandle>) {
    // DEFERRED: R4 — GL_Bind3D (see doc comment above) (DEC-37 A13.2)
    // Source: oracle/codemp/renderer/tr_backend.cpp:85-107
}

/// Raven `GL_SelectTexture` — selects a texture unit (TMU) for subsequent
/// texture state changes; `unit` must be `0..=3`.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:112-152`
pub fn GL_SelectTexture(_gpu: &mut GpuResources, unit: i32) {
    match unit {
        0..=3 => {
            // DEFERRED: R4 — GL_SelectTexture glState.currenttmu
            // cache-compare, qglActiveTextureARB/qglClientActiveTextureARB
            // per unit, and the GLimp_LogComment trace calls
            // (GpuResources::gl_state is a named placeholder until R4)
            // (DEC-37 A13.2)
            // Source: oracle/codemp/renderer/tr_backend.cpp:114-151
        }
        _ => com_error(
            errorParm_t::ERR_DROP,
            format!("GL_SelectTexture: unit = {unit}"),
        ),
    }
}

/// Raven `GL_Cull` — selects the fixed-function face-culling mode for the
/// current draw call; a no-op while `backEnd.projection2D` (2D drawing
/// always disables culling).
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:158-198`
pub fn GL_Cull(frame: &FrameState, _gpu: &mut GpuResources, cull_type: cullType_t) {
    // DEFERRED: R4 — GL_Cull glState.faceCulling cache-compare + write
    // (GpuResources::gl_state is a named placeholder until R4 defines the
    // real pipeline/bind-group cache) (DEC-37 A13.2)
    // Source: oracle/codemp/renderer/tr_backend.cpp:159-162

    if frame.projection_2d {
        //don't care, we're in 2d when it's always disabled
        return;
    }

    // DEFERRED: R4 — GL_Cull qglEnable/qglDisable(GL_CULL_FACE) and
    // qglCullFace(GL_FRONT/GL_BACK) selection (needs FrameState::view
    // .is_mirror, landed by the tr_main wave) (DEC-37 A13.2)
    // Source: oracle/codemp/renderer/tr_backend.cpp:167-197
    let _ = cull_type;
}

/// Raven `GL_TexEnv` — sets the fixed-function texture-environment mode for
/// the current TMU.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:203-236`
pub fn GL_TexEnv(_gpu: &mut GpuResources, env: u32) {
    match env {
        GL_MODULATE | GL_REPLACE | GL_DECAL | GL_ADD => {
            // DEFERRED: R4 — GL_TexEnv glState.texEnv[currenttmu]
            // cache-compare + qglTexEnvf(GL_TEXTURE_ENV, GL_TEXTURE_ENV_MODE,
            // env) (GpuResources::gl_state is a named placeholder until R4)
            // (DEC-37 A13.2)
            // Source: oracle/codemp/renderer/tr_backend.cpp:210-226
        }
        _ => com_error(
            errorParm_t::ERR_DROP,
            format!("GL_TexEnv: invalid env '{env}' passed\n"),
        ),
    }
}

/// Raven `GL_State` — diffs `stateBits` against the cached
/// `glState.glStateBits` and issues the fixed-function GL depth-func/
/// blend/depth-mask/polygon-mode/depth-test/alpha-test calls for whatever
/// changed.
///
/// DEFERRED: R4 — pure fixed-function GL state translation; every branch
/// both reads and writes `GpuResources::gl_state`, a named placeholder until
/// R4 defines the real pipeline/bind-group cache. The `GLS_*` bit-flag
/// `#define`s this decodes are not yet ported to Rust consts — left
/// undecoded rather than guessed at (DEC-37 A13.2).
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:244-431`
pub fn GL_State(_gpu: &mut GpuResources, _state_bits: u32) {
    // DEFERRED: R4 — GL_State (see doc comment above) (DEC-37 A13.2)
    // Source: oracle/codemp/renderer/tr_backend.cpp:244-431
}

/// Raven `RB_Hyperspace` — the hyperspace/warp screen-flash effect: a
/// flat-grey clear whose brightness cycles with `backEnd.refdef.time`.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:442-454`
pub fn RB_Hyperspace(frame: &mut FrameState) {
    if !frame.is_hyperspace {
        // do initialization shit
    }

    // DEFERRED: R4 — RB_Hyperspace c = (backEnd.refdef.time & 255) / 255.0;
    // qglClearColor(c, c, c, 1); qglClear(GL_COLOR_BUFFER_BIT) (needs
    // FrameState::refdef.time, landed by the tr_scene wave) (DEC-37 A13.2)
    // Source: oracle/codemp/renderer/tr_backend.cpp:449-451

    frame.is_hyperspace = true;
}

/// Raven `SetViewportAndScissor` — loads the view's projection matrix and
/// sets the GL viewport/scissor rect from `backEnd.viewParms`.
///
/// DEFERRED: R4 — pure GL (qglMatrixMode/qglLoadMatrixf/qglViewport/
/// qglScissor); every value it reads (`viewParms.projectionMatrix`/
/// `viewportX`/`viewportY`/`viewportWidth`/`viewportHeight`) lives on
/// `FrameState::view`, a placeholder landed by the tr_main wave (DEC-37
/// A13.2).
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:457-467`
pub fn SetViewportAndScissor(_frame: &FrameState) {
    // DEFERRED: R4 — SetViewportAndScissor (see doc comment above) (DEC-37 A13.2)
    // Source: oracle/codemp/renderer/tr_backend.cpp:457-467
}

/// Raven `R_WorldCoordToScreenCoordFloat` — projects a world-space point
/// through the current refdef's view axes and FOV onto screen coordinates;
/// `None` when the point is behind (or too close to) the view plane.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:598-635`
pub fn R_WorldCoordToScreenCoordFloat(
    assets: &RenderAssets,
    frame: &FrameState,
    world_coord: Vec3,
) -> Option<(f32, f32)> {
    let xcenter = (assets.glconfig.vid_width / 2) as f32;
    let ycenter = (assets.glconfig.vid_height / 2) as f32;

    //AngleVectors (tr.refdef.viewangles, vfwd, vright, vup);
    let vfwd = frame.refdef.view_axis[0];
    let vright = frame.refdef.view_axis[1];
    let vup = frame.refdef.view_axis[2];

    let local = [
        world_coord[0] - frame.refdef.view_origin[0],
        world_coord[1] - frame.refdef.view_origin[1],
        world_coord[2] - frame.refdef.view_origin[2],
    ];

    let transformed = [dot(local, vright), dot(local, vup), dot(local, vfwd)];

    // Make sure Z is not negative.
    if transformed[2] < 0.01 {
        return None;
    }

    // C promotes to double; f64 intermediate per wave-0 ruling 12 (the `90.0`
    // literals are doubles, so each `90.0/fov` quotient and its product are).
    let xzi = ((xcenter / transformed[2]) as f64 * (90.0 / frame.refdef.fov_x as f64)) as f32;
    let yzi = ((ycenter / transformed[2]) as f64 * (90.0 / frame.refdef.fov_y as f64)) as f32;

    let x = xcenter + xzi * transformed[0];
    let y = ycenter - yzi * transformed[1];

    Some((x, y))
}

/// `DotProduct` inlined (a `q_math.h` inline helper, not a module fn).
fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Raven `R_AverageTessXYZ` — averages the two nearest tessellated vertices
/// for register-combiner texgen support.
///
/// DEFERRED: R4 — `tess` dissolves into R4's tessellation/vertex-building
/// pipeline (R2 `## State ownership` row `tess`); no R3 carrier holds
/// `tess.xyz`/`tess.numVertexes` to read from.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:673-703`
pub fn R_AverageTessXYZ() -> Option<Vec3> {
    // DEFERRED: R4 — R_AverageTessXYZ (see doc comment above)
    // Source: oracle/codemp/renderer/tr_backend.cpp:673-703
    None
}

/// Raven `RB_SetColor` — the `RC_SET_COLOR` backend command: converts the
/// float `[0,1]` color to the byte `backEnd.color2D` the 2D draw commands
/// use. The oracle's `data`/`cmd + 1` command-buffer walk dissolves — the
/// caller supplies the already-decoded `FrameEvent::SetColor` payload
/// directly (`R2-D2`/A1).
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1404-1415`
pub fn RB_SetColor(frame: &mut FrameState, color: [f32; 4]) {
    frame.color_2d = [
        (color[0] * 255.0) as u8,
        (color[1] * 255.0) as u8,
        (color[2] * 255.0) as u8,
        (color[3] * 255.0) as u8,
    ];
}

/// Raven `RB_DrawBuffer` — selects the GL draw buffer, then clears the
/// screen: the world's global-fog color if the loaded BSP has one,
/// otherwise `r_clear`'s debug color cycle (or a random one at `r_clear 42`).
///
/// DEFERRED: R4 — GL-only (qglDrawBuffer/qglClear/qglClearColor) plus three
/// not-yet-landed dependencies: `RenderAssets::world`'s `WorldAsset` fog
/// fields (`global_fog`/`fogs`, tr_bsp/tr_world wave), the `r_clear` cvar's
/// live integer value (read via `Common::cvar(handle)` once this fn's carrier threads
/// `common` (see `tr_scene.rs`'s `r_markcount` precedent, A13.1), and
/// `Q_irand`'s receiver (R2 assigns the renderer none — digest note). The
/// oracle's `data`/`cmd + 1` command-buffer walk also dissolves per A1.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1710-1764`
pub fn RB_DrawBuffer(_assets: &RenderAssets, _cvars: &RendererCvars, _buffer: u32) {
    // DEFERRED: R4 — RB_DrawBuffer (see doc comment above) (DEC-37 A13.2)
    // Source: oracle/codemp/renderer/tr_backend.cpp:1710-1764
}

/// Raven `BeginPixelShader` — selects a register-combiner or ARB
/// fragment-program pixel shader.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1967-2000`
pub fn BeginPixelShader(pixel_shader: &mut PixelShaderState, ui_type: u32, _ui_id: u32) {
    match ui_type {
        // Using Register Combiners, so call the Display List that stores it.
        GL_REGISTER_COMBINERS_NV => {
            // PORT-NOTE: the oracle's `if (!qglCombinerParameterfvNV) return;`
            // extension-availability guard is folded into the deferred GL
            // call below — extension-pointer availability has no R3 home.
            // DEFERRED: R4 — BeginPixelShader qglEnable(GL_REGISTER_COMBINERS_NV)
            // + qglCallList(uiID) (DEC-37 A13.2)
            // Source: oracle/codemp/renderer/tr_backend.cpp:1975-1981
            pixel_shader.current_type = Some(GL_REGISTER_COMBINERS_NV);
        }
        // Using Fragment Programs, so call the program.
        GL_FRAGMENT_PROGRAM_ARB => {
            // PORT-NOTE: the oracle's `if (!qglGenProgramsARB) return;`
            // extension-availability guard is folded into the deferred GL
            // call below.
            // DEFERRED: R4 — BeginPixelShader qglEnable(GL_FRAGMENT_PROGRAM_ARB)
            // + qglBindProgramARB(GL_FRAGMENT_PROGRAM_ARB, uiID) (DEC-37 A13.2)
            // Source: oracle/codemp/renderer/tr_backend.cpp:1989-1996
            pixel_shader.current_type = Some(GL_FRAGMENT_PROGRAM_ARB);
        }
        _ => {}
    }
}

/// Raven `EndPixelShader` — disables whichever pixel-shader path
/// `BeginPixelShader` last selected.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:2003-2009`
pub fn EndPixelShader(pixel_shader: &PixelShaderState) {
    let Some(current_type) = pixel_shader.current_type else {
        return;
    };
    // DEFERRED: R4 — EndPixelShader qglDisable(current_type) (DEC-37 A13.2)
    // Source: oracle/codemp/renderer/tr_backend.cpp:2008
    let _ = current_type;
}

/// Raven `RB_BeginDrawingView` — clears the draw buffers for a new view and
/// sets up the portal clip plane.
///
/// DEFERRED: R4 — every dependency past the one field write below is
/// unhomed:
/// - this fn's carrier does not yet thread `common` for `Common::cvar` reads (`RendererCvars`
///   holds only `Option<CvarHandle>`, A13.1) — `r_finish`/`r_measureOverdraw`/
///   `r_shadows`/`r_fastsky`/`r_DynamicGlow`'s live values can't be read;
/// - `TrRefdef` (`FrameState::refdef`) only carries the `tr_backend` wave-0
///   fields (`fov_x`/`fov_y`/`view_origin`/`view_axis`) — `rdflags` lands
///   with the `tr_scene` wave (R2 `## State ownership`, `trRefdef_t` row),
///   so the `RDF_SKYBOXPORTAL`/`RDF_NOWORLDMODEL`/`RDF_AUTOMAP`/
///   `RDF_HYPERSPACE` bit tests this fn guards on are left undecoded rather
///   than guessed at (same treatment as `GL_State`'s `GLS_*` bits, DEC-37
///   A13.2);
/// - `ViewParms`/`OrientationR` (`FrameState::view`/`ori`) are still empty
///   stubs — `isPortal`/`portalPlane`/`ori.axis` land with the `tr_main`
///   wave;
/// - `tr.world`'s `globalFog`/`fogs` land with the `tr_bsp`/`tr_world` wave;
/// - `g_bRenderGlowingObjects`/`skyboxportal`/`tr_stencilled` are file-scope
///   statics with no R2-assigned carrier yet (DEC-37 A13.3 — a per-subsystem
///   state struct is named by whichever fn's wave actually reads/writes
///   them; none of that state is reached by the portable slice below);
/// - every `qgl*` call (`qglFinish`/`qglClearColor`/`qglClear`/
///   `qglLoadMatrixf`/`qglClipPlane`/`qglEnable`/`qglDisable`) is GL-only —
///   `GpuResources::gl_state` stays a named placeholder until R4 (DEC-37
///   A13.2).
///
/// Only the one unconditional field write lands here: `backEnd.projection2D
/// = qfalse;`. `SetViewportAndScissor`/`GL_State`/`RB_Hyperspace` (the
/// wave-0 in-module callees) are not invoked — every call site downstream of
/// them is gated by the unresolved state above.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:477-593`
pub fn RB_BeginDrawingView(
    frame: &mut FrameState,
    _gpu: &mut GpuResources,
    _assets: &RenderAssets,
    _cvars: &RendererCvars,
) {
    // we will need to change the projection matrix before drawing
    // 2D images again
    frame.projection_2d = false;

    // DEFERRED: R4 — the rest of RB_BeginDrawingView (see doc comment above)
    // (DEC-37 A13.1, A13.2, A13.3; R2 `## State ownership` trRefdef_t /
    // viewParms_t / orientationr_t rows)
    // Source: oracle/codemp/renderer/tr_backend.cpp:480-593
}

/// Raven `R_WorldCoordToScreenCoord` — the `int`-out-param wrapper around
/// `R_WorldCoordToScreenCoordFloat`; returns `None` where the oracle's
/// `bool retVal == false` left `*x`/`*y` holding whatever `xF`/`yF` happened
/// to be (an unwritten-on-failure read, porting-rules §19: the one defined
/// behavior kept here is "no coordinate" rather than a garbage cast).
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:638-645`
pub fn R_WorldCoordToScreenCoord(
    assets: &RenderAssets,
    frame: &FrameState,
    world_coord: Vec3,
) -> Option<(i32, i32)> {
    let (x_f, y_f) = R_WorldCoordToScreenCoordFloat(assets, frame, world_coord)?;
    Some((x_f as i32, y_f as i32))
}

/// Raven `RB_SetGL2D` — switches the backend into 2D orthographic drawing
/// mode: sets `backEnd.projection2D`, the 2D viewport/scissor/projection
/// matrix, blend state, and stamps `backEnd.refdef.time`/`floatTime` for 2D
/// shaders.
///
/// DEFERRED: R4 — every `qgl*` call (`qglViewport`/`qglScissor`/
/// `qglMatrixMode`/`qglLoadIdentity`/`qglOrtho`/`qglDisable`) and the
/// `GL_State` call (its `GLS_DEPTHTEST_DISABLE | GLS_SRCBLEND_SRC_ALPHA |
/// GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA` bits are left undecoded rather than
/// guessed at, same treatment as `GL_State`'s own body) are GL-only (DEC-37
/// A13.2). The `backEnd.refdef.time`/`floatTime` stamp also has no home yet
/// — `TrRefdef` (`FrameState::refdef`) only carries the `tr_backend` wave-0
/// fields (`fov_x`/`fov_y`/`view_origin`/`view_axis`); `time`/`floatTime`
/// land with the `tr_scene` wave (R2 `## State ownership`, `trRefdef_t`
/// row) — and reading it needs `Sys_Milliseconds`'s `Engine` param plus
/// `com_timescale`'s live cvar value, neither threaded to this fn.
///
/// Only the one unconditional field write lands here: `backEnd.projection2D
/// = qtrue;`.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1266-1292`
pub fn RB_SetGL2D(frame: &mut FrameState, _gpu: &mut GpuResources, _assets: &RenderAssets) {
    frame.projection_2d = true;

    // DEFERRED: R4 — the rest of RB_SetGL2D (see doc comment above) (DEC-37
    // A13.2; R2 `## State ownership` trRefdef_t row)
    // Source: oracle/codemp/renderer/tr_backend.cpp:1269-1291
}

/// Raven `RE_UploadCinematic` — (re)uploads a cinematic video frame into the
/// per-client scratch texture `tr.scratchImage[client]`, either as a fresh
/// `qglTexImage2D` when the frame size changed or a `qglTexSubImage2D` when
/// `dirty`.
///
/// DEFERRED: R4 — `tr.scratchImage[MAX_VIDEO_CLIENTS]` has no R2-assigned
/// carrier (not one of the named `## State ownership` `tr` sub-fields, and
/// not a `RenderAssets::images` registry entry — a fixed per-client scratch
/// slot, not a registered/named image); `GL_Bind`'s own body is itself
/// deferred pending that same registry wiring. Every `qgl*` call
/// (`qglTexImage2D`/`qglTexSubImage2D`/`qglTexParameterf`) is GL-only
/// (DEC-37 A13.2).
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1367-1395`
pub fn RE_UploadCinematic(
    _gpu: &mut GpuResources,
    _assets: &mut RenderAssets,
    _cols: i32,
    _rows: i32,
    _data: &[u8],
    _client: i32,
    _dirty: bool,
) {
    // DEFERRED: R4 — RE_UploadCinematic (see doc comment above) (DEC-37 A13.2)
    // Source: oracle/codemp/renderer/tr_backend.cpp:1367-1395
}

/// Raven `RB_BlurGlowTexture` — the dynamic-glow blur pass: N iterations
/// (`r_DynamicGlowPasses`) of a fullscreen-quad vertex/pixel-shader blur over
/// `tr.screenGlow`/`tr.blurImage`, widening the texel offset by
/// `r_DynamicGlowDelta` each pass.
///
/// DEFERRED: R4 — entirely GL/cvar-value/GPU-texture-handle driven: the
/// carrier does not yet thread `common` for cvar reads (`r_DynamicGlowIntensity`/
/// `r_DynamicGlowPasses`/`r_DynamicGlowDelta`'s live values, A13.1);
/// `tr.glowVShader`/`glowPShader`/`screenGlow`/`blurImage` are GL program/
/// texture handles with no R2-assigned carrier (GPU-facing state, an R4
/// concern per `GpuResources`'s own doc comment); `g_bTextureRectangleHack`
/// is homed outside this TU with no confirmed receiver; every `qgl*` call is
/// GL-only and the `glState.currenttmu` write is a named placeholder until
/// R4 (DEC-37 A13.2).
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:2015-2189`
pub fn RB_BlurGlowTexture(
    _frame: &FrameState,
    _gpu: &mut GpuResources,
    _assets: &RenderAssets,
    _cvars: &RendererCvars,
) {
    // DEFERRED: R4 — RB_BlurGlowTexture (see doc comment above) (DEC-37 A13.2)
    // Source: oracle/codemp/renderer/tr_backend.cpp:2015-2189
}

/// Raven `RB_DrawGlowOverlay` — composites the blurred glow texture
/// (`tr.blurImage`) additively over the scene texture (`tr.sceneImage`) in
/// 2D orthographic mode.
///
/// DEFERRED: R4 — entirely GL/cvar-value/GPU-texture-handle driven: the
/// carrier does not yet thread `common` for cvar reads (`r_DynamicGlow`/
/// `r_DynamicGlowHeight`/`r_DynamicGlowSoft`/`r_DynamicGlowWidth`'s live
/// values, A13.1); `tr.sceneImage`/`blurImage` are GL texture handles with
/// no R2-assigned carrier (GPU-facing state, an R4 concern); every `qgl*`
/// call is GL-only (DEC-37 A13.2).
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:2192-2325`
pub fn RB_DrawGlowOverlay(_gpu: &mut GpuResources, _assets: &RenderAssets, _cvars: &RendererCvars) {
    // DEFERRED: R4 — RB_DrawGlowOverlay (see doc comment above) (DEC-37 A13.2)
    // Source: oracle/codemp/renderer/tr_backend.cpp:2192-2325
}

/// Raven `RB_RotatePic` — the `RC_ROTATE_PIC` backend command: rotates a
/// stretched pic about its top-right corner (`x + w`, `y`) and draws it as a
/// `GL_QUADS` quad. The oracle's `data`/`cmd + 1` command-buffer walk
/// dissolves — the caller supplies the already-decoded
/// `FrameEvent::DrawRotatePic` payload directly (`R2-D2`/A1); `cmd->shader`
/// becomes the `shader: ShaderHandle` payload field per the tier-2
/// transition audit's `rotatePicCommand_t`/`shader_t` rows.
///
/// The oracle's `image = &shader->stages[0].bundle[0].image[0]` is a plain
/// re-fetch of `bundle[0].image` (indexing a pointer field with `[0]` is
/// `*image`, so `&image[0]` is `image` itself) — a real nullable pointer,
/// not a structurally-non-null address-of; landed here as
/// `ShaderStage::image` and the `if (image)` guard as an `Option` check. A
/// stale/invalid `shader` handle or a shader with no stages yet (`stages` is
/// still populated empty by every current `GeneratePermanentShader` call —
/// its per-stage copy loop is a separate, later wave) both fall through the
/// same `None` path as a genuinely unset image, matching the oracle's
/// "skip drawing" outcome (porting-rules §19).
///
/// DEFERRED: R4 — past that guard, every effect (`qglColor4ubv`/
/// `qglPushMatrix`/`qglTranslatef`/`qglRotatef`, `GL_Bind`'s own innards, the
/// `qglBegin(GL_QUADS)`/`qglTexCoord2f`/`qglVertex2f` quad, and
/// `qglEnd`/`qglPopMatrix`) is GL-only (DEC-37 A13.2).
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1498-1540`
pub fn RB_RotatePic(
    frame: &mut FrameState,
    gpu: &mut GpuResources,
    assets: &RenderAssets,
    shader: ShaderHandle,
    _x: f32,
    _y: f32,
    _w: f32,
    _h: f32,
    _a: f32,
    _s1: f32,
    _t1: f32,
    _s2: f32,
    _t2: f32,
) {
    let image = assets
        .shaders
        .get(shader)
        .and_then(|s| s.stages.first())
        .and_then(|stage| stage.image);

    if let Some(image) = image {
        if !frame.projection_2d {
            RB_SetGL2D(frame, gpu, assets);
        }

        // DEFERRED: R4 — qglColor4ubv/qglPushMatrix/qglTranslatef/qglRotatef
        // (see doc comment above) (DEC-37 A13.2)
        // Source: oracle/codemp/renderer/tr_backend.cpp:1514-1518

        GL_Bind(gpu, Some(image));

        // DEFERRED: R4 — the qglBegin(GL_QUADS)/qglTexCoord2f/qglVertex2f
        // quad and qglEnd/qglPopMatrix (see doc comment above) (DEC-37 A13.2)
        // Source: oracle/codemp/renderer/tr_backend.cpp:1521-1536
    }
}

/// Raven `RB_RotatePic2` — `RB_RotatePic`'s centered-rotation twin: rotates a
/// stretched pic about its own center rather than its top-right corner, and
/// additionally applies the shader's first stage's blend state before
/// drawing. The oracle's `data`/`cmd + 1` command-buffer walk dissolves — the
/// caller supplies the already-decoded `FrameEvent::DrawRotatePic2` payload
/// directly (`R2-D2`/A1); `cmd->shader` becomes the `shader: ShaderHandle`
/// payload field.
///
/// Two landable guards: `shader->numUnfoggedPasses` is a real
/// `ShaderAsset::num_unfogged_passes` field, and `image = &shader->stages[0]
/// .bundle[0].image[0]` is a plain re-fetch of `bundle[0].image` (a real
/// nullable pointer — see `RB_RotatePic`'s doc comment), now
/// `ShaderStage::image`. An invalid/stale `shader` handle, or one whose
/// `stages` is still populated empty (`GeneratePermanentShader`'s per-stage
/// copy loop is a separate, later wave), both fall back to "no passes"/"no
/// image" (skip drawing) rather than the oracle's implicit
/// always-valid-pointer assumption (porting-rules §19).
///
/// `shader->stages[0].stateBits` feeding the first `GL_State` call is also
/// real, as `ShaderStage::state_bits`.
///
/// DEFERRED: R4 — past both guards, every effect (`qglColor4ubv`/
/// `qglPushMatrix`/`qglTranslatef`/`qglRotatef`, `GL_Bind`/the first
/// `GL_State`'s own innards, the `qglBegin(GL_QUADS)`/`qglTexCoord2f`/
/// `qglVertex2f` quad, `qglEnd`/`qglPopMatrix`, and the trailing "Hmmm, this
/// is not too cool" `GL_State(GLS_DEPTHTEST_DISABLE | GLS_SRCBLEND_SRC_ALPHA
/// | GLS_DSTBLEND_ONE_MINUS_SRC_ALPHA)` restore) is GL-only; the trailing
/// call's `GLS_*` flags are also left undecoded, same treatment as
/// `GL_State`'s own body (DEC-37 A13.2).
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1547-1607`
pub fn RB_RotatePic2(
    frame: &mut FrameState,
    gpu: &mut GpuResources,
    assets: &RenderAssets,
    shader: ShaderHandle,
    _x: f32,
    _y: f32,
    _w: f32,
    _h: f32,
    _a: f32,
    _s1: f32,
    _t1: f32,
    _s2: f32,
    _t2: f32,
) {
    let shader_asset = assets.shaders.get(shader);

    let num_unfogged_passes = shader_asset.map(|s| s.num_unfogged_passes).unwrap_or(0);

    if num_unfogged_passes == 0 {
        return;
    }

    let first_stage = shader_asset.and_then(|s| s.stages.first());
    let image = first_stage.and_then(|stage| stage.image);

    if let Some(image) = image {
        if !frame.projection_2d {
            RB_SetGL2D(frame, gpu, assets);
        }

        // Get our current blend mode, etc.
        let state_bits = first_stage.map(|stage| stage.state_bits).unwrap_or(0);
        GL_State(gpu, state_bits);

        // DEFERRED: R4 — qglColor4ubv/qglPushMatrix/qglTranslatef/qglRotatef
        // (see doc comment above) (DEC-37 A13.2)
        // Source: oracle/codemp/renderer/tr_backend.cpp:1571-1576

        GL_Bind(gpu, Some(image));

        // DEFERRED: R4 — the qglBegin(GL_QUADS)/qglTexCoord2f/qglVertex2f
        // quad, qglEnd/qglPopMatrix, and the trailing "Hmmm, this is not too
        // cool" GL_State restore (see doc comment above) (DEC-37 A13.2)
        // Source: oracle/codemp/renderer/tr_backend.cpp:1579-1602
    }
}

/// Raven `RB_ShowImages` — the `r_showImages` debug view: tiles every
/// registered image across the screen in a 20-column grid.
///
/// DEFERRED: R4 — `qglClear`/`qglFinish` and the per-image
/// `GL_Bind`/`qglBegin(GL_QUADS)`.../`qglEnd()` quad draw are GL-only
/// (DEC-37 A13.2); the `r_showImages->integer == 2` proportional-size branch
/// additionally needs `common` threaded for `Common::cvar` reads
/// (`RendererCvars` holds only `Option<CvarHandle>`, A13.1). The tile-grid
/// math (`x`/`y`/`w`/`h` per image) and the iteration walk are real CPU logic
/// and land here.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1776-1829`
pub fn RB_ShowImages(
    frame: &mut FrameState,
    gpu: &mut GpuResources,
    assets: &RenderAssets,
    cvars: &RendererCvars,
) {
    if !frame.projection_2d {
        RB_SetGL2D(frame, gpu, assets);
    }

    // DEFERRED: R4 — qglClear(GL_COLOR_BUFFER_BIT) / qglFinish() (DEC-37 A13.2)
    // Source: oracle/codemp/renderer/tr_backend.cpp:1785-1787

    let mut i: i32 = 0;
    let _ = R_Images_StartIteration(assets);
    let mut cursor = 0usize;
    while let Some(handle) = R_Images_GetNextIteration(assets, &mut cursor) {
        let w = (assets.glconfig.vid_width / 20) as f32;
        let h = (assets.glconfig.vid_height / 15) as f32;
        let x = (i % 20) as f32 * w;
        let y = (i / 20) as f32 * h;

        // DEFERRED: R4 — r_showImages->integer == 2 proportional resize
        // (`w *= image->width / 512.0; h *= image->height / 512.0;`): the
        // fn's carrier does not yet thread `common` for cvar reads (RendererCvars
        // holds only Option<CvarHandle>, A13.1)
        // Source: oracle/codemp/renderer/tr_backend.cpp:1802-1805
        let _ = (cvars, x, y, w, h);

        // DEFERRED: R4 — GL_Bind(image) + qglBegin(GL_QUADS)/qglTexCoord2f/
        // qglVertex2f x4/qglEnd() quad draw (GL-only, DEC-37 A13.2); GL_Bind's
        // own body is itself deferred pending the glState.currenttextures
        // cache wiring.
        // Source: oracle/codemp/renderer/tr_backend.cpp:1807-1821
        GL_Bind(gpu, Some(handle));

        i += 1;
    }

    // DEFERRED: R4 — qglFinish() (DEC-37 A13.2)
    // Source: oracle/codemp/renderer/tr_backend.cpp:1825
}
