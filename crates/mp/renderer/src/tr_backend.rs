//! Raven `tr_backend.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_backend.cpp`

// Raven-named functions/types keep their original casing across this
// transcription, matching the rest of the renderer/engine crates.
#![allow(non_snake_case)]

use mp_engine_qcommon::common::com_error;
use mp_engine_qcommon::common::com_printf;
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::Common;
use mp_engine_qcommon::timing::sys_milliseconds;
use mp_qshared::shared::error_parm::errorParm_t;
use native_math::rng::Rng;

use crate::render_state::frame_data::FrameData;
use crate::render_state::frame_event::FrameEvent;
use crate::render_state::frame_state::FrameState;
use crate::render_state::image_asset::ImageHandle;
use crate::render_state::placeholders::Vec3;
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::render_state::shader_asset::ShaderHandle;
use crate::tr_cmds::R_SyncRenderThread;
use crate::tr_image::{
    PendingUpload, R_Images_GetNextIteration, R_Images_StartIteration, TrImageState,
};
use crate::tr_local::cull_type_t::cullType_t;
use crate::tr_main::{DrawSurf, SurfaceGeometry};
use crate::tr_public::ref_flags::RDF_NOWORLDMODEL;
use crate::tr_worldeffects::world_effects::{WindZoneState, WorldEffectsState};

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
/// DEFERRED: R4 — `RenderAssets::default_image`/`dlight_image` have landed,
/// but every value this fn actually compares and stores has not:
/// `ImageAsset::texnum` (the whole `texnum` decision, R4 GPU wave),
/// `ImageAsset::frame_used` against `FrameState::frame_count`, and the
/// render thread's `currenttextures`/`currenttmu` cache (DEC-63.4). The
/// bind decision and the `qglBindTexture` call are GL-only regardless
/// (DEC-37 A13.2).
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:61-82`
pub fn GL_Bind(_image: Option<ImageHandle>) {
    // DEFERRED: R4 — GL_Bind (see doc comment above) (DEC-37 A13.2)
    // Source: oracle/codemp/renderer/tr_backend.cpp:61-82
}

/// Raven `GL_Bind3D` — `GL_Bind`'s `GL_TEXTURE_3D` twin; identical texnum
/// resolution, same `glState.currenttextures[currenttmu]` cache compare.
///
/// DEFERRED: R4 — same dependency set as `GL_Bind` (DEC-37 A13.2).
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:85-107`
pub fn GL_Bind3D(_image: Option<ImageHandle>) {
    // DEFERRED: R4 — GL_Bind3D (see doc comment above) (DEC-37 A13.2)
    // Source: oracle/codemp/renderer/tr_backend.cpp:85-107
}

/// Raven `GL_SelectTexture` — selects a texture unit (TMU) for subsequent
/// texture state changes; `unit` must be `0..=3`.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:112-152`
pub fn GL_SelectTexture(unit: i32) {
    match unit {
        0..=3 => {
            // DEFERRED: R4 — GL_SelectTexture glState.currenttmu
            // cache-compare, qglActiveTextureARB/qglClientActiveTextureARB
            // per unit, and the GLimp_LogComment trace calls (the render
            // thread owns glState, DEC-63.4)
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
pub fn GL_Cull(frame: &FrameState, cull_type: cullType_t) {
    // DEFERRED: R4 — GL_Cull glState.faceCulling cache-compare + write
    // (the render thread owns the GL binding cache, DEC-63.4) (DEC-37 A13.2)
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
pub fn GL_TexEnv(env: u32) {
    match env {
        GL_MODULATE | GL_REPLACE | GL_DECAL | GL_ADD => {
            // DEFERRED: R4 — GL_TexEnv glState.texEnv[currenttmu]
            // cache-compare + qglTexEnvf(GL_TEXTURE_ENV, GL_TEXTURE_ENV_MODE,
            // env) (the render thread owns glState, DEC-63.4)
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
/// both reads and writes the render thread's GL state cache (DEC-63.4). The
/// `GLS_*` bit-flag `#define`s this decodes are not yet ported to Rust
/// consts — left undecoded rather than guessed at (DEC-37 A13.2).
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:244-431`
pub fn GL_State(_state_bits: u32) {
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
///   `RDF_HYPERSPACE` bit tests this fn guards on are left undecoded — the
///   masks themselves are no longer the gap (`tr_public::ref_flags` is the
///   crate's canonical flag home), the `rdflags` operand is;
/// - `ViewParms`/`OrientationR` (`FrameState::view`/`ori`) are still empty
///   stubs — `isPortal`/`portalPlane`/`ori.axis` land with the `tr_main`
///   wave;
/// - `tr.world`'s `globalFog`/`fogs` land with the `tr_bsp`/`tr_world` wave;
/// - `g_bRenderGlowingObjects`/`skyboxportal` are homed now
///   (`FrameState::render_glowing_objects`/`skyboxportal`, campaign #41
///   batch 1, DEC-37 A13.3), but every read of them here sits behind the
///   blocked cvar/`TrRefdef`/`ViewParms` state above; `tr_stencilled` still
///   has no carrier (DEC-37 A13.3 — a per-subsystem state struct is named by
///   whichever fn's wave actually reads/writes it, and none of that state is
///   reached by the portable slice below);
/// - every `qgl*` call (`qglFinish`/`qglClearColor`/`qglClear`/
///   `qglLoadMatrixf`/`qglClipPlane`/`qglEnable`/`qglDisable`) is GL-only —
///   the render thread owns the GL state cache (DEC-63.4, DEC-37 A13.2).
///
/// Only the one unconditional field write lands here: `backEnd.projection2D
/// = qfalse;`. `SetViewportAndScissor`/`GL_State`/`RB_Hyperspace` (the
/// wave-0 in-module callees) are not invoked — every call site downstream of
/// them is gated by the unresolved state above.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:477-593`
pub fn RB_BeginDrawingView(
    frame: &mut FrameState,
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
pub fn RB_SetGL2D(frame: &mut FrameState, _assets: &RenderAssets) {
    frame.projection_2d = true;

    // DEFERRED: R4 — the rest of RB_SetGL2D (see doc comment above) (DEC-37
    // A13.2; R2 `## State ownership` trRefdef_t row)
    // Source: oracle/codemp/renderer/tr_backend.cpp:1269-1291
}

/// Re-specifies `tr.scratchImage[client]` from a decoded cinematic frame, and
/// returns the scratch handle so a caller that also draws can name it.
///
/// This is the upload half both `RE_StretchRaw` and `RE_UploadCinematic` run.
/// Raven branches on whether `(cols, rows)` still match the texture it built
/// last time: a size change re-specifies the whole texture with
/// `qglTexImage2D`, and a same-size `dirty` frame goes in with
/// `qglTexSubImage2D`. Both branches stage one `PendingUpload`, because
/// `GpuImages::upload_pending` rebuilds the wgpu texture either way.
///
/// A clean same-size frame stages nothing, exactly as Raven uploads nothing.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1327-1344,1367-1395`
fn R_UploadScratchFrame(
    assets: &mut RenderAssets,
    img_state: &mut TrImageState,
    cols: i32,
    rows: i32,
    data: &[u8],
    client: i32,
    dirty: bool,
) -> Option<ImageHandle> {
    let handle = *assets.scratch_images.get(client as usize)?;
    let asset = assets.images.get_mut(handle)?;

    let resized = cols != asset.width || rows != asset.height;
    if resized {
        asset.width = cols;
        asset.height = rows;
    } else if !dirty {
        return Some(handle);
    }

    // DEFERRED: R4. Raven splits the two branches at the GL call, using
    // `qglTexImage2D` to re-specify and `qglTexSubImage2D` to update in place.
    // `GpuImages::upload_pending` has one path and it recreates the texture, so
    // a same-size dirty frame pays a full re-specify until that crate grows a
    // write-only update. The pixels that reach the GPU are the same either way.
    // Source: oracle/codemp/renderer/tr_backend.cpp:1341-1343
    img_state.pending_uploads.insert(
        handle,
        PendingUpload {
            pixels: data.to_vec(),
            width: cols,
            height: rows,
        },
    );
    Some(handle)
}

/// Raven `RE_UploadCinematic` — (re)uploads a cinematic video frame into the
/// per-client scratch texture `tr.scratchImage[client]` and draws nothing.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1367-1395`
pub fn RE_UploadCinematic(
    assets: &mut RenderAssets,
    img_state: &mut TrImageState,
    cols: i32,
    rows: i32,
    data: &[u8],
    client: i32,
    dirty: bool,
) {
    R_UploadScratchFrame(assets, img_state, cols, rows, data, client, dirty);
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
/// concern owned by the render thread, DEC-63.4); `g_bTextureRectangleHack`
/// is homed outside this TU with no confirmed receiver; every `qgl*` call is
/// GL-only and the `glState.currenttmu` write belongs to the render thread
/// (DEC-37 A13.2).
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:2015-2189`
pub fn RB_BlurGlowTexture(
    _frame: &FrameState,
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
pub fn RB_DrawGlowOverlay(_assets: &RenderAssets, _cvars: &RendererCvars) {
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
/// not a structurally-non-null address-of; read here as
/// `stages[0].bundle[0].image`, with the `if (image)` guard as an `Option`
/// check. A stale/invalid `shader` handle, or one whose `stages` came out
/// empty (`GeneratePermanentShader`'s copy loop `break`s on the first
/// inactive stage), falls through the same `None` path as a genuinely unset
/// image, matching the oracle's "skip drawing" outcome (porting-rules §19).
///
/// DEFERRED: R4 — past that guard, every effect (`qglColor4ubv`/
/// `qglPushMatrix`/`qglTranslatef`/`qglRotatef`, `GL_Bind`'s own innards, the
/// `qglBegin(GL_QUADS)`/`qglTexCoord2f`/`qglVertex2f` quad, and
/// `qglEnd`/`qglPopMatrix`) is GL-only (DEC-37 A13.2).
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1498-1540`
pub fn RB_RotatePic(
    frame: &mut FrameState,
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
        .and_then(|stage| stage.bundle[0].image);

    if let Some(image) = image {
        if !frame.projection_2d {
            RB_SetGL2D(frame, assets);
        }

        // DEFERRED: R4 — qglColor4ubv/qglPushMatrix/qglTranslatef/qglRotatef
        // (see doc comment above) (DEC-37 A13.2)
        // Source: oracle/codemp/renderer/tr_backend.cpp:1514-1518

        GL_Bind(Some(image));

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
/// nullable pointer — see `RB_RotatePic`'s doc comment), read here as
/// `stages[0].bundle[0].image`. An invalid/stale `shader` handle, or one
/// whose `stages` came out empty (`GeneratePermanentShader`'s copy loop
/// `break`s on the first inactive stage), both fall back to "no passes"/"no
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
    let image = first_stage.and_then(|stage| stage.bundle[0].image);

    if let Some(image) = image {
        if !frame.projection_2d {
            RB_SetGL2D(frame, assets);
        }

        // Get our current blend mode, etc.
        let state_bits = first_stage.map(|stage| stage.state_bits).unwrap_or(0);
        GL_State(state_bits);

        // DEFERRED: R4 — qglColor4ubv/qglPushMatrix/qglTranslatef/qglRotatef
        // (see doc comment above) (DEC-37 A13.2)
        // Source: oracle/codemp/renderer/tr_backend.cpp:1571-1576

        GL_Bind(Some(image));

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
    assets: &RenderAssets,
    cvars: &RendererCvars,
) {
    if !frame.projection_2d {
        RB_SetGL2D(frame, assets);
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
        GL_Bind(Some(handle));

        i += 1;
    }

    // DEFERRED: R4 — qglFinish() (DEC-37 A13.2)
    // Source: oracle/codemp/renderer/tr_backend.cpp:1825
}

/// Raven `MAX_POST_RENDERS` - the post-render queue depth `RB_RenderDrawSurfList` fills.
/// A surface past the cap falls through to the normal sorted draw instead of deferring.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:655`
pub const MAX_POST_RENDERS: usize = 128;

/// Raven `RB_RenderDrawSurfList` — the core backend draw loop: walks a sorted
/// `drawSurf_t` list, decomposing each surface's sort key into
/// entity/shader/fog/dlight, batching consecutive surfaces that share a
/// (shader, fog, dlight, entity-mergable) key into one `RB_BeginSurface`/
/// `RB_EndSurface` tess batch, deferring distortion/force-alpha/force-post
/// entities into a last `g_postRenders` pass, then re-drawing that pass.
///
/// Every real dependency this loop touches is still unhomed at this wave:
/// - `tess` dissolves into R4's tessellation/vertex-building pipeline (R2
///   `## State ownership` row `tess`) — `tess.shaderTime`/`.shader` have no
///   R3 carrier, so the `RB_ShadowFinish`'s `!didShadowPass && shader &&
///   shader->sort > SS_BANNER` gate and every `tess.shaderTime = ...` write
///   fall with it.
/// - `FrameState::counters` (`backEnd.pc`, `BackEndCounters`) is still the
///   empty tier-3 placeholder — no field backs `backEnd.pc.c_surfaces +=
///   numDrawSurfs`.
/// - `TrRefdef` (`FrameState::refdef`) only carries `fov_x`/`fov_y`/
///   `view_origin`/`view_axis` — no `entities`/`num_dlights`/`dlights`/
///   `floatTime`, so every `backEnd.refdef.entities[entityNum]` /
///   `.dlights` read (the RF_DISTORTION/RF_FORCEPOST/RF_FORCE_ENT_ALPHA/
///   RF_NODEPTH/RF_DEPTHHACK renderfx tests, the postRender entity fetch)
///   is blocked.
/// - `tr.worldEntity` has no landed carrier — it is not one of the named
///   `## State ownership` `tr` sub-fields, and `FrameState::current_entity`/
///   `entity_2d` are the scene-entity/2D-entity slots, neither the world
///   entity default `backEnd.currentEntity = &tr.worldEntity;` needs.
/// - `ViewParms`/`OrientationR` (`FrameState::view`/`ori`) are still empty
///   placeholders, yet `R_RotateForEntity`'s already-ported signature
///   (`tr_main.rs`) takes the tier-2 raw `viewParms_t` and returns a raw
///   `orientationr_t` — there is no owned field to store that result in
///   without reintroducing a tier-2 shape into new state (interior-safety
///   law), so the call is not reachable as landed.
/// - `rb_surfaceTable[*drawSurf->surface]`'s per-surface-kind dispatch has no
///   ported callees in this wave's resolved call surface (`RB_SurfaceFace`/
///   `RB_SurfaceGrid`/... are a later wave) — `DrawSurf<S>::surface`'s tagged
///   dispatch stays unresolved regardless of the state above.
/// - `g_bRenderGlowingObjects` is homed now
///   (`FrameState::render_glowing_objects`, campaign #41 batch 1, DEC-37
///   A13.3); `g_postRenders`/`g_numPostRenders`/`rb_surfaceTable`/
///   `tr_stencilled` are still this packet's STATE HOMES rows marked "NAMED
///   BY THIS WAVE if this file's wave is where the subsystem lands". Either
///   way every real read/write site sits behind the blocked state above (the
///   postRender decision itself needs
///   `backEnd.refdef.entities[entityNum].e.renderfx`), so there is no
///   landable use site here.
/// - `qglLoadMatrixf`/`qglDepthRange`/`qglCopyTexImage2D` are GL-only
///   (DEC-37 A13.2).
/// - `RB_CaptureScreenImage`/`RB_DistortionFill`/`RB_ShadowFinish`/
///   `RB_BeginSurface`/`RB_EndSurface`/`R_TransformDlights`/
///   `R_WorldCoordToScreenCoord`/`GL_Bind` are all callable (in-module,
///   already ported), but every call site in this body is gated by the
///   blocked state above.
///
/// No computation survives once every input above is removed; the
/// `#ifdef __MACOS__`/`_XBOX` branches (`Sys_PumpEvents` event-pump crutch,
/// the `#if 0` dead texture-copy experiment, the commented-out distortion
/// alternate paths) are not compiled on this target and drop with it.
///
/// Whole-body deferral: no partial body survives, so this lands as a loud
/// `todo!()` rather than a silent no-op (whole-fn-deferral convention —
/// partial-body fns keep DEFERRED comments instead).
///
/// The post-render arm of this loop is live in the GPU backend: `Pipeline3d::draw` defers, reverses and replays those surfaces.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:705-1249`
pub fn RB_RenderDrawSurfList(
    frame: &mut FrameState,
    assets: &RenderAssets,
    draw_surfs: &[DrawSurf<SurfaceGeometry<'_>>],
) {
    let _ = (frame, assets, draw_surfs);
    todo!("Port RB_RenderDrawSurfList — oracle/codemp/renderer/tr_backend.cpp:705-1249")
}

/// Raven `RB_SwapBuffers` — the `RC_SWAP_BUFFERS` backend command: flushes
/// any in-flight 2D tess batch, runs the `r_showImages` debug view, measures
/// overdraw via a stencil readback, finishes the GL pipe if needed, logs a
/// frame-boundary trace comment, and presents the frame. The oracle's
/// `data`/`cmd + 1` command-buffer walk dissolves — `swapBuffersCommand_t`
/// carries no payload past its `RC_SWAP_BUFFERS` tag, so this fn takes no
/// decoded-event argument (`R2-D2`/A1).
///
/// - `tess.numIndexes`/`RB_EndSurface()` flush and `tess.numIndexes`-summed
///   overdraw/`glState.finishCalled` gate both depend on state the render
///   thread now owns — `tess` dissolves into R4's pipeline (R2 `## State
///   ownership` row `tess`), `gl_state` belongs to the render thread
///   (DEC-63.4, DEC-37 A13.2).
/// - `r_showImages->integer` gates a real call: `common` threaded for
///   `Common::cvar` reads is established practice (`tr_image.rs`'s
///   anisotropy-clamp read is precedent), so this lands as
///   `RB_ShowImages(frame, assets, cvars)`.
/// - `r_measureOverdraw`'s live integer value gates the stencil-readback
///   block (`Hunk_AllocateTempMemory`/`qglReadPixels`/`Hunk_FreeTempMemory`,
///   `backEnd.pc.c_overDraw += sum`), which is additionally blocked by
///   `FrameState::counters` (`BackEndCounters`) still being the empty tier-3
///   placeholder — no field to accumulate into — so that block stays
///   deferred.
/// - `qglFinish()` is GL-only (DEC-37 A13.2).
/// - `GLimp_LogComment`'s already-ported signature takes a raw `*mut c_char`
///   (tier-1-adjacent engine surface) — same "would need an unsafe pointer
///   construction the interior-safety law forbids" ruling `RB_EndSurface`'s
///   port made for this exact call (`tr_shade.rs`).
/// - `GLimp_EndFrame`/`GLimp_LogComment` additionally have no reachable path
///   from this crate at all: both live in `mp_engine_client::null::
///   null_glimp`, but `crates/mp/renderer/Cargo.toml` does not depend on
///   `mp_engine_client` — escalated rather than adding an undeclared
///   cross-crate edge out of this packet's scope.
///
/// The `r_showImages->integer` gated `RB_ShowImages()` call and the
/// unconditional `backEnd.projection2D = qfalse;` field write land here.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1838-1884`
pub fn RB_SwapBuffers(
    frame: &mut FrameState,
    assets: &RenderAssets,
    cvars: &RendererCvars,
    common: &Common,
) {
    // DEFERRED: R4/A13.1 — the rest of RB_SwapBuffers (see doc comment
    // above), including the GLimp_EndFrame present call — GLimp_EndFrame has
    // no reachable path from this crate today (mp_renderer does not depend
    // on mp_engine_client; escalated)
    // Source: oracle/codemp/renderer/tr_backend.cpp:1842-1880

    // texture swapping test
    if common.cvar(cvars.r_showImages).integer != 0 {
        RB_ShowImages(frame, assets, cvars);
    }

    frame.projection_2d = false;
}

/// Raven `RB_WorldEffects` — the `RC_WORLD_EFFECTS` backend command: flushes
/// any in-flight tess batch, runs the outdoor weather/wind particle-cloud
/// pass, then re-opens a tess batch for whatever shader was still active. The
/// oracle's `data`/`cmd + 1` command-buffer walk dissolves — the caller
/// supplies whatever `RC_WORLD_EFFECTS` needs directly (`R2-D2`/A1);
/// `drawBufferCommand_t` carries no fields this fn reads.
///
/// The two `tess`-gated flush/reopen calls have no guard left to evaluate —
/// `tess` dissolves into R4's tessellation/vertex-building pipeline (R2 `##
/// State ownership` row `tess`; no R3 carrier ever holds `tess.shader`/
/// `.numIndexes`/`.fogNum`) — so `RB_EndSurface`/`RB_BeginSurface` stay
/// uncalled here rather than guessed at. `RB_RenderWorldEffects` itself
/// carries no `tess` gate and lands unconditionally, threaded per its
/// already-ported signature (`tr_worldeffects/world_effects.rs`).
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1886-1905`
pub fn RB_WorldEffects(
    world_effects: &mut WorldEffectsState,
    wind: &mut WindZoneState,
    assets: &RenderAssets,
    frame: &FrameState,
    host: &mut EngineHostView,
    rng: &mut Rng,
) {
    // DEFERRED: R4 — tess.shader && tess.numIndexes guard + RB_EndSurface()
    // flush (see doc comment above)
    // Source: oracle/codemp/renderer/tr_backend.cpp:1893-1896

    world_effects.RB_RenderWorldEffects(wind, assets, frame, host, rng);

    // DEFERRED: R4 — tess.shader guard + RB_BeginSurface(tess.shader,
    // tess.fogNum) reopen (see doc comment above)
    // Source: oracle/codemp/renderer/tr_backend.cpp:1899-1902
}

/// Raven `RB_StretchPic` — the `RC_STRETCH_PIC` backend command: draws a
/// screen-space stretched quad into the current tess batch. The oracle's
/// `data`/`cmd + 1` command-buffer walk dissolves — the caller supplies the
/// already-decoded `FrameEvent::DrawStretchPic` payload directly
/// (`R2-D2`/A1); `cmd->shader` becomes the `shader: ShaderHandle` payload
/// field per the tier-2 transition audit's `stretchPicCommand_t`/`shader_t`
/// rows.
///
/// One landable guard: `if (!backEnd.projection2D) RB_SetGL2D();`.
///
/// DEFERRED: R4 — every remaining line operates on `tess`
/// (`shaderCommands_s`), which dissolves entirely into R4's tessellation/
/// vertex-building pipeline (R2 `## State ownership` row `tess`; no R3
/// carrier holds `tess.shader`/`numIndexes`/`numVertexes`/`indexes`/
/// `vertexColors`/`xyz`/`texCoords`): the `shader != tess.shader`
/// batch-break test, the `RB_EndSurface`/`RB_BeginSurface` batch-open pair it
/// guards, the `backEnd.currentEntity = &backEnd.entity2D;` write nested
/// inside that same guard, `RB_CHECKOVERFLOW`'s bounds check (`RB_CheckOverflow`
/// is itself a `todo!()` stub for the same `tess` gap,
/// `crates/mp/renderer/src/tr_surface.rs:424`), and the six vertex/index/color/texcoord
/// writes that build the quad (`backEnd.color2D` also has no read site left
/// once its consumer, the vertex-color write, is unreachable) all stay
/// unreachable behind that missing carrier (DEC-37 A13.2).
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1422-1490`
pub fn RB_StretchPic(
    frame: &mut FrameState,
    assets: &RenderAssets,
    _x: f32,
    _y: f32,
    _w: f32,
    _h: f32,
    _s1: f32,
    _t1: f32,
    _s2: f32,
    _t2: f32,
    _shader: ShaderHandle,
) {
    if !frame.projection_2d {
        RB_SetGL2D(frame, assets);
    }

    // DEFERRED: R4 — the rest of RB_StretchPic (see doc comment above)
    // (DEC-37 A13.2; R2 `## State ownership` row `tess`)
    // Source: oracle/codemp/renderer/tr_backend.cpp:1433-1487
}

/// Raven `RB_DrawSurfs` — the `RC_DRAW_SURFS` backend command: draws the
/// sorted surface list for a view, then (retail, non-`_XBOX`) runs the
/// dynamic-glow pass when the world model is present and both the runtime
/// capability flag and the `r_DynamicGlow` cvar allow it. The oracle's
/// `data`/`cmd + 1` command-buffer walk dissolves — `drawSurfs` crosses as
/// the already-computed cull/sort output this fn receives directly
/// (`R2-D2`/A1).
///
/// PORT-NOTE: `backEnd.refdef = cmd->refdef; backEnd.viewParms =
/// cmd->viewParms;` do not land as field copies here. Per the A1 disposition
/// table's `RC_DRAW_SURFS` row and `tr_cmds.rs`'s `R_AddDrawSurfCmd` DEFERRED
/// note (same reasoning, cited there): `drawSurfsCommand_t` dissolves, and
/// `refdef`/`viewParms` already cross into `FrameState::refdef`/`view` via
/// `FrameEvent::RenderScene` at scene-seal time (`RE_RenderScene`, a
/// different, not-yet-ported fn) — "the render-thread-local hand-off from
/// cull/sort output to the backend's draw step is the owning
/// `tr_main`/`tr_backend` wave's concern, not a `FrameData` push". This fn is
/// that hand-off: it consumes the already-current `frame.refdef`/`view`
/// rather than re-copying them from a dissolved command struct.
///
/// DEFERRED: R4 — the leading "finish any 2D drawing if needed"
/// `tess.numIndexes` flush guard + `RB_EndSurface()` call: `tess` dissolves
/// into R4's tessellation/vertex-building pipeline (R2 `## State ownership`
/// row `tess`); no R3 carrier holds `tess.numIndexes`.
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1620-1622`
///
/// The dynamic-glow block's outer `!(backEnd.refdef.rdflags &
/// RDF_NOWORLDMODEL)` term IS landed: `tr.refdef.rdflags` is threaded in as
/// `refdef_rdflags: i32` — `TrRefdef` (`FrameState::refdef`) has no `rdflags`
/// field yet (it lands with the `tr_scene` wave, R2 `## State ownership`,
/// `trRefdef_t` row) — mirroring `tr_world.rs::R_AddWorldSurfaces`'s
/// identical threading of the same guard in this same batch.
///
/// DEFERRED: R4/DEC-37 A13.1-A13.3 — everything inside the dynamic-glow
/// block. The guard's remaining two terms are the blocker:
/// `g_bDynamicGlowSupported` (a GL runtime-capability flag) and
/// `r_DynamicGlow->integer` (a live cvar read, which needs `Common::cvar`,
/// not threaded to this fn) have no carrier — the same "stay unmapped rather
/// than invented" call `RB_RenderDrawSurfList`'s port made just above for
/// `g_postRenders`/`rb_surfaceTable`, and the
/// reason this fn takes no `RendererCvars` (a cvar handle table with no
/// `Common` to resolve it against buys nothing). The block's body is GL
/// surface throughout: `qglDisable`/`qglEnable`/`qglBindTexture`/
/// `qglCopyTexSubImage2D`/`qglClearColor`/`qglClear`/`qglFinish` against
/// `tr.sceneImage`/`tr.screenGlow`/`tr.blurImage`, `SetViewportAndScissor`'s
/// viewport-swap dance over `r_DynamicGlowWidth`/`r_DynamicGlowHeight`, and
/// `RB_BlurGlowTexture`/`RB_DrawGlowOverlay`'s own already-deferred bodies.
/// The one non-GL effect, the second `RB_RenderDrawSurfList` call bracketed
/// by the `g_bRenderGlowingObjects` write, now has its flag homed
/// (`FrameState::render_glowing_objects`, campaign #41 batch 1) — but it is
/// still unreachable, because the guard above it needs
/// `g_bDynamicGlowSupported`/`r_DynamicGlow` (DEC-37 A13.1, A13.3).
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1641-1697`
///
/// Panics via `RB_RenderDrawSurfList`'s loud stub until its owning wave lands.
pub fn RB_DrawSurfs(
    frame: &mut FrameState,
    assets: &RenderAssets,
    refdef_rdflags: i32,
    draw_surfs: &[DrawSurf<SurfaceGeometry<'_>>],
) {
    RB_RenderDrawSurfList(frame, assets, draw_surfs);

    // Render dynamic glowing/flaring objects.
    if refdef_rdflags & RDF_NOWORLDMODEL == 0 {
        // DEFERRED: R4/DEC-37 A13.1-A13.3 — the rest of the guard
        // (`g_bDynamicGlowSupported && r_DynamicGlow->integer`) and the whole
        // glow block behind it (see doc comment above).
        // Source: oracle/codemp/renderer/tr_backend.cpp:1641-1697
    }
}

// PORT-NOTE: `RB_ExecuteRenderCommands` (the backend command-list dispatch
// loop) is superseded, not stubbed. Under DEC-50 the command list IS
// `FrameData::events`, and the render-side executor
// (`mp_renderer_gpu::FrameExecutor::execute_frame`) is the ported dispatch:
// it walks the events in order and draws. The `backEnd.pc.msec` stamp waits
// for a `BackEndCounters` home with the other performance counters.
// Source: `oracle/codemp/renderer/tr_backend.cpp:1916-1959`

/// Raven `RE_StretchRaw` — (re)uploads a cinematic video frame into the
/// per-client scratch texture and draws it as a screen-space quad; the
/// direct-call twin of the command-buffer-routed [`RE_UploadCinematic`]
/// (same `tr.scratchImage[client]` target, same format-change/dirty-subimage
/// decision).
///
/// PORT-NOTE: the packet's RESOLVED CALL SURFACE lists `Sys_Milliseconds` as
/// `mp_engine_core::lifecycle::sys_milliseconds(engine: &Engine, base_time:
/// bool)`, but that fn is unreachable from this crate — `mp_engine_core`
/// already depends on `mp_renderer` (the reverse edge would cycle), the same
/// block `RE_Font_DrawString`'s port hit for the identical callee
/// (`tr_font.rs`). The oracle call site (`Sys_Milliseconds()`, no args) is
/// the base-relative clock, whose one real implementation is
/// `mp_engine_qcommon::timing::sys_milliseconds(common: &Common)` —
/// `lifecycle::sys_milliseconds`'s own `base_time == false` arm delegates to
/// it — and `mp_renderer` already depends on `mp_engine_qcommon`, so it is
/// called directly here.
///
/// `Sys_Milliseconds()*com_timescale->value` — `int * float` promotes the
/// `int` operand to `float` (C's usual arithmetic conversions only reach
/// `double` when an operand IS `double`/`long double`), so this stays an
/// `f32` product, no ruling-12 double intermediate.
///
/// DEFERRED: R4 — `qglFinish()` ("we definately want to sync every frame for
/// the cinematics") is GL-only (DEC-37 A13.2).
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1313-1314`
///
/// DEFERRED: R4. `qglColor3f(tr.identityLight x 3)` is GL-only, and the 2D
/// batch has no per-quad color channel of its own yet (DEC-37 A13.2).
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1353`
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1304-1365`
#[allow(clippy::too_many_arguments)]
pub fn RE_StretchRaw(
    frame: &mut FrameState,
    frame_data: &mut FrameData,
    assets: &mut RenderAssets,
    img_state: &mut TrImageState,
    cvars: &RendererCvars,
    common: &mut Common,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    cols: i32,
    rows: i32,
    data: &[u8],
    client: i32,
    dirty: bool,
) {
    if !assets.registered {
        return;
    }
    // `assets` is the single published registry.
    // Every call below reads it or writes it directly.
    R_SyncRenderThread(assets, common, cvars);

    // DEFERRED: R4 — qglFinish() (see doc comment above) (DEC-37 A13.2)
    // Source: oracle/codemp/renderer/tr_backend.cpp:1313-1314

    let start = if common.cvar(cvars.r_speeds).integer != 0 {
        Some((sys_milliseconds(common) as f32 * common.cvar(common.com_timescale).value) as i32)
    } else {
        None
    };

    // make sure rows and cols are powers of 2
    if (cols & (cols - 1)) != 0 || (rows & (rows - 1)) != 0 {
        com_error(
            errorParm_t::ERR_DROP,
            format!("Draw_StretchRaw: size not a power of 2: {cols} by {rows}"),
        );
    }

    let image = R_UploadScratchFrame(assets, img_state, cols, rows, data, client, dirty);

    // `r_speeds->integer` can't change between the two oracle checks (nothing
    // in between re-enters cvar code), so `start.is_some()` stands in for the
    // oracle's second, independently-re-read `if ( r_speeds->integer )`.
    if let Some(start) = start {
        let end =
            (sys_milliseconds(common) as f32 * common.cvar(common.com_timescale).value) as i32;
        com_printf(
            common,
            &format!("qglTexSubImage2D {cols}, {rows}: {} msec\n", end - start),
        );
    }

    RB_SetGL2D(frame, assets);

    // DEFERRED: R4. qglColor3f(tr.identityLight x 3) (see doc comment above)
    // Source: oracle/codemp/renderer/tr_backend.cpp:1353

    // A client number outside the scratch set has no texture to draw, so the
    // quad is dropped rather than pointed at another client's frame.
    let Some(image) = image else {
        return;
    };
    let (cols, rows) = (cols as f32, rows as f32);
    frame_data.events.push(FrameEvent::DrawStretchRaw {
        x: x as f32,
        y: y as f32,
        w: w as f32,
        h: h as f32,
        s1: 0.5 / cols,
        t1: 0.5 / rows,
        s2: (cols - 0.5) / cols,
        t2: (rows - 0.5) / rows,
        image,
    });
}
