//! Raven `tr_scene.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_scene.cpp`

// Raven-named functions/types keep their original casing across this
// transcription, matching the rest of the renderer/engine crates.
#![allow(non_snake_case)]

use native_math::qmath::{
    _DotProduct, _VectorScale, _VectorSubtract, CrossProduct, PerpendicularVectorMP,
    RotatePointAroundVector, VectorNormalize2,
};

use core::ffi::c_void;

use mp_engine_ghoul2::info_array::Ghoul2Handle;
use mp_engine_qcommon::common::{com_error, com_printf, Common};
use mp_qshared::common::mp::cgame::mini_ref_entity_s::miniRefEntity_t;
use mp_qshared::common::mp::cgame::poly_vert_t::polyVert_t;
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::common::mp::cgame::refdef_t::{
    refdef_t, MAX_MAP_AREA_BYTES, MAX_RENDER_STRINGS, MAX_RENDER_STRING_LENGTH,
};
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::qhandle_t;

use crate::render_state::frame_data::FrameData;
use crate::render_state::frame_event::FrameEvent;
use crate::render_state::frame_state::FrameState;
use crate::render_state::light_style_table::LightStyleTable;
use crate::render_state::placeholders::{PolyVert, RefEntity, TrRefdef, Vec3};
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::render_state::shader_asset::ShaderHandle;
use crate::tr_local::decal_poly_s::{decalPoly_t, MAX_VERTS_ON_DECAL_POLY};
use crate::tr_main::{DrawSurf, R_AddDrawSurf, SurfaceGeometry};
use crate::tr_marks::{MarkNode, R_MarkFragments};
use crate::tr_public::ref_flags::{RDF_DRAWSKYBOX, RDF_NOWORLDMODEL, RDF_SKYBOXPORTAL};
use crate::tr_shader::R_GetShaderByHandle;

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
// the oracle's raw index, not `ModelHandle`/`Handle<ShaderAsset>`
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
    /// Raven `re_decalPolys[NUM_DECAL_POLY_TYPES][MAX_DECAL_POLYS]` — the
    /// decal-polygon pool. Outer index = decal type (`DECALPOLY_TYPE_NORMAL`/
    /// `DECALPOLY_TYPE_FADE`), inner index = per-type slot (wave-1 resolves
    /// the shape the wave-0 PORT-NOTE left open).
    ///
    /// PORT-NOTE: `MAX_DECAL_POLYS` (the C array's fixed capacity) is not in
    /// this packet's resolved call surface either; each per-type pool grows
    /// lazily to the highest index `RE_AllocDecal`/`RE_FreeDecal` touch
    /// (`ensure_decal_pool`), bounded at runtime by `r_markcount->integer` —
    /// the same bound Raven's own head/wraparound logic uses — rather than
    /// being pre-sized. `RE_ClearDecals`'s memset-to-zero is `Vec::clear()`.
    pub decal_polys: Vec<Vec<decalPoly_t>>,
    /// Raven `re_decalPolyHead[NUM_DECAL_POLY_TYPES]` — decal ring-buffer
    /// head, one slot per decal type. Same PORT-NOTE as `decal_polys`.
    pub decal_poly_head: Vec<i32>,
    /// Raven `re_decalPolyTotal[NUM_DECAL_POLY_TYPES]` — decal per-type
    /// running total. Same PORT-NOTE as `decal_polys`.
    pub decal_poly_total: Vec<i32>,
    /// Raven `R_AddDecals`'s `static int lastMarkCount` — cross-frame
    /// `r_markcount` latch driving the one-shot "cvar changed -> clear the
    /// decal pool" reset. Initialised to the oracle's `-1` sentinel ("never
    /// sampled"), which suppresses the clear on the first call.
    ///
    /// Source: `oracle/codemp/renderer/tr_scene.cpp:624`
    pub last_mark_count: i32,
    /// Raven `RE_RenderScene`'s `static int lastTime` — the previous call's
    /// `fd->time`, differenced against the new one to derive `frametime`
    /// (kind-3 fn-scope state, this file's own carrier per the three-kind
    /// rule; DEC-37 A13.3).
    ///
    /// Source: `oracle/codemp/renderer/tr_scene.cpp:709`
    pub last_time: i32,
    /// The trap-side persistent `tr.refdef.areamask` — the previous scene's
    /// area bits. `RE_RenderScene` compares the new `fd->areamask` against
    /// this to set `areamaskModified`, then stores the new bits here. The
    /// render-thread `tr.refdef` is not reachable at trap time, so the diff
    /// state lives beside `last_time`.
    ///
    /// Source: `oracle/codemp/renderer/tr_scene.cpp:770-786`
    pub refdef_areamask: [u8; MAX_MAP_AREA_BYTES],
    /// The oracle's sticky `skyboxportal` file-scope static, write side.
    /// `RE_RenderScene` sets it to 1 when `RDF_SKYBOXPORTAL` is present and
    /// never clears it, so the trap side keeps it across scenes.
    ///
    /// Source: `oracle/codemp/renderer/tr_scene.cpp:35,744-747`
    pub skyboxportal: i32,
    /// The oracle's `drawskyboxportal` file-scope static, write side.
    /// `RE_RenderScene` sets it to 1 or 0 each scene from `RDF_DRAWSKYBOX`.
    ///
    /// Source: `oracle/codemp/renderer/tr_scene.cpp:36,749-756`
    pub drawskyboxportal: i32,
}

impl Default for SceneState {
    fn default() -> Self {
        Self {
            ref_ent_parent: None,
            decal_polys: Vec::new(),
            decal_poly_head: Vec::new(),
            decal_poly_total: Vec::new(),
            last_mark_count: -1,
            last_time: 0,
            refdef_areamask: [0; MAX_MAP_AREA_BYTES],
            skyboxportal: 0,
            drawskyboxportal: 0,
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

/// Decodes the tier-1 `refEntity_t.ghoul2` pointer field into a
/// [`Ghoul2Handle`].
///
/// The engine owns every Ghoul2 instance and hands cgame an opaque token in the
/// `refEntity_t.ghoul2` pointer field. This repo encodes the `Ghoul2System`
/// instance handle as `handle + 1` cast to pointer width, so a null pointer
/// reads as no instance (`None`). The render side threads a `&mut Ghoul2System`
/// and looks the list up by the handle, so the raw pointer never crosses into
/// safe code. [`ghoul2_token_encode`] is the inverse.
///
/// RECONCILE (DEC-51): the live server ghoul2 seam does not yet produce this
/// token. `sv_game.rs:3116-3125` hands a module a raw `Box<CGhoul2Info_v>`
/// pointer in the `void*` ghoul2 slot (freed at `G_G2_CLEANMODELS`,
/// `:3347-3358`), and cgame copies that raw pointer into `refEntity_t.ghoul2`.
/// A raw pointer decoded here as `ptr - 1` yields a garbage handle. Today only
/// the render harness fills the field, through [`ghoul2_token_encode`], so
/// nothing is wrong yet. When the real cgame path lands, the raw-pointer seam
/// and this token convention must be reconciled to one scheme.
pub fn ghoul2_token_decode(token: *mut c_void) -> Option<Ghoul2Handle> {
    if token.is_null() {
        None
    } else {
        Some(Ghoul2Handle(token as i32 - 1))
    }
}

/// Encodes a [`Ghoul2Handle`] back into the tier-1 `refEntity_t.ghoul2` pointer
/// token. The inverse of [`ghoul2_token_decode`]; `None` encodes as null.
pub fn ghoul2_token_encode(handle: Option<Ghoul2Handle>) -> *mut c_void {
    match handle {
        Some(h) => (h.0 + 1) as *mut c_void,
        None => core::ptr::null_mut(),
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
    // (`light_dir`/`ambient_light`/`ambient_light_int`/`directed_light`, plus
    // `need_dlights`/`dlight_bits`) have no `.e` counterpart — Raven leaves
    // last frame's values in place and `lightingCalculated = qfalse` forces
    // `R_SetupEntityLighting` to overwrite them before any read.
    let re = RefEntity {
        re_type: ent.reType,
        renderfx: ent.renderfx,
        h_model: ent.hModel,
        axis: ent.axis,
        non_normalized_axes: ent.nonNormalizedAxes != 0,
        origin: ent.origin,
        old_origin: ent.oldorigin,
        custom_shader: ent.customShader,
        shader_rgba: ent.shaderRGBA,
        radius: ent.radius,
        rotation: ent.rotation,
        shader_time: ent.shaderTime,
        frame: ent.frame,
        old_frame: ent.oldframe,
        backlerp: ent.backlerp,
        skin_num: ent.skinNum,
        custom_skin: ent.customSkin,
        lighting_origin: ent.lightingOrigin,
        end_time: ent.endTime,
        saber_length: ent.saberLength,
        angles: ent.angles,
        model_scale: ent.modelScale,
        ghoul2: ghoul2_token_decode(ent.ghoul2),
        need_dlights: false,
        lighting_calculated: false,
        light_dir: [0.0; 3],
        ambient_light: [0.0; 3],
        ambient_light_int: [0; 4],
        directed_light: [0.0; 3],
        dlight_bits: 0,
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

// ---------------------------------------------------------------------
// wave 1
// ---------------------------------------------------------------------

/// Raven `QSORT_ENTITYNUM_SHIFT` — restated from `tr_main.rs`'s own local
/// `const` (not `pub` there, so not reachable from this file). Same value,
/// same bit-packing rationale as `tr_main.rs`'s `R_AddDrawSurf`/
/// `R_DecomposeSort` PORT-NOTE (dlight 2 bits + fog 5 bits => entity's
/// bit-7 start; DEC-37 ruling 1: the sort key is pure renderer interior, only
/// its resulting relative order is observable) — not re-derived here.
///
/// Source: `oracle/codemp/renderer/tr_local.h:1226-1228`
const QSORT_ENTITYNUM_SHIFT: u32 = 7;

/// Raven `R_AddPolygonSurfaces`.
///
/// `tr.currentEntityNum`/`tr.shiftedEntityNum` are recomputed to the same
/// value (`TR_WORLDENT`/`TR_WORLDENT << QSORT_ENTITYNUM_SHIFT`) on every call
/// and consumed immediately by `R_AddDrawSurf` below. This packet's STATE
/// HOMES row assigns the persistent write to `RenderWorld::frame: FrameState`,
/// but this wave is scoped to `tr_scene.rs` only (cannot add a field to
/// `render_state/frame_state.rs`), so the write stays a local computation —
/// escalate a `FrameState` field-merge if a later wave needs to read either
/// value back outside this call.
///
/// `tr.refdef.polys` is this frame's already-appended
/// `FrameEvent::AddPolyToScene` events — matching this file's own
/// `frame_poly_counts`/`frame_entity_count` precedent of treating `FrameData`
/// itself as the render-thread's "current refdef" carrier until `TrRefdef`
/// grows a dedicated `polys` field.
///
/// Source: `oracle/codemp/renderer/tr_scene.cpp:97-109`
pub fn R_AddPolygonSurfaces<'a>(
    frame: &'a FrameData,
    assets: &RenderAssets,
    common: &mut Common,
    draw_surfs: &mut Vec<DrawSurf<SurfaceGeometry<'a>>>,
) {
    let current_entity_num = TR_WORLDENT as i32;
    let shifted_entity_num = current_entity_num << QSORT_ENTITYNUM_SHIFT;

    for event in &frame.events {
        if let FrameEvent::AddPolyToScene {
            shader,
            verts,
            fog_index,
        } = event
        {
            let sh = R_GetShaderByHandle(assets, common, shader.index() as i32);
            let shader_sorted_index = assets.shaders.get(sh).map(|s| s.sorted_index).unwrap_or(0);

            // DEFERRED: rdf_nofog (`tr.refdef.rdflags & RDF_NOFOG`) — `TrRefdef`
            // has no `rdflags` field yet (lands with whichever wave gives it
            // its full shape). `fog_index` is already forced to 0 by
            // `RE_AddPolyToScene`'s own DEFERRED fog-search note (above), so
            // `rdf_nofog`'s value is a no-op either way until both land.
            // Source: oracle/codemp/renderer/tr_main.cpp:1266
            R_AddDrawSurf(
                SurfaceGeometry::Poly {
                    verts: verts.as_slice(),
                },
                shader_sorted_index,
                shifted_entity_num,
                false,
                *fog_index,
                0,
                draw_surfs,
            );
        }
    }
}

/// Raven `RE_AddMiniRefEntityToScene`.
///
/// Only the oracle's live `#if 1` branch is transcribed — the `#else` branch
/// (chain-parent bookkeeping against `backEndData->miniEntities`) is dead
/// source, guarded out at compile time in the oracle itself (DEC-37 ruling
/// 13: the real mini-refentity chain is `#if 0`) and dropped per §19's
/// dead-surface rule rather than ported.
///
/// `ent` is `Option<&miniRefEntity_t>` — unlike `RE_AddRefEntityToScene`'s
/// moot `!ent` check, this fn's null branch is genuinely live (sets
/// `refEntParent = -1` and returns before touching `ent`).
///
/// The C body's `memcpy(&tempEnt, ent, sizeof(*ent));
/// memset(...+sizeof(*ent), 0, ...)` becomes an explicit field-by-field copy
/// from `miniRefEntity_t` onto a zeroed `refEntity_t` — the two types share
/// an identical field-for-field prefix layout (both structs' own doc
/// comments: "this structure must remain identical as the miniRefEntity_t"),
/// so copying exactly `miniRefEntity_t`'s 14 named fields onto a
/// `refEntity_t::zeroed()` reproduces the memcpy+memset pair without a raw
/// byte copy (interior-safety law).
///
/// Source: `oracle/codemp/renderer/tr_scene.cpp:271-317`
pub fn RE_AddMiniRefEntityToScene(
    frame: &mut FrameData,
    assets: &RenderAssets,
    scene: &mut SceneState,
    ent: Option<&miniRefEntity_t>,
) {
    if !assets.registered {
        return;
    }

    let ent = match ent {
        Some(ent) => ent,
        None => {
            scene.ref_ent_parent = None;
            return;
        }
    };

    let mut temp_ent = refEntity_t::zeroed();
    temp_ent.reType = ent.reType;
    temp_ent.renderfx = ent.renderfx;
    temp_ent.hModel = ent.hModel;
    temp_ent.axis = ent.axis;
    temp_ent.nonNormalizedAxes = ent.nonNormalizedAxes;
    temp_ent.origin = ent.origin;
    temp_ent.oldorigin = ent.oldorigin;
    temp_ent.customShader = ent.customShader;
    temp_ent.shaderRGBA = ent.shaderRGBA;
    temp_ent.shaderTexCoord = ent.shaderTexCoord;
    temp_ent.radius = ent.radius;
    temp_ent.rotation = ent.rotation;
    temp_ent.shaderTime = ent.shaderTime;
    temp_ent.frame = ent.frame;

    RE_AddRefEntityToScene(frame, assets, scene, &temp_ent);
}

/// Raven `RE_AddLightToScene`.
///
/// Source: `oracle/codemp/renderer/tr_scene.cpp:355-357`
pub fn RE_AddLightToScene(
    frame: &mut FrameData,
    assets: &RenderAssets,
    org: Vec3,
    intensity: f32,
    r: f32,
    g: f32,
    b: f32,
) {
    RE_AddDynamicLightToScene(frame, assets, org, intensity, r, g, b, false);
}

/// Raven `RE_AddAdditiveLightToScene`.
///
/// Source: `oracle/codemp/renderer/tr_scene.cpp:367-369`
pub fn RE_AddAdditiveLightToScene(
    frame: &mut FrameData,
    assets: &RenderAssets,
    org: Vec3,
    intensity: f32,
    r: f32,
    g: f32,
    b: f32,
) {
    RE_AddDynamicLightToScene(frame, assets, org, intensity, r, g, b, true);
}

/// Raven `R_InitDecals`.
///
/// Source: `oracle/codemp/renderer/tr_scene.cpp:404-407`
pub fn R_InitDecals(scene: &mut SceneState) {
    RE_ClearDecals(scene);
}

/// Raven `DECALPOLY_TYPE_NORMAL` — first member of the file-local decal-type
/// enum.
///
/// Source: `oracle/codemp/renderer/tr_scene.cpp:375`
const DECALPOLY_TYPE_NORMAL: i32 = 0;

/// Raven `DECALPOLY_TYPE_FADE` — second member of the file-local decal-type
/// enum.
///
/// Source: `oracle/codemp/renderer/tr_scene.cpp:376`
const DECALPOLY_TYPE_FADE: i32 = 1;

/// Raven `DECAL_FADE_TIME`.
///
/// Source: `oracle/codemp/renderer/tr_scene.cpp:380`
const DECAL_FADE_TIME: i32 = 1000;

/// Ensures `scene.decal_polys[type_]` exists and has at least `len` slots,
/// growing with `decalPoly_t::zeroed()` entries — the owned-`Vec` replacement
/// for Raven's fixed `re_decalPolys[NUM_DECAL_POLY_TYPES][MAX_DECAL_POLYS]`
/// (`decal_polys`'s own doc comment: `MAX_DECAL_POLYS`'s value is not in
/// this packet, so the pool grows to whatever index is touched instead of
/// being pre-sized).
fn ensure_decal_pool(scene: &mut SceneState, type_: usize, len: usize) {
    while scene.decal_polys.len() <= type_ {
        scene.decal_polys.push(Vec::new());
    }
    while scene.decal_polys[type_].len() < len {
        scene.decal_polys[type_].push(decalPoly_t::zeroed());
    }
}

/// Reads `scene.decal_poly_head[type_]`, growing the per-type head list on
/// demand (same lazy-growth shape as `ensure_decal_pool`).
fn decal_head(scene: &mut SceneState, type_: usize) -> i32 {
    while scene.decal_poly_head.len() <= type_ {
        scene.decal_poly_head.push(0);
    }
    scene.decal_poly_head[type_]
}

/// Writes `scene.decal_poly_head[type_]`, growing on demand.
fn set_decal_head(scene: &mut SceneState, type_: usize, value: i32) {
    while scene.decal_poly_head.len() <= type_ {
        scene.decal_poly_head.push(0);
    }
    scene.decal_poly_head[type_] = value;
}

/// Reads `scene.decal_poly_total[type_]`, growing the per-type total list on
/// demand.
fn decal_total(scene: &mut SceneState, type_: usize) -> i32 {
    while scene.decal_poly_total.len() <= type_ {
        scene.decal_poly_total.push(0);
    }
    scene.decal_poly_total[type_]
}

/// Writes `scene.decal_poly_total[type_]`, growing on demand.
fn set_decal_total(scene: &mut SceneState, type_: usize, value: i32) {
    while scene.decal_poly_total.len() <= type_ {
        scene.decal_poly_total.push(0);
    }
    scene.decal_poly_total[type_] = value;
}

/// Raven `RE_FreeDecal`.
///
/// `type_`/`index` mirror the oracle's `type`/`index` params (`type` is a
/// Rust keyword, so `type_`). `refdef_time` stands in for `tr.refdef.time` —
/// `TrRefdef` (this packet's STATE HOMES carrier for `tr`'s frontend scratch)
/// has no `time` field yet and this wave is scoped to `tr_scene.rs` only, so
/// the value is threaded as an explicit parameter instead of extending that
/// struct (escalate a `TrRefdef` field-merge if a later wave needs to read
/// it back from `FrameState` directly). `cvars`/`common` forward to the
/// `RE_AllocDecal` recursive call below (SCC 506).
///
/// Source: `oracle/codemp/renderer/tr_scene.cpp:409-431`
pub fn RE_FreeDecal(
    scene: &mut SceneState,
    cvars: &RendererCvars,
    common: &Common,
    refdef_time: i32,
    type_: i32,
    index: i32,
) {
    ensure_decal_pool(scene, type_ as usize, index as usize + 1);
    if scene.decal_polys[type_ as usize][index as usize].time == 0 {
        return;
    }

    if type_ == DECALPOLY_TYPE_NORMAL {
        // `fade = RE_AllocDecal( DECALPOLY_TYPE_FADE );` — see this fn's
        // return-shape PORT-NOTE on `RE_AllocDecal` for why this is a
        // `(type, index)` location rather than a `decalPoly_t *`.
        let fade = RE_AllocDecal(scene, cvars, common, refdef_time, DECALPOLY_TYPE_FADE);

        // `memcpy ( fade, &re_decalPolys[type][index], sizeof(decalPoly_t) );`
        // — `decalPoly_t` is `Copy`, so an owned read-then-write reproduces
        // the whole-struct copy without a raw byte copy.
        let normal_poly = scene.decal_polys[type_ as usize][index as usize];
        ensure_decal_pool(scene, fade.0, fade.1 + 1);
        scene.decal_polys[fade.0][fade.1] = normal_poly;
        scene.decal_polys[fade.0][fade.1].time = refdef_time;
        scene.decal_polys[fade.0][fade.1].fadetime = refdef_time + DECAL_FADE_TIME;
    }

    ensure_decal_pool(scene, type_ as usize, index as usize + 1);
    scene.decal_polys[type_ as usize][index as usize].time = 0;

    let new_total = decal_total(scene, type_ as usize) - 1;
    set_decal_total(scene, type_ as usize, new_total);
}

/// Raven `RE_AllocDecal`.
///
/// `refdef_time` stands in for `tr.refdef.time` — same STATE HOMES caveat as
/// `RE_FreeDecal`. Returns `(type, index)` into `SceneState::decal_polys`
/// rather than the oracle's `decalPoly_t *`: holding a live `&mut
/// decalPoly_t` across this fn's own `RE_FreeDecal` recursion would alias
/// `scene` twice at once (forbidden in safe Rust) — the interior-safety law's
/// "need to reference another asset -> store its `Handle`" rule applies the
/// same way here, so callers re-index through the returned location instead
/// of holding a live reference.
///
/// Source: `oracle/codemp/renderer/tr_scene.cpp:440-500`
pub fn RE_AllocDecal(
    scene: &mut SceneState,
    cvars: &RendererCvars,
    common: &Common,
    refdef_time: i32,
    type_: i32,
) -> (usize, usize) {
    let r_markcount = common.cvar(cvars.r_markcount).integer;

    // See if the cvar changed
    if decal_total(scene, type_ as usize) > r_markcount {
        RE_ClearDecals(scene);
    }

    let head = decal_head(scene, type_ as usize) as usize;
    ensure_decal_pool(scene, type_ as usize, head + 1);

    // If it has no time its the first occasion its been used
    if scene.decal_polys[type_ as usize][head].time != 0 {
        if scene.decal_polys[type_ as usize][head].time != refdef_time {
            let mut i = head as i32;

            // since we are killing one that existed before, make sure we
            // kill all the other marks that belong to the group
            loop {
                i += 1;
                if i >= r_markcount {
                    i = 0;
                }
                ensure_decal_pool(scene, type_ as usize, i as usize + 1);
                ensure_decal_pool(scene, type_ as usize, head + 1);

                // Break out on the first one thats not part of the group
                // PORT-NOTE: `le->time` and `re_decalPolyHead[type]` are live
                // pointer/array reads in the oracle, not values snapshotted
                // before the loop — `RE_FreeDecal` below re-enters
                // `RE_AllocDecal`, which can run `RE_ClearDecals` and reset
                // both. Each is re-read at its own oracle read point.
                let le_time = scene.decal_polys[type_ as usize][head].time;
                if scene.decal_polys[type_ as usize][i as usize].time != le_time {
                    break;
                }

                RE_FreeDecal(scene, cvars, common, refdef_time, type_, i);

                if i == decal_head(scene, type_ as usize) {
                    break;
                }
            }

            let live_head = decal_head(scene, type_ as usize);
            RE_FreeDecal(scene, cvars, common, refdef_time, type_, live_head);
        } else {
            let live_head = decal_head(scene, type_ as usize);
            RE_FreeDecal(scene, cvars, common, refdef_time, type_, live_head);
        }
    }

    ensure_decal_pool(scene, type_ as usize, head + 1);
    scene.decal_polys[type_ as usize][head] = decalPoly_t::zeroed();
    scene.decal_polys[type_ as usize][head].time = refdef_time;

    let new_total = decal_total(scene, type_ as usize) + 1;
    set_decal_total(scene, type_ as usize, new_total);

    // Move on to the next decal poly and wrap around if need be
    // PORT-NOTE: the increment reads `re_decalPolyHead[type]` live, not the
    // entry snapshot `head` — same reset-by-`RE_ClearDecals` reason as above.
    let mut new_head = decal_head(scene, type_ as usize) + 1;
    if new_head >= r_markcount {
        new_head = 0;
    }
    set_decal_head(scene, type_ as usize, new_head);

    (type_ as usize, head)
}

// ---------------------------------------------------------------------
// wave 2
// ---------------------------------------------------------------------

/// Raven `R_AddDecals`.
///
/// `refdef_time` stands in for `tr.refdef.time` — same STATE HOMES caveat as
/// `RE_FreeDecal`/`RE_AllocDecal` (`TrRefdef` has no `time` field yet and this
/// wave is scoped to `tr_scene.rs` only; escalate a field-merge if a later
/// wave needs it back from `FrameState` directly).
///
/// `type_` walks `DECALPOLY_TYPE_NORMAL..=DECALPOLY_TYPE_FADE` rather than
/// the oracle's `DECALPOLY_TYPE_MAX` sentinel: `DECALPOLY_TYPE_MAX` is not
/// itself in this packet's FILE-SCOPE CONSTANTS section nor this fn's own
/// oracle slice, so its numeric value is never invented — the loop still
/// covers exactly the same two already-ported, cited members
/// (`DECALPOLY_TYPE_NORMAL = 0`, `DECALPOLY_TYPE_FADE = 1`, wave-1) that a
/// sequential C enum's `_MAX` sentinel would bound.
///
/// Source: `oracle/codemp/renderer/tr_scene.cpp:620-690`
pub fn R_AddDecals(
    frame: &mut FrameData,
    assets: &RenderAssets,
    scene: &mut SceneState,
    cvars: &RendererCvars,
    common: &mut Common,
    refdef_time: i32,
) {
    let r_markcount = common.cvar(cvars.r_markcount).integer;

    // `static int lastMarkCount` is homed on `SceneState` (DEC-37 A13.3 — this
    // file's own state carrier), initialised to the oracle's `-1` sentinel.
    // Source: oracle/codemp/renderer/tr_scene.cpp:624,626-634
    if r_markcount != scene.last_mark_count {
        if scene.last_mark_count != -1 {
            RE_ClearDecals(scene);
        }

        scene.last_mark_count = r_markcount;
    }

    if r_markcount <= 0 {
        return;
    }

    for type_ in DECALPOLY_TYPE_NORMAL..=DECALPOLY_TYPE_FADE {
        let mut decal_poly = decal_head(scene, type_ as usize);

        loop {
            ensure_decal_pool(scene, type_ as usize, decal_poly as usize + 1);
            let p = scene.decal_polys[type_ as usize][decal_poly as usize];

            if p.time != 0 {
                if p.fadetime != 0 {
                    // fade all marks out with time
                    let t = refdef_time - p.time;
                    if t < DECAL_FADE_TIME {
                        let fade = 255.0f32 * (1.0 - (t as f32 / DECAL_FADE_TIME as f32));
                        let num_verts = p.poly.numVerts as usize;

                        for j in 0..num_verts {
                            scene.decal_polys[type_ as usize][decal_poly as usize].verts[j]
                                .modulate[3] = fade as u8;
                        }

                        let verts: Vec<PolyVert> = scene.decal_polys[type_ as usize]
                            [decal_poly as usize]
                            .verts[..num_verts]
                            .to_vec();
                        RE_AddPolyToScene(frame, assets, common, p.shader, &verts, num_verts, 1);
                    } else {
                        RE_FreeDecal(scene, cvars, &*common, refdef_time, type_, decal_poly);
                    }
                } else {
                    let num_verts = p.poly.numVerts as usize;
                    let verts: Vec<PolyVert> = p.verts[..num_verts].to_vec();
                    RE_AddPolyToScene(frame, assets, common, p.shader, &verts, num_verts, 1);
                }
            }

            decal_poly += 1;
            if decal_poly >= r_markcount {
                decal_poly = 0;
            }

            if decal_poly == decal_head(scene, type_ as usize) {
                break;
            }
        }
    }
}

// ---------------------------------------------------------------------
// wave 3
// ---------------------------------------------------------------------

/// Raven `MAX_DECAL_FRAGMENTS` — the `R_MarkFragments` fragment-buffer bound
/// `RE_AddDecalToScene` passes through.
///
/// Source: `oracle/codemp/renderer/tr_scene.cpp:514`
const MAX_DECAL_FRAGMENTS: usize = 128;

/// Raven `MAX_DECAL_POINTS` — the `R_MarkFragments` point-buffer bound
/// `RE_AddDecalToScene` passes through.
///
/// Source: `oracle/codemp/renderer/tr_scene.cpp:515`
const MAX_DECAL_POINTS: usize = 384;

/// Raven `RE_AddDecalToScene`.
///
/// `frame`/`assets`/`scene`/`cvars`/`common` mirror this file's established
/// wave-2 `R_AddDecals`/`RE_AllocDecal` threading; `refdef_time` stands in for
/// `tr.refdef.time` for the same STATE HOMES reason those two fns already
/// carry it as an explicit param (`TrRefdef` has no `time` field yet, this
/// wave is scoped to `tr_scene.rs` only). `world_root`/`frame_state` are new
/// to this wave: they are the two extra params `tr_marks`' landed (idiomatic,
/// not the oracle's raw C shape) `R_MarkFragments` signature requires
/// (`MarkNode` BSP-walk root and `FrameState::view_count`) — this fn is
/// `R_MarkFragments`'s only caller so far, so they thread straight through
/// rather than being invented state on this file's own carrier.
///
/// `alphaFade` (`_alpha_fade` — unread past the parameter list, same as the
/// oracle body: no `decalPoly_t` field it could write to, and no branch reads
/// it) mirrors the oracle signature for fidelity but is otherwise dead, as it
/// is in the oracle itself.
///
/// `PerpendicularVector` -> `PerpendicularVectorMP`: the resolved-call-surface
/// table flagged this name unconfirmed, but `tr_surface.rs`'s own
/// `PerpendicularVector` transcription (and `q_math.rs`'s
/// `PerpendicularVectorMP as PerpendicularVector` re-export) already establish
/// it as this codebase's MP idiom, so no escalation is needed.
///
/// `assert(decalShader)` -> `debug_assert!(decal_shader != 0)`.
///
/// Source: `oracle/codemp/renderer/tr_scene.cpp:517-613`
#[allow(clippy::too_many_arguments)]
pub fn RE_AddDecalToScene(
    frame: &mut FrameData,
    assets: &RenderAssets,
    scene: &mut SceneState,
    cvars: &RendererCvars,
    common: &mut Common,
    world_root: &mut MarkNode,
    frame_state: &mut FrameState,
    refdef_time: i32,
    decal_shader: qhandle_t,
    origin: Vec3,
    dir: Vec3,
    orientation: f32,
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
    _alpha_fade: bool,
    radius: f32,
    temporary: bool,
) {
    debug_assert!(decal_shader != 0);

    let r_markcount = common.cvar(cvars.r_markcount).integer;
    if r_markcount <= 0 && !temporary {
        return;
    }

    if radius <= 0.0 {
        com_error(
            errorParm_t::ERR_FATAL,
            "RE_AddDecalToScene:  called with <= 0 radius".to_string(),
        );
    }

    // create the texture axis
    let mut axis: [Vec3; 3] = [[0.0; 3]; 3];
    VectorNormalize2(dir, &mut axis[0]);
    let axis0 = axis[0];
    PerpendicularVectorMP(&mut axis[1], axis0);
    let axis1 = axis[1];
    RotatePointAroundVector(&mut axis[2], axis0, axis1, orientation);
    let axis2 = axis[2];
    CrossProduct(axis0, axis2, &mut axis[1]);
    let axis1 = axis[1];

    // C's `0.5 * 1.0 / radius` promotes to double (bare literals); f64
    // intermediate, rounded once at the assignment (ruling 12).
    let tex_coord_scale = (0.5f64 * 1.0f64 / radius as f64) as f32;

    // create the full polygon
    let mut original_points: [Vec3; 4] = [[0.0; 3]; 4];
    for i in 0..3usize {
        original_points[0][i] = origin[i] - radius * axis1[i] - radius * axis2[i];
        original_points[1][i] = origin[i] + radius * axis1[i] - radius * axis2[i];
        original_points[2][i] = origin[i] + radius * axis1[i] + radius * axis2[i];
        original_points[3][i] = origin[i] - radius * axis1[i] + radius * axis2[i];
    }

    // get the fragments
    let mut projection: Vec3 = [0.0; 3];
    _VectorScale(dir, -20.0, &mut projection);
    let mut mark_points: Vec<Vec3> = Vec::new();
    let mut mark_fragments = Vec::new();
    let num_fragments = R_MarkFragments(
        &original_points,
        projection,
        MAX_DECAL_POINTS,
        &mut mark_points,
        MAX_DECAL_FRAGMENTS,
        &mut mark_fragments,
        world_root,
        frame_state,
    );

    // §19: C's out-of-range float->byte conversion is UB; Rust's `as u8`
    // saturates, which is the one defined behavior picked here.
    let colors: [u8; 4] = [
        (red * 255.0) as u8,
        (green * 255.0) as u8,
        (blue * 255.0) as u8,
        (alpha * 255.0) as u8,
    ];

    let zero_vert = polyVert_t {
        xyz: [0.0; 3],
        st: [0.0; 2],
        modulate: [0; 4],
    };

    for i in 0..num_fragments as usize {
        let mf = mark_fragments[i];

        // we have an upper limit on the complexity of polygons that we store
        // persistantly
        let num_points = if mf.numPoints > MAX_VERTS_ON_DECAL_POLY as i32 {
            MAX_VERTS_ON_DECAL_POLY as i32
        } else {
            mf.numPoints
        };
        // Raven clamps `mf->numPoints` in place; `mf` is a `Copy` snapshot
        // here, so the clamp is written back to the owning element too.
        // Source: oracle/codemp/renderer/tr_scene.cpp:578-581
        mark_fragments[i].numPoints = num_points;

        let mut verts: [PolyVert; MAX_VERTS_ON_DECAL_POLY] = [zero_vert; MAX_VERTS_ON_DECAL_POLY];
        for j in 0..num_points as usize {
            let point = mark_points[mf.firstPoint as usize + j];
            verts[j].xyz = point;

            let mut delta: Vec3 = [0.0; 3];
            _VectorSubtract(point, origin, &mut delta);
            // Both `st[]` writes are `0.5 + float_expr` — the bare `0.5`
            // double literal promotes the whole RHS to double before the
            // implicit truncation back to `float` (ruling 12).
            verts[j].st[0] = (0.5f64 + (_DotProduct(delta, axis1) * tex_coord_scale) as f64) as f32;
            verts[j].st[1] = (0.5f64 + (_DotProduct(delta, axis2) * tex_coord_scale) as f64) as f32;

            // `*(int *)v->modulate = *(int *)colors;` reinterpret-casts to
            // copy all 4 bytes in one shot; a plain array assignment copies
            // the same 4 bytes without the raw-pointer cast the
            // interior-safety law forbids.
            verts[j].modulate = colors;
        }

        // if it is a temporary (shadow) mark, add it immediately and forget
        // about it
        if temporary {
            RE_AddPolyToScene(
                frame,
                assets,
                common,
                decal_shader,
                &verts[..num_points as usize],
                num_points as usize,
                1,
            );
            continue;
        }

        // otherwise save it persistantly
        let (decal_type, decal_index) =
            RE_AllocDecal(scene, cvars, &*common, refdef_time, DECALPOLY_TYPE_NORMAL);
        let decal = &mut scene.decal_polys[decal_type][decal_index];
        decal.time = refdef_time;
        decal.shader = decal_shader;
        decal.poly.numVerts = num_points;
        decal.color[0] = red;
        decal.color[1] = green;
        decal.color[2] = blue;
        decal.color[3] = alpha;
        // `memcpy( decal->verts, verts, mf->numPoints * sizeof( verts[0] ) );`
        // — both sides are `polyVert_t` (`Copy`), so a slice copy reproduces
        // the byte-count-bounded memcpy without a raw pointer.
        decal.verts[..num_points as usize].copy_from_slice(&verts[..num_points as usize]);
    }
}

// ---------------------------------------------------------------------
// wave 13
// ---------------------------------------------------------------------

/// Raven `RE_RenderScene` — commits a scene's `refdef_t` into a
/// `FrameEvent::RenderScene` for the render side to replay (DEC-50).
///
/// This is a trap-time handler (`CG_R_/UI_R_RENDERSCENE`). It records the
/// scene into `FrameData` and returns. The render side replays the event and
/// runs `R_RenderView` against render-side world assets, so this fn never
/// calls `R_RenderView` itself (ruling 3: `R_RenderView` touches
/// render-thread-only `GpuResources`).
///
/// The `refdef` payload carries the scalar `trRefdef_t` fields. The four
/// oracle count+pointer pairs (entities, polys, dlights, draw surfaces) stay
/// out. The render side rebuilds those from the `Add*ToScene` events. The
/// dynamic-light disable decision rides `disable_dynamic_light` because
/// `num_dlights` is one of those rebuilt-render-side counts.
///
/// `light_styles` is the A11 snapshot: the sim copies `LightStyleTable::
/// colors` into the event so render-side consumers read the frame's snapshot,
/// not the live table.
///
/// Still deferred below: `startTime`/`frontEndMsec` timing, the `R_RenderView`
/// call, and `RE_RenderWorldEffects`/`RE_RenderAutoMap` (both unported).
///
/// Source: `oracle/codemp/renderer/tr_scene.cpp:706-874`
pub fn RE_RenderScene(
    fd: &refdef_t,
    frame: &mut FrameData,
    assets: &RenderAssets,
    cvars: &RendererCvars,
    scene: &mut SceneState,
    common: &mut Common,
    light_styles: &LightStyleTable,
) {
    if !assets.registered {
        return;
    }

    // DEFERRED: GLimp_LogComment("====== RE_RenderScene =====\n") —
    // unreachable from this crate: `crates/mp/renderer/Cargo.toml` doesn't
    // depend on `mp_engine_client` (where `GLimp_LogComment`/`GLimp_EndFrame`
    // live), and its raw `*mut c_char` signature would need the unsafe
    // pointer construction the interior-safety law forbids even if it did —
    // the same ruling `tr_backend.rs`'s `RB_EndSurface`/
    // `R_IssuePendingRenderCommands` ports already made for this exact call
    // (DEC-37 A13.2).
    // Source: oracle/codemp/renderer/tr_scene.cpp:714

    if common.cvar(cvars.r_norefresh).integer != 0 {
        return;
    }

    // DEFERRED: `startTime = Sys_Milliseconds()*com_timescale->value;` —
    // paired with `tr.frontEndMsec += ... - startTime` at the end of this
    // fn (oracle line 866); computing `startTime` alone has nowhere to feed
    // since `tr.frontEndMsec` has no `FrameState`/`BackEndCounters` field
    // (`BackEndCounters` is the established empty tier-3 placeholder, owned
    // by "the R4 backend wave" per `tr_cmds.rs`'s `R_PerformanceCounters`
    // DEFERRED note) — deferring both together rather than landing a
    // computation with no observable effect.
    // Source: oracle/codemp/renderer/tr_scene.cpp:720,866

    if assets.world.is_none() && (fd.rdflags & RDF_NOWORLDMODEL) == 0 {
        com_error(
            errorParm_t::ERR_DROP,
            "R_RenderScene: NULL worldmodel".to_string(),
        );
    }

    // Build the scene refdef payload field by field, in oracle order.
    let mut refdef = TrRefdef::default();

    // `Com_Memcpy( tr.refdef.text, fd->text, ... )` — copy each NUL-terminated
    // Latin-1 row into an owned string (one row per `MAX_RENDER_STRINGS`).
    // Source: oracle/codemp/renderer/tr_scene.cpp:726
    for row in 0..MAX_RENDER_STRINGS {
        let bytes = &fd.text[row][..MAX_RENDER_STRING_LENGTH];
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        refdef
            .text
            .push(bytes[..end].iter().map(|&b| b as u8 as char).collect());
    }

    refdef.x = fd.x;
    refdef.y = fd.y;
    refdef.width = fd.width;
    refdef.height = fd.height;
    refdef.fov_x = fd.fov_x;
    refdef.fov_y = fd.fov_y;

    refdef.view_origin = fd.vieworg;
    refdef.view_axis = fd.viewaxis;

    refdef.time = fd.time;
    // `frametime = fd->time - lastTime; lastTime = fd->time;` — `lastTime` is
    // this file's `SceneState` carrier (kind-3 fn-scope state; DEC-37 A13.3).
    let mut frametime = fd.time - scene.last_time;
    scene.last_time = fd.time;

    // `skyboxportal` is sticky in the oracle - set to 1 and never cleared here.
    // `drawskyboxportal` is set or cleared each scene. Both live on the
    // trap-side `SceneState` carrier, then ride the payload to write
    // `FrameState::skyboxportal`/`drawskyboxportal` render-side.
    // Source: oracle/codemp/renderer/tr_scene.cpp:744-756
    if fd.rdflags & RDF_SKYBOXPORTAL != 0 {
        scene.skyboxportal = 1;
    }
    if fd.rdflags & RDF_DRAWSKYBOX != 0 {
        scene.drawskyboxportal = 1;
    } else {
        scene.drawskyboxportal = 0;
    }
    refdef.skyboxportal = scene.skyboxportal;
    refdef.drawskyboxportal = scene.drawskyboxportal;

    // Clamp `frametime` to 0-500 ms.
    if frametime > 500 {
        frametime = 500;
    } else if frametime < 0 {
        frametime = 0;
    }
    refdef.frametime = frametime;
    refdef.rdflags = fd.rdflags;

    // Copy the areamask over and note a change, which forces `R_MarkLeaves` to
    // re-mark even if the view did not move. The previous scene's bits live on
    // the `SceneState` carrier because the render-thread `tr.refdef` is not
    // reachable at trap time (ruling 3). The oracle diffs 4 bytes at a time
    // with XOR - a byte compare gives the same nonzero result.
    // Source: oracle/codemp/renderer/tr_scene.cpp:768-786
    refdef.areamask_modified = false;
    if fd.rdflags & RDF_NOWORLDMODEL == 0 {
        let mut area_diff = false;
        for i in 0..MAX_MAP_AREA_BYTES {
            if scene.refdef_areamask[i] != fd.areamask[i] {
                area_diff = true;
            }
            scene.refdef_areamask[i] = fd.areamask[i];
        }
        if area_diff {
            // a door just opened or something
            refdef.areamask_modified = true;
        }
    }
    refdef.areamask = scene.refdef_areamask;

    // derived info
    refdef.float_time = refdef.time as f32 * 0.001;

    // Add the decals here because decals add polys, and the polys must be
    // added before the scene is sealed.
    // Source: oracle/codemp/renderer/tr_scene.cpp:805-810
    if fd.rdflags & RDF_NOWORLDMODEL == 0 {
        R_AddDecals(frame, assets, scene, cvars, common, fd.time);
    }

    // The oracle clears `tr.refdef.num_dlights` when dynamic light is off or
    // vertex light is on. `num_dlights` has no `TrRefdef` field because the
    // render side replays dlights from events, so the payload carries the
    // disable decision as a bool the replay reads.
    // Source: oracle/codemp/renderer/tr_scene.cpp:815-822
    let disable_dynamic_light = common.cvar(cvars.r_dynamiclight).integer == 0
        || common.cvar(cvars.r_vertexLight).integer == 1;

    // `tr.frameSceneNum++; tr.sceneCount++;` is render-thread state
    // (`FrameState::frame_scene_num`/`scene_count`), so ruling 3 keeps it off
    // this trap-time fn. The render-side `R_RenderView` driver bumps both
    // before it stamps `view.frameSceneNum` (see `boot.rs`'s
    // `load_world_and_render`).
    // Source: oracle/codemp/renderer/tr_scene.cpp:829-830

    // Seal the scene. This push stands in for the oracle's `R_RenderView`
    // call: the render side replays the event and runs `R_RenderView` itself
    // (DEC-50), against render-side world assets (ruling 3).
    // Source: oracle/codemp/renderer/tr_scene.cpp:832-855
    frame.events.push(FrameEvent::RenderScene {
        refdef,
        light_styles: light_styles.colors,
        disable_dynamic_light,
    });

    // The `r_firstSceneDrawSurf`/`Entity`/`Dlight`/`Poly` per-scene-offset
    // bookkeeping (oracle lines 857-862) is not a Rust write at all: same
    // "no dedicated field, a property of the `FrameData` under
    // construction" disposition `R_ToggleSmpFrame`/`RE_ClearScene` (this
    // file, above) already establish for these counters — nothing to port.
    // Source: oracle/codemp/renderer/tr_scene.cpp:857-862

    scene.ref_ent_parent = None;

    // DEFERRED: `tr.frontEndMsec += Sys_Milliseconds()*com_timescale->value
    // - startTime;` — paired with `startTime` above.
    // Source: oracle/codemp/renderer/tr_scene.cpp:866

    // DEFERRED: `RE_RenderWorldEffects()` — unported: `tr_cmds.rs` carries
    // its own `DEFERRED: RE_RenderWorldEffects` marker (no callable fn under
    // this name exists anywhere in the crate; verified by grep, not
    // assumed).
    // Source: oracle/codemp/renderer/tr_scene.cpp:868

    // DEFERRED: `if (tr.refdef.rdflags & RDF_AUTOMAP) RE_RenderAutoMap();` —
    // `RDF_AUTOMAP`'s value (32) is independently confirmed (`tr_main.rs`),
    // but `RE_RenderAutoMap` itself is unported — same as
    // `RE_RenderWorldEffects` above, `tr_cmds.rs` carries its own
    // `DEFERRED: RE_RenderAutoMap` marker.
    // Source: oracle/codemp/renderer/tr_scene.cpp:870-873
}

#[cfg(test)]
mod ghoul2_token_tests {
    use super::{ghoul2_token_decode, ghoul2_token_encode};
    use mp_engine_ghoul2::info_array::Ghoul2Handle;

    // A null token decodes to no instance, and no instance encodes to null.
    #[test]
    fn null_token_round_trips_to_none() {
        assert!(ghoul2_token_decode(ghoul2_token_encode(None)).is_none());
        assert!(ghoul2_token_encode(None).is_null());
    }

    // A live handle survives encode then decode unchanged. The first handle a
    // fresh arena issues is `MAX_G2_MODELS` (1024), so this covers a real value.
    #[test]
    fn live_handle_round_trips_through_the_token() {
        let handle = Ghoul2Handle(1024);
        let token = ghoul2_token_encode(Some(handle));
        assert!(!token.is_null());
        assert_eq!(ghoul2_token_decode(token), Some(handle));
    }
}
