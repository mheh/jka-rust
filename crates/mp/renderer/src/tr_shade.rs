//! Raven `tr_shade.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_shade.cpp`

// Raven-named functions/types keep their original casing across this
// transcription, matching the rest of the renderer/engine crates.
#![allow(non_snake_case)]
// Wave-0 ports of Raven `static` helpers: private by fidelity, with their
// callers landing in later R3 waves.
#![allow(dead_code)]

use crate::render_state::frame_state::FrameState;
use crate::render_state::gpu_resources::GpuResources;
use crate::render_state::light_style_table::LightStyleTable;
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::render_state::shader_asset::ShaderHandle;
use crate::render_state::shader_stage::ShaderStage;
use crate::tr_local::gl_index_t::glIndex_t;
use crate::tr_shader::TextureBundleParse;

/// Per-subsystem render-thread counters `R_DrawStripElements` accumulates
/// across calls.
///
/// Raven `static int c_begins`/`static int c_vertexes` — file-scope statics
/// in `tr_shade.cpp`, not part of `trGlobals_t`/`backEndCounters_t`; named
/// here per DEC-37 A13.3 since this wave lands their one write site.
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:63,75` (used throughout
/// `R_DrawStripElements`)
#[derive(Default)]
pub struct TrShadeCounters {
    pub c_begins: i32,
    pub c_vertexes: i32,
}

/// Raven `R_ArrayElementDiscrete` — emits one tessellated vertex (color,
/// texcoords, position) to the fixed-function GL immediate-mode pipeline by
/// index.
///
/// DEFERRED: R4 — every touched value lives on `tess` (dissolved into R4's
/// tessellation/vertex-building pipeline, R2 `## State ownership` row
/// `tess`; no R3 carrier holds `tess.svars.colors`/`texcoords`/`xyz` to read
/// from) and `glState.currenttmu` (`GpuResources::gl_state`, a named
/// placeholder until R4). The body is GL calls only (DEC-37 A13.2).
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:37-48`
fn R_ArrayElementDiscrete(_gpu: &mut GpuResources, _index: i32) {
    // DEFERRED: R4 — R_ArrayElementDiscrete (see doc comment above)
    // Source: oracle/codemp/renderer/tr_shade.cpp:37-48
}

/// Raven `R_DrawStripElements` — batches a triangle-strip index list into
/// one or more GL `GL_TRIANGLE_STRIP` runs, restarting the strip whenever the
/// next triangle doesn't share an edge with the previous one.
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:58-149`
fn R_DrawStripElements(
    indexes: &[glIndex_t],
    counters: &mut TrShadeCounters,
    mut element: impl FnMut(glIndex_t),
) {
    counters.c_begins += 1;

    if indexes.is_empty() {
        return;
    }

    // DEFERRED: R4 — qglBegin(GL_TRIANGLE_STRIP) (DEC-37 A13.2)
    // Source: oracle/codemp/renderer/tr_shade.cpp:69

    // prime the strip
    element(indexes[0]);
    element(indexes[1]);
    element(indexes[2]);
    counters.c_vertexes += 3;

    let mut last = [indexes[0], indexes[1], indexes[2]];
    let mut even = false;

    let mut i = 3;
    while i < indexes.len() {
        // odd numbered triangle in potential strip
        if !even {
            // check previous triangle to see if we're continuing a strip
            if indexes[i] == last[2] && indexes[i + 1] == last[1] {
                element(indexes[i + 2]);
                counters.c_vertexes += 1;
                // PORT-NOTE: dropped `assert( indexes[i+2] < tess.numVertexes )`
                // — tess is R4-dissolved (R2 `## State ownership` row `tess`),
                // no R3 carrier; debug-only bound check, no behavioral effect.
                even = true;
            }
            // otherwise we're done with this strip so finish it and start
            // a new one
            else {
                // DEFERRED: R4 — qglEnd/qglBegin(GL_TRIANGLE_STRIP) (DEC-37 A13.2)
                // Source: oracle/codemp/renderer/tr_shade.cpp:100-102
                counters.c_begins += 1;

                element(indexes[i]);
                element(indexes[i + 1]);
                element(indexes[i + 2]);

                counters.c_vertexes += 3;

                even = false;
            }
        } else {
            // check previous triangle to see if we're continuing a strip
            if last[2] == indexes[i + 1] && last[0] == indexes[i] {
                element(indexes[i + 2]);
                counters.c_vertexes += 1;

                even = false;
            }
            // otherwise we're done with this strip so finish it and start
            // a new one
            else {
                // DEFERRED: R4 — qglEnd/qglBegin(GL_TRIANGLE_STRIP) (DEC-37 A13.2)
                // Source: oracle/codemp/renderer/tr_shade.cpp:128-130
                counters.c_begins += 1;

                element(indexes[i]);
                element(indexes[i + 1]);
                element(indexes[i + 2]);
                counters.c_vertexes += 3;

                even = false;
            }
        }

        // cache the last three vertices
        last[0] = indexes[i];
        last[1] = indexes[i + 1];
        last[2] = indexes[i + 2];

        i += 3;
    }

    // DEFERRED: R4 — qglEnd (DEC-37 A13.2)
    // Source: oracle/codemp/renderer/tr_shade.cpp:148
}

/// Raven `RB_BeginSurface` — resets the tessellation buffer for a new
/// surface batch under `shader` (or its `remappedShader` twin), snapshotting
/// the shader's stage list, fog index, iterator function, and shader-relative
/// time into `tess`.
///
/// DEFERRED: R4 — the entire body writes `tess` (dissolved into R4's
/// tessellation/vertex-building pipeline, R2 `## State ownership` row
/// `tess`) from `shader_t` fields (`remappedShader`, `stages`, `sky`,
/// `numUnfoggedPasses`, `timeOffset`, `clampTime`) not yet landed on
/// `ShaderAsset` (`tr_shader` wave, `render_state::shader_asset`). No R3
/// carrier for either side.
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:362-382`
pub fn RB_BeginSurface(_frame: &mut FrameState, _shader: ShaderHandle, _fog_num: i32) {
    // DEFERRED: R4 — RB_BeginSurface (see doc comment above)
    // Source: oracle/codemp/renderer/tr_shade.cpp:362-382
}

/// Raven `ForceAlpha` — overwrites the alpha byte of every tessellated
/// vertex color in `dstColors` with a fixed value.
///
/// DEFERRED: R4 — the loop bound is `tess.numVertexes` (dissolved into R4's
/// tessellation/vertex-building pipeline, R2 `## State ownership` row
/// `tess`); no R3 carrier holds it.
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:1929-1939`
pub fn ForceAlpha(_dst_colors: &mut [u8], _force_ent_alpha: i32) {
    // DEFERRED: R4 — ForceAlpha (see doc comment above)
    // Source: oracle/codemp/renderer/tr_shade.cpp:1929-1939
}

/// Raven `R_DrawElements` — dispatches an index list either straight to
/// `qglDrawElements` in one shot, or to `R_DrawStripElements`'s
/// triangle-strip batching, chosen by the `r_primitives` cvar (falling back
/// on whether `qglLockArraysEXT` is bound when the cvar is unset).
///
/// DEFERRED: R4 — every branch is gated by state with no R3 carrier yet:
/// `qglLockArraysEXT`'s bound-or-not fallback has no R3 home (this packet's
/// `STATE HOMES` row for it: DEFERRED R4, DEC-37 A13.2); and every terminal
/// action is a GL call
/// (`qglDrawElements`/`qglArrayElement`, `R_ArrayElementDiscrete`) or the
/// `qglArrayElement` engine-seam receiver this wave doesn't resolve (this
/// packet's `STATE HOMES` rows for `qglArrayElement`/`qglDrawElements`: "NOT
/// renderer state ... confirm the exact receiver at port time"). No branch
/// has an observable non-GL effect once the cvar and the GL calls are
/// removed. The wave-0 in-module callee `R_DrawStripElements` is not
/// invoked — every call site downstream of it is gated by the unresolved
/// state above (same treatment as `RB_BeginDrawingView`'s deferred callees).
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:162-216`
pub fn R_DrawElements(_gpu: &mut GpuResources, _cvars: &RendererCvars, _indexes: &[glIndex_t]) {
    // DEFERRED: R4 — R_DrawElements (see doc comment above)
    // Source: oracle/codemp/renderer/tr_shade.cpp:162-216
}

/// Raven `R_BindAnimatedImage` — binds the correct frame of an animated (or
/// video-mapped, or fullbright-lightmap, or single-frame) texture bundle as
/// the active GL texture.
///
/// DEFERRED: R4 for every branch except the video-map one:
/// - the fullbright/lightmap fast path needs a live `r_fullbright->value`
///   read (`RendererCvars` holds only `Option<CvarHandle>` — the cvar-value
///   -read seam isn't wired yet, same reason as `RB_DrawBuffer`'s `r_clear`
///   dependency, DEC-37 A13.1); `tr.whiteImage` is homed
///   (`RenderAssets::white_image`) but only feeds the deferred `GL_Bind`;
/// - the single-frame path (`bundle->numImageAnimations <= 1`) is reachable
///   only once the fullbright branch above is known not to have returned,
///   so it inherits that same block;
/// - the animated-index path needs `backEnd.currentEntity->e.skinNum`
///   (`RefEntity` — `crate::render_state::placeholders` — has no `skin_num`
///   field landed by any prior wave; this file may not add one, porting-rules
///   §17/the wave contract restricts it to `tr_shade.rs`) in the
///   `RF_SETANIMINDEX` leg, and `tess.shaderTime` in the `else` leg (`tess`
///   is R4-dissolved outright — R2 `## State ownership` row `tess` — no R3
///   carrier exists for it, ever);
/// - every leg's terminal action is `GL_Bind`, itself a DEFERRED-R4 GL call
///   (DEC-37 A13.2).
///
/// The video-map branch (`bundle->isVideoMap`) is otherwise fully
/// determined by `bundle` alone (no cvar/global dependency), so it is
/// transcribed; its two callees are a genuine out-of-packet dependency (this
/// packet's `RESOLVED CALL SURFACE`: "`CIN_RunCinematic`/`CIN_UploadCinematic`:
/// NOT RESOLVED in the workspace ... escalate, never stub").
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:239-290`
pub fn R_BindAnimatedImage(
    _gpu: &mut GpuResources,
    _assets: &RenderAssets,
    _frame: &FrameState,
    _cvars: &RendererCvars,
    bundle: &TextureBundleParse,
) {
    if bundle.is_video_map {
        //TODO: Port CIN_RunCinematic
        //TODO: Port CIN_UploadCinematic
        // Source: oracle/codemp/renderer/tr_shade.cpp:243-244 (client-side
        // cinematic surface, not resolved anywhere in the workspace)
        todo!("Port CIN_RunCinematic/CIN_UploadCinematic — oracle/codemp/renderer/tr_shade.cpp:243-244");
    }

    // DEFERRED: R4 — everything past the video-map check (see doc comment
    // above)
    // Source: oracle/codemp/renderer/tr_shade.cpp:248-289
}

/// Raven `DrawNormals` — debug-draws each tessellated vertex's normal as a
/// short `GL_LINES` segment, gated behind `r_shownormals`.
///
/// DEFERRED: R4 — every touched value lives on `input` (`shaderCommands_t
/// *`, the same type as the dissolved `tess` — R2 `## State ownership` row
/// `tess`: "dissolved into R4's tessellation/vertex-building pipeline ...
/// no single global scratch buffer survives the new topology"; no R3 type
/// exists for `numVertexes`/`xyz`/`normal` to be read from) and every call
/// is GL-only (`qglBegin`/`qglColor3f`/`qglDepthRange`/`qglEnd`/
/// `qglVertex3fv`, plus the wave-0 `GL_Bind`/`GL_State`, DEC-37 A13.2). No
/// computation survives once both are removed — the loop's only output
/// (`temp`, via `VectorMA`) feeds straight into `qglVertex3fv` and is never
/// otherwise observed.
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:333-351`
pub fn DrawNormals(_gpu: &mut GpuResources) {
    // DEFERRED: R4 — DrawNormals (see doc comment above)
    // Source: oracle/codemp/renderer/tr_shade.cpp:333-351
}

/// Raven `DrawTris` — draws the shader's wireframe-triangle overlay
/// (`r_showtris`): binds the white image, sets line-polygon-mode GL state,
/// disables the color/texcoord client arrays, feeds the vertex pointer, and
/// draws the element list.
///
/// DEFERRED: R4 — every step is a GL call (`qglColor3f`/`qglDepthRange`/
/// `qglDisableClientState`/`qglVertexPointer`/`qglLockArraysEXT`/
/// `qglUnlockArraysEXT`, DEC-37 A13.2) or a wave-0/1 in-module callee that is
/// itself deferred to R4 (`GL_Bind`/`GL_State`/`R_DrawElements`); its
/// `shaderCommands_t *input` parameter is the same dissolved type as `tess`
/// (R2 `## State ownership` row `tess` — "no single global scratch buffer
/// survives the new topology"), so no R3 carrier holds `input->xyz`/
/// `numVertexes`/`numIndexes`/`indexes` to read from. No computation survives
/// once both are removed.
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:299-323`
pub fn DrawTris(_gpu: &mut GpuResources) {
    // DEFERRED: R4 — DrawTris (see doc comment above)
    // Source: oracle/codemp/renderer/tr_shade.cpp:299-323
}

/// Raven `DrawMultitextured` — draws a two-pass multitextured surface: binds
/// stage `stage`'s base texture on TMU 0, then TMU 1's lightmap/secondary
/// pass (or `GL_REPLACE` under `r_lightmap`), then draws.
///
/// DEFERRED: R4 — `pStage = &tess.xstages[stage]` and `input->svars.texcoords`
/// read the dissolved `tess`/`shaderCommands_t` (R2 `## State ownership` row
/// `tess`; no R3 carrier); `tess.shader->multitextureEnv` needs the same
/// dissolved receiver even though `ShaderAsset::multitexture_env` itself has
/// landed — the chooser (`tess.shader`) has not; `r_lightmap->integer` needs
/// a live cvar-value read (`RendererCvars` holds only `Option<CvarHandle>`,
/// unwired — same DEFERRED reason as `RB_DrawBuffer`'s `r_clear` dependency,
/// DEC-37 A13.1); `backEnd.viewParms.isPortal` has no landed field
/// (`ViewParms` is still the empty tier-3 placeholder — fields land with the
/// `tr_main` R3 wave) and its only effect is a GL call anyway; every
/// terminal action is a GL call or a deferred in-module callee
/// (`GL_SelectTexture`/`GL_State`/`GL_TexEnv`/`R_BindAnimatedImage`/
/// `R_DrawElements`, DEC-37 A13.2). No computation survives once all of the
/// above are removed.
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:394-443`
pub fn DrawMultitextured(_gpu: &mut GpuResources, _stage: i32) {
    // DEFERRED: R4 — DrawMultitextured (see doc comment above)
    // Source: oracle/codemp/renderer/tr_shade.cpp:394-443
}

/// Raven `ProjectDlightTexture2` — for each active dynamic light, clip-tests
/// every tessellated vertex against the light's radius, builds a per-triangle
/// projected dlight texture-coordinate/color hit list for the triangles that
/// pass, then draws that hit list additively (optionally through an
/// unfogged shader stage).
///
/// DEFERRED: R4 — every input is unavailable at this R3 wave:
/// `backEnd.refdef.num_dlights`/`.dlights[l]` need fields `TrRefdef` doesn't
/// carry (only `fov_x`/`fov_y`/`view_origin`/`view_axis` landed by the
/// `tr_backend` wave-0; `TrRefdef`'s array fields cross as
/// `FrameEvent::RenderScene` payloads per R2, not a render-thread-local
/// dlights array); `tess.dlightBits`/`.numVertexes`/`.xyz`/`.numIndexes`/
/// `.indexes`/`.shader`/`.fogNum`/`.svars.texcoords`/`.texCoords` are the
/// dissolved `tess` (R2 `## State ownership` row `tess`, no R3 carrier
/// ever); `tess.shader->stages`/`.bundle` need `ShaderAsset::stages`, not
/// landed by any prior wave (only `name`/`lightmap_index`/`styles`/`sort`/
/// `sorted_index`/`surface_flags`/`content_flags`/`multitexture_env`/
/// `default_shader`/`explicitly_defined`/`num_unfogged_passes`/`sky` are
/// real); `tr.world->globalFog`/`.numfogs` need fields `WorldAsset` doesn't
/// carry (only `name`/`shaders`/`bmodels`/`planes`/`nodes`/`mark_surfaces`/
/// light-grid/`vis`/`novis`/entity-string fields are real); `r_drawfog->value`
/// needs a live cvar-value read (unwired, DEC-37 A13.1, same reason as
/// above); `backEnd.pc.c_totalIndexes`/`c_dlightIndexes` need fields
/// `BackEndCounters` doesn't carry (still the empty tier-3 placeholder,
/// fields land with the R4 backend wave). Every terminal action past the
/// math is a GL call or a deferred in-module callee (`GL_Bind`/
/// `GL_SelectTexture`/`GL_State`/`GL_TexEnv`/`R_BindAnimatedImage`/
/// `R_DrawElements`, DEC-37 A13.2); `qglIsEnabled`/`qglActiveTextureARB`/
/// `qglLockArraysEXT`/`qglUnlockArraysEXT` gate branches with no
/// CPU-observable output once the GL calls themselves are removed. No
/// computation survives once every input above is removed — this includes
/// the entire clip/dot-product/triangle-hit math, since its sole inputs are
/// the dissolved `tess`/un-landed `backEnd.refdef` fields.
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:523-838`
pub fn ProjectDlightTexture2(_gpu: &mut GpuResources) {
    // DEFERRED: R4 — ProjectDlightTexture2 (see doc comment above)
    // Source: oracle/codemp/renderer/tr_shade.cpp:523-838
}

/// Raven `ProjectDlightTexture` — the non-triangle-hit-list twin of
/// `ProjectDlightTexture2`: for each active dynamic light, computes a
/// per-vertex projected dlight texture coordinate/color (choosing the
/// dominant vertex-normal axis to project along), clips per vertex, then
/// builds and draws the triangle hit list for triangles whose vertices
/// weren't all clipped.
///
/// DEFERRED: R4 — same unavailable-input set as `ProjectDlightTexture2`
/// (`backEnd.refdef.num_dlights`/`.dlights[l]`: `TrRefdef` doesn't carry
/// them; `tess.dlightBits`/`.numVertexes`/`.xyz`/`.normal`/`.numIndexes`/
/// `.indexes`/`.shader`/`.fogNum`/`.svars.texcoords`: the dissolved `tess`,
/// no R3 carrier ever; `tess.shader->stages`/`.bundle`: `ShaderAsset::stages`
/// not landed; `tr.world->globalFog`/`.numfogs`: `WorldAsset` fields not
/// landed; `r_drawfog->value`: unwired cvar-value read, DEC-37 A13.1;
/// `backEnd.pc.c_dlightVertexes`: `BackEndCounters` empty placeholder), plus
/// `vec3_origin` — **not** renderer state (this packet's `STATE HOMES`: "NOT
/// renderer state ... homed by the engine port ... confirm the exact
/// receiver at port time"), so `VectorCompare(tess.normal[i], vec3_origin)`
/// has no resolvable receiver even before `tess.normal` is considered. Every
/// terminal action is a GL call or a deferred in-module callee (`GL_Bind`/
/// `GL_SelectTexture`/`GL_State`/`GL_TexEnv`/`R_BindAnimatedImage`/
/// `R_DrawElements`, DEC-37 A13.2); `qglIsEnabled`/`qglActiveTextureARB` gate
/// branches with no CPU-observable output once the GL calls themselves are
/// removed. No computation survives once every input above is removed —
/// this includes the entire dominant-axis/clip/modulate math, since its sole
/// inputs are the dissolved `tess`/un-landed `backEnd.refdef` fields.
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:840-1170`
pub fn ProjectDlightTexture(_gpu: &mut GpuResources) {
    // DEFERRED: R4 — ProjectDlightTexture (see doc comment above)
    // Source: oracle/codemp/renderer/tr_shade.cpp:840-1170
}

/// Raven `RB_FogPass` — draws a fog pass over the current tessellated
/// surface: writes the fog volume's packed color into every vertex,
/// computes fog texture coordinates via `RB_CalcFogTexCoords`, binds the fog
/// image, and draws.
///
/// DEFERRED: R4 — every input is unavailable at this R3 wave:
/// `tess.svars.colors`/`.svars.texcoords`/`.numVertexes`/`.numIndexes`/
/// `.indexes`/`.shader`/`.fogNum` are the dissolved `tess` (R2 `## State
/// ownership` row `tess`, no R3 carrier ever); `tr.world->fogs` needs a
/// `WorldAsset::fogs` field not landed by any prior wave (only `name`/
/// `shaders`/`bmodels`/`planes`/`nodes`/`mark_surfaces`/light-grid/`vis`/
/// `novis`/entity-string are real); the wave-0 in-module callee
/// `RB_CalcFogTexCoords` takes the same dissolved `tess.svars.texcoords`
/// buffer as its sole argument, so it has no value to call with; every
/// terminal action is a GL call or a deferred in-module callee (`GL_Bind`/
/// `GL_State`/`R_DrawElements`, DEC-37 A13.2). No computation survives once
/// both are removed.
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:1182-1209`
pub fn RB_FogPass(_gpu: &mut GpuResources) {
    // DEFERRED: R4 — RB_FogPass (see doc comment above)
    // Source: oracle/codemp/renderer/tr_shade.cpp:1182-1209
}

/// Raven `RB_EndSurface` — closes out the current tessellated surface batch:
/// validates the tess-buffer overflow guards, special-cases the shadow
/// shader, applies the sort-order/skybox-portal debug cutoffs, accumulates
/// performance counters, dispatches to the shader's stage-iterator function,
/// then draws the `r_showtris`/`r_shownormals` debug overlays and resets
/// `tess.numIndexes`.
///
/// DEFERRED: R4 — every input is unavailable at this R3 wave: `input->
/// numIndexes`/`.indexes`/`.xyz`/`.shader`/`.currentStageIteratorFunc`/
/// `.numVertexes`/`.fogNum`/`.numPasses` are the dissolved `tess`
/// (`shaderCommands_t`, R2 `## State ownership` row `tess`; no R3 carrier
/// ever) — including the very first guard (`input->numIndexes == 0`), so no
/// downstream line is even reachable without it; `tr.shadowShader` has no
/// `RenderAssets` field landed by any prior wave (only the tier-2
/// `tr_globals_t::shadowShader` raw pointer exists, scaffolding this wave may
/// not extend); `skyboxportal`/`drawskyboxportal` are homed now
/// (`FrameState::skyboxportal`/`drawskyboxportal`, campaign #41 batch 1,
/// DEC-37 A13.3) but this wave's one fn only reads them, behind the
/// dissolved `tess`, so nothing lands from them;
/// `backEnd.refdef.rdflags` needs a `TrRefdef` field not landed (only
/// `fov_x`/`fov_y`/`view_origin`/`view_axis` are real —
/// `render_state::placeholders`); `backEnd.pc.c_shaders`/`c_vertexes`/
/// `c_indexes`/`c_totalIndexes` need `BackEndCounters` fields not landed
/// (still the empty tier-3 placeholder, fields land with the R4 backend
/// wave); `com_developer`/`com_sv_running` are this packet's `STATE HOMES`
/// "NOT renderer state ... confirm the exact receiver at port time" rows,
/// unconfirmed; `RB_StageIteratorSky`'s fn-pointer comparison
/// (`tess.currentStageIteratorFunc == RB_StageIteratorSky`) has no receiver
/// for the same `tess` reason. `GLimp_LogComment`'s already-ported signature
/// takes a raw `*mut c_char` (tier-1-adjacent engine surface, not this file's
/// to reshape) — calling it would need an unsafe pointer construction the
/// interior-safety law forbids, moot regardless since that call is only
/// reached past every guard above. No computation survives once every input
/// above is removed.
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:2391-2474`
pub fn RB_EndSurface(_gpu: &mut GpuResources) {
    todo!("Port RB_EndSurface — oracle/codemp/renderer/tr_shade.cpp:2391-2474")
}

/// Raven `ComputeTexCoords` — for one shader stage, generates the base
/// texture coordinates for each of `NUM_TEXTURE_BUNDLES` bundles
/// (`TCGEN_IDENTITY`/`TEXTURE`/`LIGHTMAP`.../`VECTOR`/`FOG`/
/// `ENVIRONMENT_MAPPED`), then walks that bundle's `texMods` list applying
/// the `TMOD_*` coordinate modifiers (turbulent/scroll/scale/stretch/
/// transform/rotate/entity-translate) in sequence. Oracle's signature takes
/// only `shaderStage_t *pStage` — no `frame`/`tess` argument — so this fn's
/// parameter is `_stage: &ShaderStage`, not `FrameState`.
///
/// DEFERRED: R4 — every input and the sole output are unavailable at this R3
/// wave: the write target `tess.svars.texcoords[b]` and the read sources
/// `tess.numVertexes`/`.texCoords`/`.xyz` are the dissolved `tess` (R2 `##
/// State ownership` row `tess`, no R3 carrier ever — including the loop
/// bound, so no `TCGEN_*`/`TMOD_*` case is even reachable without it); the
/// error-path read `tess.shader->name` is the same dissolved receiver.
/// `pStage->bundle[b]` (`tcGen`/`tcGenVectors`/`numTexMods`/`texMods`) needs
/// per-bundle fields `ShaderStage` doesn't carry — only `image`/`state_bits`/
/// `active` are real (`render_state::shader_stage`'s own doc comment: "The
/// remaining `shaderStage_t` fields (`bundle[1]`, ... `index`,
/// `lightmapStyle`, `isDetail`) have no reader yet"; `bundle[0]`'s own
/// `tcGen`/`tcGenVectors`/`texMods` aren't among the landed fields either).
/// The `TMOD_ENTITY_TRANSLATE` leg additionally needs
/// `backEnd.currentEntity->e.shaderTexCoord`, a `RefEntity` field no prior
/// wave landed (`render_state::placeholders::RefEntity`'s doc comment: real
/// fields land "field-by-field as call sites need them" — `shaderTexCoord`
/// is not among them, and this file may not add one out of the wave that
/// actually reads it downstream). No computation survives once every input
/// above is removed — this includes the entire per-bundle `TCGEN_*`
/// dispatch and the `TMOD_*` modifier loop, since their sole inputs are the
/// dissolved `tess` and the un-landed `pStage`/`backEnd.currentEntity`
/// fields; the wave-0..3 in-module callees this fn would otherwise call
/// (`RB_CalcEnvironmentTexCoords`/`RB_CalcFogTexCoords`/
/// `RB_CalcRotateTexCoords`/`RB_CalcScaleTexCoords`/`RB_CalcScrollTexCoords`/
/// `RB_CalcStretchTexCoords`/`RB_CalcTransformTexCoords`/
/// `RB_CalcTurbulentTexCoords`) all take the same dissolved `tess.xyz`/
/// `tess.svars.texcoords[b]` buffers as their `st`/`xyz` arguments, so none
/// has a value to call with here.
///
/// Whole-body deferral: no partial body survives, so this lands as a loud
/// `todo!()` rather than a silent no-op (whole-fn-deferral convention —
/// partial-body fns keep DEFERRED comments instead).
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:1809-1927`
pub fn ComputeTexCoords(_stage: &ShaderStage) {
    todo!("Port ComputeTexCoords — oracle/codemp/renderer/tr_shade.cpp:1809-1927")
}

/// Raven `ComputeColors` — for one shader stage, generates the tessellated
/// vertex colors (dispatching on `rgbGen`/`forceRGBGen`: identity,
/// identity-lighting, diffuse, diffuse-entity, exact-vertex, const, vertex,
/// one-minus-vertex, fog, waveform, entity, one-minus-entity, lightmap-style)
/// and then the alpha channel (dispatching on `alphaGen`: skip, identity,
/// const, waveform, lighting-specular, entity, one-minus-entity, vertex,
/// one-minus-vertex, portal, blend), with a `RF_DISINTEGRATE1`/
/// `RF_DISINTEGRATE2`/`RF_VOLUMETRIC` special-case short-circuit ahead of
/// both switches and a fog-fade adjustment (`adjustColorsForFog`) after.
///
/// DEFERRED: R4 — whole-body deferral, both switches' *dispatch keys* are
/// unavailable, not just their arms: `pStage->rgbGen`/`alphaGen`/
/// `constantColor`/`rgbWave`/`alphaWave`/`adjustColorsForFog`/
/// `lightmapStyle`/`index` are exactly the `shaderStage_t` fields
/// `ShaderStage` (`render_state/shader_stage.rs`'s own doc comment) lists as
/// having "no reader yet" — this fn is oracle's one real reader of all of
/// them, but the wave contract restricts this packet to `tr_shade.rs` only
/// (same restriction `R_BindAnimatedImage`'s doc comment above cites for
/// `skin_num`), so they cannot be added here. Every write target and most
/// read sources are the dissolved `tess` (R2 `## State ownership` row
/// `tess`, no R3 carrier ever): `tess.svars.colors` (the sole output
/// buffer), `tess.numVertexes` (every loop bound, including the disintegrate/
/// volumetric short-circuit's), `tess.vertexColors`, `tess.xyz`,
/// `tess.vertexAlphas`, `tess.fogNum`, `tess.shader` (both the
/// `!= tr.projectionShadowShader/shadowShader` guard and `->portalRange` in
/// `AGEN_PORTAL`). `tr.world->fogs` needs a `WorldAsset::fogs` field not
/// landed by any prior wave (`RB_FogPass`'s doc comment above: only `name`/
/// `shaders`/`bmodels`/`planes`/`nodes`/`mark_surfaces`/light-grid/`vis`/
/// `novis`/entity-string are real). `tr.shadowShader`/`projectionShadowShader`
/// have no `RenderAssets` field landed (`RB_EndSurface`'s doc comment above:
/// only the tier-2 `tr_globals_t::shadowShader` raw pointer exists,
/// scaffolding this wave may not extend). `tr.identityLight`/
/// `identityLightByte` and `styleColors` (`LightStyleTable::colors`) *are*
/// real (`FrameState::identity_light`/`identity_light_byte`,
/// `LightStyleTable` — landed, threaded as parameters below), and
/// `backEnd.currentEntity->e.renderfx`/`.shaderRGBA` are real
/// (`RefEntity::renderfx`/`shader_rgba`), but every branch that would read
/// them writes into the same missing `tess.svars.colors`/`tess.numVertexes`,
/// so nothing downstream of them is reachable either. The already-ported
/// wave-0..4 in-module callees this fn would otherwise call
/// (`RB_CalcDiffuseColor`/`RB_CalcDiffuseEntityColor`/`RB_CalcWaveColor`/
/// `RB_CalcWaveAlpha`/`RB_CalcSpecularAlpha`/`RB_CalcColorFromEntity`/
/// `RB_CalcColorFromOneMinusEntity`/`RB_CalcAlphaFromEntity`/
/// `RB_CalcAlphaFromOneMinusEntity`/`RB_CalcDisintegrateColors`/
/// `RB_CalcDisintegrateVertDeform`/`RB_CalcModulateColorsByFog`/
/// `RB_CalcModulateAlphasByFog`/`RB_CalcModulateRGBAsByFog`) all take the
/// same dissolved `tess.svars.colors`/`.xyz`/`.vertexColors` buffers (and, for
/// the fog trio, a `fog_t`/`orientationr_t` this fn has no way to select —
/// `tr.world->fogs + tess.fogNum`) as their arguments, so none has a value to
/// call with here. No computation survives once every input above is
/// removed — this includes both switch statements in full, since their
/// dispatch keys are the unlanded `pStage` fields themselves.
///
/// Whole-body deferral: no partial body survives, so this lands as a loud
/// `todo!()` rather than a silent no-op (whole-fn-deferral convention —
/// partial-body fns keep DEFERRED comments instead).
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:1529-1801`
pub fn ComputeColors(
    _frame: &FrameState,
    _assets: &RenderAssets,
    _styles: &LightStyleTable,
    _stage: &ShaderStage,
    _force_rgb_gen: i32,
) {
    todo!("Port ComputeColors — oracle/codemp/renderer/tr_shade.cpp:1529-1801")
}

/// Raven `RB_IterateStagesGeneric` — the fixed-function per-stage draw loop:
/// sets up global fog for the surface's fog volume, then for each of the
/// shader's unfogged passes computes colors/texcoords, resolves the correct
/// GL state/blend/texture-binding path (distortion, vertex-lit, stencil-mask,
/// forced-entity-alpha, or the plain path), and draws.
///
/// DEFERRED: R4 — whole-body deferral, both the outer guard and the loop
/// bound are unavailable, not just individual branches: `tess.fogNum`/
/// `.shader` and the `shaderCommands_t *input` parameter itself are the
/// dissolved `tess` type (R2 `## State ownership` row `tess`: "dissolved
/// into R4's tessellation/vertex-building pipeline ... no single global
/// scratch buffer survives the new topology"; same treatment this packet's
/// sibling fns give a `shaderCommands_t *input` parameter — `DrawTris`'s doc
/// comment above: "no R3 carrier holds `input->xyz`/`numVertexes`/
/// `numIndexes`/`indexes` to read from") — so `input->shader->
/// numUnfoggedPasses`, the `for` loop's own bound, has no value to iterate
/// even though `ShaderAsset::num_unfogged_passes` itself has landed, because
/// there is no way to reach a `ShaderAsset` through the unavailable `input`.
/// `tess.xstages[stage]` (`pStage`) is the same dissolved receiver, and even
/// were it reachable, `ShaderStage` (`render_state::shader_stage`'s own doc
/// comment) lists only `image`/`state_bits`/`active` as landed — `ss`/
/// `mGLFogColorOverride`/`glow`/`bundle[0].isLightmap`/`bundle[0]
/// .vertexLightmap`/`bundle[1].isLightmap`/`bundle[1].image` all have no
/// reader yet. `tr.world->fogs`/`globalFog`/`numfogs` need a
/// `WorldAsset::fogs` field not landed by any prior wave (`RB_FogPass`'s doc
/// comment above: only `name`/`shaders`/`bmodels`/`planes`/`nodes`/
/// `mark_surfaces`/light-grid/`vis`/`novis`/entity-string are real).
/// `tr.rangedFog`/`tr.distanceCull` — `RenderAssets` carries
/// `distance_cull`/`distance_cull_squared` but no `ranged_fog` field.
/// `tr.distortionShader`/`tr.screenImage` have no `RenderAssets` field landed
/// (the registry survey lists `default_image`/`fog_image`/`dlight_image`/
/// `white_image` only). `r_drawfog`/`r_lightmap`/`r_uiFullScreen`/
/// `r_vertexLight` need a live cvar-value read — `RendererCvars` holds only
/// `Option<CvarHandle>`, the value-read seam is unwired (same DEFERRED
/// reason as `RB_DrawBuffer`'s `r_clear` dependency, DEC-37 A13.1).
/// `g_bRenderGlowingObjects` is homed now
/// (`FrameState::render_glowing_objects`, campaign #41 batch 1, DEC-37
/// A13.3), but every touch of it here is a read that only gates GL calls or
/// feeds the unreachable `tess`/`pStage` logic above, so nothing lands from
/// it. `GLFogOverrideColors`/`logtestExp2`/`setArraysOnce` are this packet's
/// `STATE HOMES` "per-subsystem owned state struct, NAMED BY THIS WAVE if
/// this file's wave is where the subsystem lands" rows — same read-only
/// situation, so they stay unmapped rather than invented; `tr_stencilled` (write) is the fn-scope-adjacent
/// `lStencilled: bool` static's cross-frame twin — a kind-3 escalation per
/// the three-kind rule, but with no reachable write site here either, so it
/// stays unmapped rather than invented. `backEnd.currentEntity` itself is
/// landed (`FrameState::current_entity: Option<RefEntity>`, and
/// `RefEntity::renderfx`/`shader_rgba` are real per `ComputeColors`'s doc
/// comment above), but every branch that reads it
/// (`RF_DISINTEGRATE1`/`RF_RGB_TINT`/`RF_DISTORTION`/`RF_FORCE_ENT_ALPHA`)
/// only ever feeds `stateBits`/`forceRGBGen` into the unreachable
/// `ComputeColors`/`GL_State` calls below, so it has no independent
/// observable effect. Every in-module callee this fn would otherwise call is
/// itself unavailable: `ComputeColors`/`ComputeTexCoords` are whole-fn
/// `todo!()` stubs on this same file (their doc comments above: no
/// computation survives once `tess`/`pStage` are removed); `DrawMultitextured`/
/// `R_DrawElements`/`ForceAlpha` are DEFERRED-R4 no-ops for the identical
/// dissolved-`tess`/unwired-cvar reasons; `R_BindAnimatedImage` is live only
/// for its video-map branch, unreachable here since `pStage->bundle[0]` is
/// itself unavailable; `GL_Bind`/`GL_Cull`/`GL_State` are GL entry points
/// (DEC-37 A13.2). No computation survives once every input above is
/// removed — this includes the entire per-stage dispatch (distortion /
/// vertex-lit-lightmap / stencil-mask / forced-entity-alpha / plain paths),
/// since its sole inputs are the dissolved `tess`/`input` and the un-landed
/// `pStage`/cvar/registry fields.
///
/// Whole-body deferral: no partial body survives, so this lands as a loud
/// `todo!()` rather than a silent no-op (whole-fn-deferral convention —
/// partial-body fns keep DEFERRED comments instead).
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:1953-2231`
pub fn RB_IterateStagesGeneric(_gpu: &mut GpuResources) {
    todo!("Port RB_IterateStagesGeneric — oracle/codemp/renderer/tr_shade.cpp:1953-2231")
}

/// Raven `RB_StageIteratorGeneric` — the fixed-function stage-iterator entry
/// point for one tessellated surface: deforms the geometry, logs the call,
/// sets face culling and polygon offset, decides (and records into
/// `setArraysOnce`) whether the color/texcoord client arrays can be enabled
/// once for the whole surface or must be re-enabled per pass, locks the
/// vertex array, dispatches to `RB_IterateStagesGeneric`, projects dynamic
/// lights, draws a fog pass, draws surface sprites, then tears the GL state
/// back down.
///
/// DEFERRED: R4 — whole-body deferral, every input is unavailable at this R3
/// wave: `input = &tess` and every subsequent `input->shader->*`/
/// `tess.*` read (`shader->cullType`/`polygonOffset`/`multitextureEnv`/
/// `fogPass`, `tess.numPasses`/`svars.colors`/`svars.texcoords`/`xyz`/
/// `numVertexes`/`dlightBits`/`fogNum`/`shader->sort`/`shader->surfaceFlags`)
/// are the dissolved `tess`/`shaderCommands_t` (R2 `## State ownership` row
/// `tess`: "dissolved into R4's tessellation/vertex-building pipeline ... no
/// single global scratch buffer survives the new topology"; same treatment
/// this file's sibling fns give a `shaderCommands_t *input` parameter —
/// `DrawTris`'s doc comment above) — so even the function's very first
/// statement has no value to read, and every downstream branch condition
/// depends on it. `r_logFile`/`r_dlightStyle`/`r_drawfog`/`r_offsetFactor`/
/// `r_offsetUnits`/`r_surfaceSprites` do have real carriers —
/// `common.cvar(cvars.r_x).integer`/`.value` is this crate's established
/// idiom (~100 call sites) and all six handles are live fields on
/// `RendererCvars` (`render_state/renderer_cvars.rs:104,112,172,200,228,246`)
/// — and `tr.world->globalFog`/`.numfogs` likewise have real carriers now
/// (`WorldAsset::global_fog`/`::fogs`, `render_state/placeholders.rs:242-250`).
/// Neither shortens the deferral: every one of those reads is reached only
/// downstream of `input = &tess`, this fn's very first statement, so the fn
/// can still never get past its own opening line before any of them would be
/// consulted. `setArraysOnce` is this packet's `STATE
/// HOMES` row "per-subsystem owned state struct, NAMED BY THIS WAVE if this
/// file's wave is where the subsystem lands" (DEC-37 A13.3) — this fn is that
/// write site, but the value written is `tess.numPasses > 1 ||
/// input->shader->multitextureEnv`, itself unavailable via the dissolved
/// `input`, so naming a carrier here still couldn't be assigned a correct
/// value; it stays unmapped rather than invented with a placeholder. Every
/// in-module callee is itself unreachable with real arguments: the wave-8
/// `RB_DeformTessGeometry` port takes explicit `xyz`/`normal`/`tex_coords0`/
/// `indexes`/`vertex_colors`/... parameters sourced from `tess` fields this
/// fn has no carrier for and no parameter list to receive them through;
/// `RB_IterateStagesGeneric` is itself a whole-fn `todo!()` stub on this same
/// file; `ProjectDlightTexture`/`ProjectDlightTexture2`/`RB_FogPass` are
/// DEFERRED-R4 no-ops needing the same unavailable `tess`/cvar inputs to even
/// select which one to call; `RB_DrawSurfaceSprites`'s wave-2 port takes
/// `&shaderStage_t`/`&mut CQuickSpriteSystem`/`&mut SurfaceSpriteState`/...
/// sourced from `tess.xstages[stage]`, the same dissolved receiver;
/// `GL_Cull`'s wave-0 port takes a `cullType_t` sourced from
/// `input->shader->cullType`, unavailable for the same reason. Every terminal
/// action past the unavailable inputs is a GL call
/// (`qglColorPointer`/`qglDisable`/`qglDisableClientState`/`qglEnable`/
/// `qglEnableClientState`/`qglLockArraysEXT`/`qglPolygonOffset`/
/// `qglTexCoordPointer`/`qglUnlockArraysEXT`/`qglVertexPointer`, DEC-37
/// A13.2). No computation survives once every input above is removed — this
/// includes the entire face-culling/polygon-offset/array-mode/dlight/fog/
/// surface-sprite orchestration, since its sole inputs are the dissolved
/// `tess`/`input` and the un-landed cvar-value/`WorldAsset` fields.
///
/// Whole-body deferral: no partial body survives, so this lands as a loud
/// `todo!()` rather than a silent no-op (whole-fn-deferral convention —
/// partial-body fns keep DEFERRED comments instead).
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:2237-2385`
pub fn RB_StageIteratorGeneric(_gpu: &mut GpuResources) {
    todo!("Port RB_StageIteratorGeneric — oracle/codemp/renderer/tr_shade.cpp:2237-2385")
}
