//! Raven `tr_mesh.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_mesh.cpp`

use mp_engine_qcommon::common::Common;
use mp_engine_qcommon::qfiles::md3_header_t::md3Header_t;
use mp_qshared::shared::vec3_t;

use crate::render_state::frame_state::FrameState;
use crate::render_state::placeholders::RefEntity;
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::tr_model::render_models::RenderModels;

/// Raven `float ProjectRadius(float r, vec3_t location)` — projects a
/// world-space radius `r` at `location` into a screen-space fraction, for
/// LOD/culling decisions.
// DEFERRED: ProjectRadius — depends on `FrameState::view`/`FrameState::ori`
// (`ViewParms`/`OrientationR`, both empty placeholder structs pending the
// `tr_main` wave's `ori.axis`/`ori.origin`/`view.projectionMatrix` fields)
// — see `crates/mp/renderer/src/render_state/placeholders.rs`, out of this
// file's edit scope. A state home this packet marks mapped-but-not-yet-
// populated is an escalation, not an invention (preamble "state home ...
// ESCALATION").
// Source: `oracle/codemp/renderer/tr_mesh.cpp:12-50`
pub fn project_radius(_r: f32, _location: vec3_t, _frame: &FrameState) -> f32 {
    todo!("Port ProjectRadius — oracle/codemp/renderer/tr_mesh.cpp:12-50")
}

/// Raven `int R_ComputeFogNum(md3Header_t *header, trRefEntity_t *ent)` —
/// the fog volume `ent`'s MD3 frame bounds fall inside, if any.
// DEFERRED: R_ComputeFogNum — depends on `RenderAssets::world` (`WorldAsset`,
// an empty placeholder struct pending the `tr_bsp` wave's `numfogs`/`fogs`
// fields) and `FrameState::refdef` (`TrRefdef`, empty pending the `tr_scene`
// wave's `rdflags` field) — see
// `crates/mp/renderer/src/render_state/placeholders.rs`, out of this file's
// edit scope. Also needs a parsed-frame accessor for `md3Header_t`'s
// `ofsFrames`-relative frame array (the oracle's raw
// `(byte*)header + header->ofsFrames + ent->e.frame` walk) and `ent`'s owned
// `RefEntity` (empty placeholder, same `tr_scene` wave) — no such accessor is
// in this wave's resolved call surface. A state home this packet marks
// mapped-but-not-yet-populated is an escalation, not an invention (preamble
// "state home ... ESCALATION"). Same blocker already hit by the ghoul2
// variant of this fn (`r_g_compute_fog_num`, `crates/mp/renderer/src/
// tr_ghoul2.rs`).
// Source: `oracle/codemp/renderer/tr_mesh.cpp:244-273`
pub fn r_compute_fog_num(
    _header: &md3Header_t,
    _ent: &RefEntity,
    _assets: &RenderAssets,
    _frame: &FrameState,
) -> i32 {
    todo!("Port R_ComputeFogNum — oracle/codemp/renderer/tr_mesh.cpp:244-273")
}

/// Raven `void RE_GetModelBounds(refEntity_t *refEnt, vec3_t bounds1, vec3_t
/// bounds2)` — the MD3 model's per-frame bounding box for `refEnt->hModel`
/// at `refEnt->frame`. Out-params fold into a returned pair (dictionary:
/// out-params→returns).
// DEFERRED: RE_GetModelBounds — the oracle body indexes the on-disk frame
// array by `refEnt->frame` (`(md3Frame_t *)((byte *)header + header->
// ofsFrames) + refEnt->frame`, a walk `md3Frame_t`
// (`mp_engine_qcommon::qfiles::md3_frame_s::md3Frame_t`) already supports and
// tier-2's `model_t.md3: [*mut md3Header_t; 3]` (`RenderModels::get_model`,
// the already-ported `R_GetModelByHandle`) is legal to READ through per the
// interior-safety law — but `refEntity_t.frame` has no landing field on
// `RefEntity` yet (`crate::render_state::placeholders`, owned by the
// `tr_scene` wave, out of this file's edit scope). A state home this packet
// marks mapped-but-not-yet-populated is an escalation, not an invention
// (preamble "state home ... ESCALATION").
// Source: `oracle/codemp/renderer/tr_mesh.cpp:148-165`
pub fn re_get_model_bounds(_ref_ent: &RefEntity, _models: &RenderModels) -> (vec3_t, vec3_t) {
    todo!("Port RE_GetModelBounds — oracle/codemp/renderer/tr_mesh.cpp:148-165")
}

/// Raven `int R_ComputeLOD(trRefEntity_t *ent)` — picks `ent`'s MD3 LOD level
/// from its projected screen-space bounding-sphere radius, biased by
/// `r_lodscale`/`r_autolodscalevalue`/`r_lodbias` and clamped to
/// `tr.currentModel->numLods`.
// DEFERRED: R_ComputeLOD — same `md3Frame_t`/`ofsFrames` walk as
// `RE_GetModelBounds` above (tier-2 `model_t.md3[0]` is legal to READ
// through), plus two fields this wave cannot add (out of this file's edit
// scope): `tr.currentModel` has no landing field on `FrameState` yet (STATE
// HOMES: "frontend scratch/counters/.../currentModel → RenderWorld::frame:
// FrameState" — `crate::render_state::frame_state`, owned by the `tr_main`
// wave per its `## Seam definition` row) and `ent->e.frame` has no landing
// field on `RefEntity` yet (`crate::render_state::placeholders`, owned by
// the `tr_scene` wave — same field `RE_GetModelBounds` needs). A state home
// this packet marks mapped-but-not-yet-populated is an escalation, not an
// invention (preamble "state home ... ESCALATION").
// Source: `oracle/codemp/renderer/tr_mesh.cpp:173-236`
pub fn r_compute_lod(
    _common: &Common,
    _cvars: &RendererCvars,
    _assets: &RenderAssets,
    _frame: &FrameState,
    _ent: &RefEntity,
) -> i32 {
    todo!("Port R_ComputeLOD — oracle/codemp/renderer/tr_mesh.cpp:173-236")
}
