//! Raven `tr_scene.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_scene.cpp`

// Raven-named functions/types keep their original casing across this
// transcription, matching the rest of the renderer/engine crates.
#![allow(non_snake_case)]

use mp_engine_qcommon::common::{com_error, com_printf, Common};
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::qhandle_t;

use crate::render_state::frame_data::FrameData;
use crate::render_state::frame_event::FrameEvent;
use crate::render_state::placeholders::{PolyVert, RefEntity, Vec3};
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::shader_asset::ShaderHandle;
use crate::tr_local::decal_poly_s::decalPoly_t;

// This wave threads `RenderAssets`, `FrameData`/`FrameEvent` and `Common`
// (for `com_printf`) as the fns below expect them, per this packet's STATE
// HOMES rows. `RenderAssets::registered`/`max_polys`/`max_polyverts` are
// already real fields (not skeleton placeholders) — no field-merge note
// needed for them.
//
// `RefEntity` (`crate::render_state::placeholders`) is owned by this wave
// (`placeholders.rs`'s "fields land with the tr_scene R3 wave" note) —
// extending `tr_light.rs`'s already-declared subset (`renderfx: i32`,
// `origin`/`lighting_origin`/`ambient_light`/`directed_light`/
// `light_dir: vec3_t`) with the fields `RE_AddRefEntityToScene` needs:
// `re_type: refEntityType_t`; `h_model`/`custom_shader: qhandle_t` — kept as
// the oracle's raw index, not `Handle<ModelAsset>`/`Handle<ShaderAsset>`
// (`tr_font.rs`'s `CFontInfo::mShader` precedent: zero means "unset" here,
// but `Handle{0,0}` means the registry's *live default* entry (A12), and
// reconciling the two conventions is a design decision this packet doesn't
// make); `has_ghoul2: bool` (replaces the raw `*mut c_void ghoul2` pointer
// per the interior-safety law — a presence flag, not the pointer itself);
// `lighting_calculated: bool` (`trRefEntity_t.lightingCalculated`).
// `RefEntity` also needs `#[derive(Default)]` alongside its existing
// `Clone` so this wave's constructing fn and other waves' field-touching
// fns can each set only the subset they own.
//
// Fog-volume assignment (`RE_AddPolyToScene`, oracle lines 151-179) stays
// deferred at the trap: `FrameEvent::AddPolyToScene` grows the `fog_index`
// field the oracle stores, but the search itself needs `WorldAsset::fogs`,
// which the `tr_bsp` fog wave lands (DEC-37 A1 trap-time-validation). See the
// DEFERRED marker at the append site.
//
// `Com_Memcpy`'s poly-vertex copy (oracle line 145) becomes a plain slice
// `.to_vec()` — the translation dictionary's `memcpy` -> owned-slice-copy
// idiom — rather than calling the raw-pointer `Com_Memcpy` (interior code
// stays unsafe-free per porting-rules §D11).

/// Per-subsystem owned state for `tr_scene.cpp` (DEC-37 A13.3 — named by
/// this wave).
///
/// Homes Raven's `refEntParent` and the decal-polygon pool
/// (`re_decalPolys`/`re_decalPolyHead`/`re_decalPolyTotal`) — file-scope
/// statics with no `## State ownership` row of their own.
///
/// Source: `oracle/codemp/renderer/tr_scene.cpp` (file-scope statics
/// referenced by `R_ToggleSmpFrame`/`RE_ClearScene`/`RE_AddRefEntityToScene`/
/// `RE_ClearDecals`)
pub struct SceneState {
    /// Raven `refEntParent` — parent-entity index for the mini-refentity
    /// chain. Every live write sets it to "no parent": the chain-building
    /// branch that would set a real parent index is dead C, wrapped inside a
    /// `/* … */` comment in `RE_AddRefEntityToScene` (DEC-37 ruling 13) —
    /// only the unconditional `refEntParent = -1;` survives as live code.
    pub ref_ent_parent: Option<u32>,
    /// Raven `re_decalPolys[MAX_DECAL_POLYS]` — the decal-polygon pool.
    ///
    /// PORT-NOTE: exact capacity (`MAX_DECAL_POLYS`) is not in this packet's
    /// resolved call surface; `Vec<decalPoly_t>` replaces the fixed C array
    /// under the owned-collection translation, `RE_ClearDecals`'s
    /// memset-to-zero becoming `Vec::clear()` (consumers reindex against
    /// `.len()` once the decal-add path lands in a later wave).
    pub decal_polys: Vec<decalPoly_t>,
    /// Raven `re_decalPolyHead[...]` — decal hash-bucket head list. Same
    /// PORT-NOTE as `decal_polys`; exact element type/size unresolved here.
    pub decal_poly_head: Vec<i32>,
    /// Raven `re_decalPolyTotal[...]` — decal per-bucket running total.
    /// Same PORT-NOTE as `decal_polys`.
    pub decal_poly_total: Vec<i32>,
}

impl Default for SceneState {
    fn default() -> Self {
        Self {
            ref_ent_parent: None,
            decal_polys: Vec::new(),
            decal_poly_head: Vec::new(),
            decal_poly_total: Vec::new(),
        }
    }
}

/// Raven `MAX_DLIGHTS` — the per-frame dynamic-light append bound.
///
/// Source: `oracle/codemp/renderer/tr_local.h:2266`
const MAX_DLIGHTS: usize = 32;

/// Raven `MAX_ENTITIES` — the per-frame ref-entity append bound.
///
/// Source: `oracle/codemp/cgame/tr_types.h:15`
const MAX_ENTITIES: usize = 2048;

/// Raven `TR_WORLDENT = MAX_ENTITIES - 1` — the last slot is reserved for
/// the world entity, so `RE_AddRefEntityToScene` checks against this, not
/// `MAX_ENTITIES` itself.
///
/// Source: `oracle/codemp/cgame/tr_types.h:15`
const TR_WORLDENT: usize = MAX_ENTITIES - 1;

/// Current per-frame ref-entity count — a derived property of the
/// `FrameData` under construction (`### FrameData`'s append-validation
/// principle), never a dedicated field.
fn frame_entity_count(frame: &FrameData) -> usize {
    frame
        .events
        .iter()
        .filter(|event| matches!(event, FrameEvent::AddRefEntityToScene(_)))
        .count()
}

/// Current per-frame dynamic-light count — same derivation principle as
/// `frame_entity_count`; both `AddLightToScene` and `AddAdditiveLightToScene`
/// share the oracle's single `r_numdlights` counter.
fn frame_dlight_count(frame: &FrameData) -> usize {
    frame
        .events
        .iter()
        .filter(|event| {
            matches!(
                event,
                FrameEvent::AddLightToScene { .. } | FrameEvent::AddAdditiveLightToScene { .. }
            )
        })
        .count()
}

/// Current per-frame `(numPolys, numPolyverts)` — same derivation principle;
/// counts only `AddPolyToScene` events (the sibling `AddPolysToScene`/
/// `AddDecalToScene` paths belong to their own, not-yet-ported oracle fns —
/// a later wave sharing this bound extends the scan).
fn frame_poly_counts(frame: &FrameData) -> (usize, usize) {
    frame
        .events
        .iter()
        .fold((0, 0), |(polys, verts), event| match event {
            FrameEvent::AddPolyToScene { verts: v, .. } => (polys + 1, verts + v.len()),
            _ => (polys, verts),
        })
}

/// Raven `R_ToggleSmpFrame` — starts a new frame's event stream.
///
/// Raven's per-frame append counters (`r_numentities`/`r_numdlights`/
/// `r_numpolys`/`r_numpolyverts`) and per-scene offsets (`r_firstScene*`)
/// are not fields under R2 — they are derived properties of the `FrameData`
/// currently under construction (never `FrameState`, never a dedicated
/// field): resetting `frame.events` to empty resets them all in one step.
/// The dead mini-refentity-chain counters (`r_numminientities`/
/// `r_firstSceneMiniEntity`, DEC-37 ruling 13) need no Rust counterpart.
///
/// Source: `oracle/codemp/renderer/tr_scene.cpp:44-65`
pub fn R_ToggleSmpFrame(frame: &mut FrameData, scene: &mut SceneState) {
    frame.events.clear();
    scene.ref_ent_parent = None;
}

/// Raven `RE_ClearScene` — records this scene's boundary in the frame's
/// event stream and resets the (dead) entity-chain parent state.
///
/// The oracle's per-scene offsets (`r_firstSceneDlight`/`Entity`/`Poly`/
/// `MiniEntity` — the last dead per DEC-37 ruling 13) are not fields under
/// R2: pushing `FrameEvent::ClearScene` marks the boundary directly in the
/// stream the `RE_RenderScene` R4 wave later scans, replacing "record
/// `r_firstSceneX = r_numX`" with "mark the point in the stream itself".
///
/// Source: `oracle/codemp/renderer/tr_scene.cpp:74-80`
pub fn RE_ClearScene(frame: &mut FrameData, scene: &mut SceneState) {
    frame.events.push(FrameEvent::ClearScene);
    scene.ref_ent_parent = None;
}

/// Raven `RE_AddPolyToScene`.
///
/// `hshader`/`verts`/`num_verts`/`num_polys` mirror the oracle's
/// `hShader`/`verts`/`numVerts`/`numPolys` params exactly, except `hshader`
/// stays the raw `qhandle_t` (not `ShaderHandle`) until the null check below
/// excludes zero — only then does it convert, since `Handle{0,0}` is a valid
/// default-shader handle under A12, not this "no shader" sentinel.
///
/// Source: `oracle/codemp/renderer/tr_scene.cpp:117-182`
pub fn RE_AddPolyToScene(
    frame: &mut FrameData,
    assets: &RenderAssets,
    common: &mut Common,
    hshader: qhandle_t,
    verts: &[PolyVert],
    num_verts: usize,
    num_polys: usize,
) {
    if !assets.registered {
        return;
    }

    if hshader == 0 {
        // S_COLOR_YELLOW ("^3"), `mp_qshared::shared::q_color::S_COLOR_YELLOW`.
        com_printf(common, "^3WARNING: RE_AddPolyToScene: NULL poly shader\n");
        return;
    }
    let shader = ShaderHandle::new(hshader as u32, 0);

    let (mut num_polys_so_far, mut num_polyverts_so_far) = frame_poly_counts(frame);
    for j in 0..num_polys {
        if num_polyverts_so_far + num_verts > assets.max_polyverts
            || num_polys_so_far >= assets.max_polys
        {
            com_printf(
                common,
                "^3WARNING: RE_AddPolyToScene: r_max_polys or r_max_polyverts reached\n",
            );
            return;
        }

        let start = num_verts * j;
        let poly_verts = verts[start..start + num_verts].to_vec();
        // DEFERRED: RE_AddPolyToScene fogIndex search — WorldAsset.fogs
        // placeholder empty until the tr_bsp fog wave (DEC-37 A1
        // trap-time-validation). The event carries Raven's no-world/single-fog
        // answer, 0, until then.
        // Source: oracle/codemp/renderer/tr_scene.cpp:151-179
        frame.events.push(FrameEvent::AddPolyToScene {
            shader,
            verts: poly_verts,
            fog_index: 0,
        });

        num_polys_so_far += 1;
        num_polyverts_so_far += num_verts;
    }
}

/// Raven `RE_AddRefEntityToScene`.
///
/// Source: `oracle/codemp/renderer/tr_scene.cpp:194-255`
pub fn RE_AddRefEntityToScene(
    frame: &mut FrameData,
    assets: &RenderAssets,
    scene: &mut SceneState,
    ent: &refEntity_t,
) {
    if !assets.registered {
        return;
    }

    // Raven `assert(!ent || ent->renderfx >= 0)` — the `!ent` null-tolerance
    // is moot under Rust's non-null `&refEntity_t` (porting-rules §19: the
    // oracle never exercises that arm either, since every statement past it
    // dereferences `ent` unconditionally).
    debug_assert!(ent.renderfx >= 0);

    if ent.reType == refEntityType_t::RT_ENT_CHAIN {
        // minirefents must die.
        return;
    }

    // `#ifdef _DEBUG` — kept as `debug_assert!`, a no-op in release exactly
    // like the oracle's own debug-only build.
    if ent.reType == refEntityType_t::RT_MODEL {
        debug_assert!(ent.hModel != 0 || !ent.ghoul2.is_null() || ent.customShader != 0);
    }

    if frame_entity_count(frame) >= TR_WORLDENT {
        // `#ifndef FINAL_BUILD` — retail compiles this print out entirely
        // (DEC-37 A13.5, `tr_ghoul2.rs`'s precedent); dropped, not ported.
        return;
    }

    let re_type_value = ent.reType as i32;
    if re_type_value < 0 || re_type_value >= refEntityType_t::RT_MAX_REF_ENTITY_TYPE as i32 {
        com_error(
            errorParm_t::ERR_DROP,
            format!("RE_AddRefEntityToScene: bad reType {}", re_type_value),
        );
    }

    // Raven's `backEndData->entities[r_numentities].e = *ent;` is a whole-struct
    // copy: every `RefEntity` field that flattens a `refEntity_t` member comes
    // from `ent`. The three `trRefEntity_t` lighting outputs
    // (`light_dir`/`ambient_light`/`directed_light`) have no `.e` counterpart —
    // Raven leaves last frame's values in place and `lightingCalculated =
    // qfalse` forces `R_SetupEntityLighting` to overwrite them before any read.
    let re = RefEntity {
        re_type: ent.reType,
        renderfx: ent.renderfx,
        h_model: ent.hModel,
        origin: ent.origin,
        old_origin: ent.oldorigin,
        custom_shader: ent.customShader,
        shader_rgba: ent.shaderRGBA,
        lighting_origin: ent.lightingOrigin,
        end_time: ent.endTime,
        has_ghoul2: !ent.ghoul2.is_null(),
        lighting_calculated: false,
        light_dir: [0.0; 3],
        ambient_light: [0.0; 3],
        directed_light: [0.0; 3],
    };

    // PORT-NOTE: Raven dereferences `ent->ghoul2` here (`CGhoul2Info_v
    // &ghoul2 = *(CGhoul2Info_v *)ent->ghoul2; if (!ghoul2[0].mModel)
    // Com_Printf(...)`) to print a diagnostic when a live ghoul2 instance has
    // no model loaded.
    // DEFERRED: RE_AddRefEntityToScene ghoul2-model diagnostic — reaching
    // into the ghoul2 instance's internals needs the DEC-35 ghoul2 ownership
    // seam (`mp_engine_ghoul2`), not reachable from `tr_scene`. Diagnostic
    // print only, no state effect.
    // Source: oracle/codemp/renderer/tr_scene.cpp:227-239

    frame.events.push(FrameEvent::AddRefEntityToScene(re));

    // Raven's commented-out mini-refentity-chain branch always falls through
    // to the live `refEntParent = -1;` (the `/* … */` block comments out
    // everything up to and including that assignment's preceding `else {`,
    // DEC-37 ruling 13) — an unconditional reset, not the chain-parent
    // bookkeeping the comment shape suggests.
    scene.ref_ent_parent = None;
}

/// Raven `RE_AddDynamicLightToScene`.
///
/// `additive` mirrors the oracle's `int additive` flag as a `bool`; it
/// selects between the two `FrameEvent` variants the frozen shape splits it
/// into (`AddLightToScene`/`AddAdditiveLightToScene`) rather than riding
/// along as a payload field.
///
/// Source: `oracle/codemp/renderer/tr_scene.cpp:326-345`
pub fn RE_AddDynamicLightToScene(
    frame: &mut FrameData,
    assets: &RenderAssets,
    org: Vec3,
    intensity: f32,
    r: f32,
    g: f32,
    b: f32,
    additive: bool,
) {
    if !assets.registered {
        return;
    }
    if frame_dlight_count(frame) >= MAX_DLIGHTS {
        return;
    }
    if intensity <= 0.0 {
        return;
    }

    let event = if additive {
        FrameEvent::AddAdditiveLightToScene {
            org,
            intensity,
            r,
            g,
            b,
        }
    } else {
        FrameEvent::AddLightToScene {
            org,
            intensity,
            r,
            g,
            b,
        }
    };
    frame.events.push(event);
}

/// Raven `RE_ClearDecals`.
///
/// The oracle's three `memset(..., 0, sizeof(...))` calls zero fixed-size C
/// arrays in place; under the owned-collection translation (`SceneState`'s
/// `decal_polys`/`decal_poly_head`/`decal_poly_total` fields, `Vec`-backed),
/// the equivalent reset is `Vec::clear()`.
///
/// Source: `oracle/codemp/renderer/tr_scene.cpp:397-402`
pub fn RE_ClearDecals(scene: &mut SceneState) {
    scene.decal_polys.clear();
    scene.decal_poly_head.clear();
    scene.decal_poly_total.clear();
}
