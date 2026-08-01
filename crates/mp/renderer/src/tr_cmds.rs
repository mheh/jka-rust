//! Raven `tr_cmds.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_cmds.cpp`

// Raven-named functions keep their original casing across this
// transcription, matching the rest of the renderer crate.
#![allow(non_snake_case)]

use mp_engine_qcommon::common::{com_error, com_printf, Common, EngineHostView};
use mp_engine_qcommon::cvar_fns::Cvar_Set;
use mp_qshared::common::mp::cgame::stereo_frame_t::{
    stereoFrame_t, STEREO_CENTER, STEREO_LEFT, STEREO_RIGHT,
};
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::qhandle_t;

use crate::render_state::frame_data::FrameData;
use crate::render_state::frame_event::FrameEvent;
use crate::render_state::frame_state::FrameState;
use crate::render_state::gpu_resources::GpuResources;
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::tr_image::{GL_TextureMode, R_SetColorMappings, TrImageState};
use crate::tr_scene::{R_ToggleSmpFrame, SceneState};
use crate::tr_shader::R_GetShaderByHandle;

/// Raven `R_InitCommandBuffers` — command-buffer subsystem init.
///
/// Raven: retail body is empty; command-buffer state is now `FrameData`,
/// built per render pass rather than a persistent buffer requiring init.
/// Source: `oracle/codemp/renderer/tr_cmds.cpp:72-73`
pub fn r_init_command_buffers() {}

/// Raven `R_ShutdownCommandBuffers` — command-buffer subsystem shutdown.
///
/// Raven: retail body is empty.
/// Source: `oracle/codemp/renderer/tr_cmds.cpp:80-81`
pub fn r_shutdown_command_buffers() {}

/// Raven `R_IssueRenderCommands`.
///
/// Raven's `cmdList = &backEndData->commands;`/`assert(cmdList)`/append-
/// `RC_END_OF_LIST`/`cmdList->used = 0` bookkeeping has no R3 equivalent:
/// `backEndData_t` dissolves under R2 ("its field list is the reference
/// vocabulary for `FrameData`'s event payloads, not a struct that
/// survives") and `FrameData` is built fresh per pass rather than a
/// persistent byte buffer needing a fresh end-of-list marker or a
/// used-length reset — the same reasoning as this file's
/// `r_init_command_buffers`/`r_shutdown_command_buffers` empty bodies.
/// `runPerformanceCounters` → `bool` (qboolean translation dictionary).
/// `r_skipBackEnd` reads through the live engine cvar table
/// (`RendererCvars::r_skipBackEnd`, DEC-37 A13.1).
///
/// The dispatch itself is render-side under DEC-50 (`FrameExecutor::
/// execute_frame`), so this trap-time fn starts nothing.
///
/// Source: `oracle/codemp/renderer/tr_cmds.cpp:88-110`
pub fn R_IssueRenderCommands(
    common: &Common,
    cvars: &RendererCvars,
    run_performance_counters: bool,
) {
    // at this point, the back end thread is idle, so it is ok
    // to look at it's performance counters
    if run_performance_counters {
        // DEFERRED: R_PerformanceCounters — no R2 carrier exists for this
        // callee's own state (`tr.pc`/`backEnd.pc`/`tr.viewParms.zFar` all
        // UNMAPPED; `tr.viewCluster` is homed now as
        // `FrameState::view_cluster` — see this file's own
        // `R_PerformanceCounters` DEFERRED note below), so no fn exists in
        // this crate to call. This wave's packet lists `R_PerformanceCounters`
        // as already ported in wave 2 under `tr_cmds.cpp`; that claim is
        // false — grepping the crate finds only the DEFERRED note, no
        // callable fn, loud or otherwise. Flagged as a wave-planning defect
        // per the preamble's never-guess rule rather than inventing a call.
        // Source: oracle/codemp/renderer/tr_cmds.cpp:101-103
    }

    // The oracle starts the backend on the accumulated command list here.
    // Under DEC-50 the command list IS `FrameData::events`, and the
    // render-side executor (`FrameExecutor::execute_frame` in
    // `mp_renderer_gpu`) is the ported dispatch. Ruling 3 keeps that
    // dispatch off this trap-time fn, so nothing starts here.
    //
    // The `r_skipBackEnd` guard moved to the executor. The caller resolves the
    // handle into `RenderCvarSnapshot::skip_back_end` and passes the snapshot
    // to `execute_frame`, which gates the whole replay at its top under the
    // same `!r_skipBackEnd` test.
    // Source: oracle/codemp/renderer/tr_cmds.cpp:105-109
    let _ = (common, cvars);
}

// DEFERRED: R_GetCommandBuffer — `backEndData_t` (and its byte-packed
// `renderCommandList_t`/`cmds` buffer this fn hand-allocates slices of)
// dissolves under R2; `commands` IS `FrameData.events: Vec<FrameEvent>`.
// RC_* command payloads cross as typed `FrameEvent` variants pushed directly
// onto `FrameData.events` in their owning waves, never through a raw
// byte-buffer allocator — a Rust equivalent of this fn's `void*`-slice-
// carving would require raw pointers, banned by the interior-safety law.
// (R2 `## State ownership` row `backEndData`; R2 `### A1 disposition table`)
// Source: `oracle/codemp/renderer/tr_cmds.cpp:140-160`

// DEFERRED: R_AddDrawSurfCmd — the A1 disposition table's `RC_DRAW_SURFS`
// row rules this command "stays render-side": `drawSurfsCommand_t`'s
// `refdef`/`viewParms` inputs already cross via `FrameEvent::RenderScene`
// (pushed by `RE_RenderScene`, a different, not-yet-ported fn), and
// `drawSurfs` itself is cull/sort output the render thread computes locally,
// "never a channel payload". There is no remaining R2-carrier behavior for
// this fn to perform: the render-thread-local hand-off from cull/sort output
// to the backend's draw step is the owning `tr_main`/`tr_backend` wave's
// concern, not a `FrameData` push.
// (R2 `### A1 disposition table` rows `RC_DRAW_SURFS` / `drawSurfsCommand_t`;
// R2 Group-3 tier-2 audit row `drawSurfsCommand_t`)
// Source: `oracle/codemp/renderer/tr_cmds.cpp:169-183`

/// Raven `RE_SetColor`.
///
/// Raven's `rgba` is a nullable `const float *`; a null pointer means "use
/// opaque white" (the fn-scope `colorWhite` static — a kind-1 const table
/// per the three-kind rule, never mutated, so it becomes a plain `const`
/// rather than any owning field). The nullable-pointer input becomes
/// `Option<[f32; 4]>` (interior-safety law: no raw pointers) with the guard
/// preserved faithfully.
///
/// Source: `oracle/codemp/renderer/tr_cmds.cpp:193-211`
pub fn RE_SetColor(frame: &mut FrameData, rgba: Option<[f32; 4]>) {
    /// Raven fn-scope `static float colorWhite[4] = { 1, 1, 1, 1 };` — a
    /// kind-1 const table (never mutated), so a plain `const` replaces it.
    const COLOR_WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

    let color = rgba.unwrap_or(COLOR_WHITE);
    frame.events.push(FrameEvent::SetColor(color));
}

/// Raven `RE_StretchPic`.
///
/// Source: `oracle/codemp/renderer/tr_cmds.cpp:219-237`
#[allow(clippy::too_many_arguments)]
pub fn RE_StretchPic(
    frame: &mut FrameData,
    assets: &RenderAssets,
    common: &mut Common,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    s1: f32,
    t1: f32,
    s2: f32,
    t2: f32,
    h_shader: qhandle_t,
) {
    let shader = R_GetShaderByHandle(assets, common, h_shader);
    frame.events.push(FrameEvent::DrawStretchPic {
        x,
        y,
        w,
        h,
        s1,
        t1,
        s2,
        t2,
        shader,
    });
}

/// Raven `RE_RotatePic`.
///
/// Source: `oracle/codemp/renderer/tr_cmds.cpp:244-263`
#[allow(clippy::too_many_arguments)]
pub fn RE_RotatePic(
    frame: &mut FrameData,
    assets: &RenderAssets,
    common: &mut Common,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    s1: f32,
    t1: f32,
    s2: f32,
    t2: f32,
    a: f32,
    h_shader: qhandle_t,
) {
    let shader = R_GetShaderByHandle(assets, common, h_shader);
    frame.events.push(FrameEvent::DrawRotatePic {
        x,
        y,
        w,
        h,
        s1,
        t1,
        s2,
        t2,
        angle: a,
        shader,
    });
}

/// Raven `RE_RotatePic2`.
///
/// Source: `oracle/codemp/renderer/tr_cmds.cpp:270-289`
#[allow(clippy::too_many_arguments)]
pub fn RE_RotatePic2(
    frame: &mut FrameData,
    assets: &RenderAssets,
    common: &mut Common,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    s1: f32,
    t1: f32,
    s2: f32,
    t2: f32,
    a: f32,
    h_shader: qhandle_t,
) {
    let shader = R_GetShaderByHandle(assets, common, h_shader);
    frame.events.push(FrameEvent::DrawRotatePic2 {
        x,
        y,
        w,
        h,
        s1,
        t1,
        s2,
        t2,
        angle: a,
        shader,
    });
}

// DEFERRED: RE_RenderWorldEffects — the A1 disposition table's `RC_WORLD_
// EFFECTS` row folds this command into `FrameEvent::WorldEffectCommand`,
// but that variant's payload (`String`) is built by a different, not-yet-
// ported oracle fn (the `CG_R_WORLDEFFECTCOMMAND` handler, `RE_WorldEffect-
// Command` per the trap-table naming), not by this one — `RE_RenderWorld-
// Effects` takes no arguments and, per its own threading digest, carries no
// state channel at all now that `R_GetCommandBuffer`'s buffer is dissolved.
// A bare bufferless `RC_WORLD_EFFECTS` marker has no `FrameEvent` variant in
// scope for this wave to push, and inventing one collides with the existing
// `WorldEffectCommand(String)` variant's name/shape without a design
// ruling reconciling the two RC_WORLD_EFFECTS producers. Escalated for the
// wave that ports `RE_WorldEffectCommand`.
// (R2 `### A1 disposition table` row `RC_WORLD_EFFECTS`)
// Source: `oracle/codemp/renderer/tr_cmds.cpp:291-300`

// DEFERRED: RE_RenderAutoMap — the A1 disposition table's `RC_AUTO_MAP` row
// splits this command: `AutomapElevAdj` already crosses as its own
// `FrameEvent` from a different fn, and `InitWireframeAuto`'s sim-side
// rebuild is likewise a different fn's concern. This fn's own job — queuing
// the bare `RC_AUTO_MAP` backend-draw marker — is explicitly not yet scoped:
// "the render command's full struct beyond the bare enum tag ... gets its
// targeted oracle read at the first automap wave (A7, R2-D8)"
// (`render_state/placeholders.rs`'s `AutomapWireframe` doc comment says the
// same). No `FrameEvent` variant exists for this wave to push.
// (R2 `### A1 disposition table` row `RC_AUTO_MAP`; `R2-D8`)
// Source: `oracle/codemp/renderer/tr_cmds.cpp:302-312`

// DEFERRED: R_PerformanceCounters — every branch past the `r_speeds` cvar
// check, plus the unconditional reset every path (including the early
// `!r_speeds->integer` return) shares at the end, needs state homes this
// wave cannot supply:
//   - `tr.pc` (`frontEndCounters_t`) has no R2/placeholder home at all. The
//     `## State ownership` row for `tr`'s "frontend scratch/counters"
//     bucket names `FrameState` only generically — no wave has landed a
//     `pc` field there, and `render_state::placeholders` has no
//     `FrontEndCounters` type. UNMAPPED, not invented, per the preamble's
//     "leave a cited `// DEFERRED:`... do NOT create a field" rule.
//   - `backEnd.pc` (`backEndCounters_t`'s `c_shaders`/`c_surfaces`/
//     `c_vertexes`/`c_indexes`/`c_totalIndexes`/`c_overDraw`/
//     `c_dlightVertexes`/`c_dlightIndexes`/`c_flareAdds`/`c_flareTests`/
//     `c_flareRenders` fields) are explicitly owned by "the R4 backend
//     wave", per `BackEndCounters`'s own doc comment
//     (`render_state/placeholders.rs:176-181`) — not this R3 wave.
//   - `tr.viewParms.zFar` is explicitly owned by "the tr_main R3 wave", per
//     `ViewParms`'s own doc comment (`render_state/placeholders.rs:159-166`)
//     — also not this wave.
//   - `tr.viewCluster` is no longer a blocker: campaign #41 batch 1 homed it
//     as `FrameState::view_cluster`. The three rows above still stand, and
//     every path shares them, so the fn stays deferred.
// No branch — including the `r_speeds->integer == 7` texture/buffer-size
// print, whose own inputs (`glConfig`, `RendererCvars::r_texturebits`) are
// otherwise available — can be transcribed standalone, because every path
// shares the trailing unconditional reset of both blocked counter structs.
// Nothing in the ported crate calls `R_PerformanceCounters` yet (its sole
// oracle caller, `RB_ExecuteRenderCommands`, is superseded by the
// render-side executor per DEC-50), so no stub is required for compilation.
// (R2 `## State ownership` row `tr` frontend scratch/counters, row
// `backEnd`; `placeholders.rs` `BackEndCounters`/`ViewParms` doc comments)
// Source: `oracle/codemp/renderer/tr_cmds.cpp:12-64`

/// Raven `R_SyncRenderThread`.
///
/// Raven's `#ifndef _XBOX` wraps the entire body; this target is not Xbox,
/// so the guard *includes* it and the whole body is transcribed below — the
/// same platform-guard convention the rest of this crate follows (the
/// mirror-image case, an `#ifdef _XBOX` block dropped on a non-Xbox build,
/// is `RB_SwapBuffers`'s own doc comment, `tr_backend.rs`).
///
/// Source: `oracle/codemp/renderer/tr_cmds.cpp:123-130`
pub fn R_SyncRenderThread(assets: &RenderAssets, common: &Common, cvars: &RendererCvars) {
    if !assets.registered {
        return;
    }
    R_IssueRenderCommands(common, cvars, false);
}

/// Raven `RE_EndFrame`.
///
/// Source: `oracle/codemp/renderer/tr_cmds.cpp:441-475`
pub fn RE_EndFrame(
    frame: &mut FrameData,
    scene: &mut SceneState,
    assets: &RenderAssets,
    common: &Common,
    cvars: &RendererCvars,
) {
    if !assets.registered {
        return;
    }

    // DEFERRED: `cmd = R_GetCommandBuffer(sizeof(*cmd)); if (!cmd) return;
    // cmd->commandId = RC_SWAP_BUFFERS;` — `backEndData_t`'s byte-packed
    // command buffer dissolves under R2 (this file's own `R_GetCommandBuffer`
    // DEFERRED note, above); `RC_SWAP_BUFFERS` is a frame-orchestration
    // command `R_IssueRenderCommands` issues directly rather than a
    // trap-pushed `FrameEvent` (`RB_ExecuteRenderCommands`'s doc comment,
    // `tr_backend.rs`, lists it among the five deliberately-absent `RC_*`
    // tags). No allocation-failure early return survives an unbounded `Vec`.
    // (R2 `### A1 disposition table`; this file's `R_GetCommandBuffer`
    // DEFERRED note)
    // Source: oracle/codemp/renderer/tr_cmds.cpp:447-451

    R_IssueRenderCommands(common, cvars, true);

    // use the other buffers next frame, because another CPU
    // may still be rendering into the current ones
    R_ToggleSmpFrame(frame, scene);

    // DEFERRED: `if (frontEndMsec) *frontEndMsec = tr.frontEndMsec;
    // tr.frontEndMsec = 0; if (backEndMsec) *backEndMsec = backEnd.pc.msec;
    // backEnd.pc.msec = 0;` — neither `tr.frontEndMsec` nor `backEnd.pc.msec`
    // has an R2 carrier: `FrameState` (the `tr` frontend-scratch/`backEnd`
    // home) has no `frontEndMsec` field, and `BackEndCounters`
    // (`FrameState::counters`) is the established empty tier-3 placeholder,
    // `msec` explicitly owned by "the R4 backend wave" — the same UNMAPPED
    // finding this file's own `R_PerformanceCounters` DEFERRED note and
    // `RB_ExecuteRenderCommands`'s (`tr_backend.rs`) terminal-write DEFERRED
    // note both cite. The nullable out-params drop from this fn's signature
    // (translation dictionary: out-params → returns) rather than returning
    // invented values.
    // (R2 `## State ownership` rows `tr` frontend scratch/counters, `backEnd`)
    // Source: oracle/codemp/renderer/tr_cmds.cpp:467-474
}

/// Raven `RE_BeginFrame`.
///
/// `glState.finishCalled = qfalse;` — DEFERRED: `GpuResources::gl_state`
/// (`GlStatePlaceholder`) is a field-less named placeholder until R4 defines
/// the real pipeline/bind-group cache (R2 `## State ownership` row
/// `glState`, `R2-D1`/B6); nothing to write to.
///
/// `tr.frameSceneNum = 0;` — DEFERRED: `tr`'s frontend-scratch bucket routes
/// to `FrameState` per this packet's STATE HOMES table, but only
/// `frame_count` (matching `tr.frameCount`) has landed there — no
/// `frame_scene_num` field exists. UNMAPPED, not invented (preamble: "leave
/// a cited `// DEFERRED:`... do NOT create a field").
///
/// The stencil-overdraw block, the texture-mode/gamma `R_SyncRenderThread`
/// gates, and the GL-error check are transcribed in full for their CPU-side
/// cvar logic (`Cvar_Set`/`modified`-flag bookkeeping/`Com_Printf`); every
/// `qgl*` call each block guards (`qglEnable`/`qglStencilMask`/
/// `qglClearStencil`/`qglStencilFunc`/`qglStencilOp`/`qglDisable`/
/// `qglGetError`) is DEC-01/DEC-37's fixed-function GL surface, unhomed until
/// R4 (A13.2) — left as a cited `// DEFERRED: R4` in place.
///
/// The "draw buffer stuff" tail dissolves per this file's own
/// `R_GetCommandBuffer` DEFERRED note (`backEndData_t`'s byte-packed command
/// buffer has no R3 successor) and the R2 `### A1 disposition table`'s
/// `RC_DRAW_BUFFER`/`drawBufferCommand_t` rows ("stays render-side ... no
/// sim-thread payload"): no `FrameEvent` variant and no render-thread
/// carrier exists for "which GL draw buffer to select", so `cmd->buffer`'s
/// `GL_BACK_LEFT`/`GL_BACK_RIGHT`/`GL_BACK` assignments have nowhere to land
/// and are never computed (no numeric GL enum is invented for a value
/// nothing consumes). The `Com_Error` protocol-violation guards survive —
/// a fatal panic is real, R3-representable behavior independent of the
/// dissolved payload, matching the oracle's own validate-then-act shape.
///
/// Source: `oracle/codemp/renderer/tr_cmds.cpp:322-431`
#[allow(clippy::too_many_arguments)]
pub fn RE_BeginFrame(
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    assets: &RenderAssets,
    frame: &mut FrameState,
    image_state: &mut TrImageState,
    gpu: &mut GpuResources,
    stereo_frame: stereoFrame_t,
) {
    if !assets.registered {
        return;
    }

    // `glState.finishCalled = qfalse;` — DEFERRED (see doc comment above)
    // Source: oracle/codemp/renderer/tr_cmds.cpp:328

    frame.frame_count += 1;
    // `tr.frameSceneNum = 0;` — DEFERRED (see doc comment above)
    // Source: oracle/codemp/renderer/tr_cmds.cpp:331

    // do overdraw measurement
    if view.common.cvar(cvars.r_measureOverdraw).integer != 0 {
        if assets.glconfig.stencil_bits < 4 {
            com_printf(
                view.common,
                &format!(
                    "Warning: not enough stencil bits to measure overdraw: {}\n",
                    assets.glconfig.stencil_bits
                ),
            );
            Cvar_Set(view, "r_measureOverdraw", "0");
            view.common.cvar_mut(cvars.r_measureOverdraw).modified = false;
        } else if view.common.cvar(cvars.r_shadows).integer == 2 {
            com_printf(
                view.common,
                "Warning: stencil shadows and overdraw measurement are mutually exclusive\n",
            );
            Cvar_Set(view, "r_measureOverdraw", "0");
            view.common.cvar_mut(cvars.r_measureOverdraw).modified = false;
        } else {
            R_SyncRenderThread(assets, view.common, cvars);
            // DEFERRED: R4 — qglEnable(GL_STENCIL_TEST); qglStencilMask(~0U);
            // qglClearStencil(0U); qglStencilFunc(GL_ALWAYS, 0U, ~0U);
            // qglStencilOp(GL_KEEP, GL_INCR, GL_INCR) (DEC-37 A13.2)
            // Source: oracle/codemp/renderer/tr_cmds.cpp:353-358
        }
        view.common.cvar_mut(cvars.r_measureOverdraw).modified = false;
    } else {
        // this is only reached if it was on and is now off
        if view.common.cvar(cvars.r_measureOverdraw).modified {
            R_SyncRenderThread(assets, view.common, cvars);
            // DEFERRED: R4 — qglDisable(GL_STENCIL_TEST) (DEC-37 A13.2)
            // Source: oracle/codemp/renderer/tr_cmds.cpp:367
        }
        view.common.cvar_mut(cvars.r_measureOverdraw).modified = false;
    }

    // texturemode stuff
    if view.common.cvar(cvars.r_textureMode).modified
        || view
            .common
            .cvar(cvars.r_ext_texture_filter_anisotropic)
            .modified
    {
        R_SyncRenderThread(assets, view.common, cvars);
        let texture_mode = view.common.cvar(cvars.r_textureMode).string.clone();
        GL_TextureMode(view, cvars, assets, image_state, gpu, &texture_mode);
        view.common.cvar_mut(cvars.r_textureMode).modified = false;
        view.common
            .cvar_mut(cvars.r_ext_texture_filter_anisotropic)
            .modified = false;
    }

    // gamma stuff
    if view.common.cvar(cvars.r_gamma).modified {
        view.common.cvar_mut(cvars.r_gamma).modified = false;

        R_SyncRenderThread(assets, view.common, cvars);
        R_SetColorMappings(view, cvars, &assets.glconfig, image_state, frame);
    }

    // check for errors
    if view.common.cvar(cvars.r_ignoreGLErrors).integer == 0 {
        R_SyncRenderThread(assets, view.common, cvars);
        // DEFERRED: R4 — int err = qglGetError(); if (err != GL_NO_ERROR)
        // Com_Error(ERR_FATAL, "RE_BeginFrame() - glGetError() failed
        // (0x%x)!\n", err); (DEC-37 A13.2; qglGetError is GL-only, no
        // CPU-side value to test)
        // Source: oracle/codemp/renderer/tr_cmds.cpp:394-400
    }

    // draw buffer stuff — see doc comment above: the command payload
    // dissolves, only the fatal-error validation survives.
    if assets.glconfig.stereo_enabled {
        if stereo_frame == STEREO_LEFT {
            // `cmd->buffer = (int)GL_BACK_LEFT;` — DEFERRED (see doc comment)
        } else if stereo_frame == STEREO_RIGHT {
            // `cmd->buffer = (int)GL_BACK_RIGHT;` — DEFERRED (see doc comment)
        } else {
            com_error(
                errorParm_t::ERR_FATAL,
                format!("RE_BeginFrame: Stereo is enabled, but stereoFrame was {stereo_frame}"),
            );
        }
    } else {
        if stereo_frame != STEREO_CENTER {
            com_error(
                errorParm_t::ERR_FATAL,
                format!("RE_BeginFrame: Stereo is disabled, but stereoFrame was {stereo_frame}"),
            );
        }
        // `cmd->buffer = (int)GL_BACK;` — DEFERRED (see doc comment above)
    }
}
