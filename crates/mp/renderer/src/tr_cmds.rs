//! Raven `tr_cmds.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_cmds.cpp`

// Raven-named functions keep their original casing across this
// transcription, matching the rest of the renderer crate.
#![allow(non_snake_case)]

use mp_engine_qcommon::common::Common;
use mp_qshared::shared::qhandle_t;

use crate::render_state::frame_data::FrameData;
use crate::render_state::frame_event::FrameEvent;
use crate::render_state::render_assets::RenderAssets;
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
