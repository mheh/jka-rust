//! Raven `tr_shade_calc.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_shade_calc.cpp`

// Raven-named functions/types keep their original casing across this
// transcription, matching the rest of the renderer/engine crates.
#![allow(non_snake_case)]
// Wave-0 ports of Raven `static` helpers: private by fidelity, with their
// callers landing in later R3 waves.
#![allow(dead_code)]

use core::f32::consts::PI;

use mp_engine_qcommon::common::com_error;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::q_math::{_DotProduct, _VectorSubtract, VectorLengthSquared};
use mp_qshared::shared::vec3_t;
use native_math::qmath::Q_rsqrt;

use crate::render_state::placeholders::RefEntity;
use crate::render_state::render_assets::RenderAssets;
use crate::tr_local::deform_stage_t::deformStage_t;
use crate::tr_local::fog_t::fog_t;
use crate::tr_local::gen_func_t::genFunc_t;
use crate::tr_local::orientationr_t::orientationr_t;
use crate::tr_local::tex_mod_info_t::texModInfo_t;
use crate::tr_local::wave_form_t::waveForm_t;

// This wave threads `RenderAssets` (`## State ownership` row `tr` registries
// SPLIT) and `RefEntity` (`crate::render_state::placeholders`, owned by the
// `tr_scene` R3 wave) as several fns below expect them; both are still
// skeleton stubs owned by other waves this wave may not touch (porting-rules
// process — `tr_light.rs`/`tr_backend.rs`/`tr_bsp.rs` precedent). The fields
// this file's fns read land on those structs with the field-merge step of
// this wave's integration:
// - `RenderAssets::function_tables` (`FunctionTables`, owned by the
//   `tr_init` R3 wave): `sin_table: [f32; FUNCTABLE_SIZE]` (`tr.sinTable`,
//   already assumed by `tr_light.rs`).
// - `RefEntity` (owned by the `tr_scene` R3 wave), extending the subset
//   `tr_light.rs` already flattened onto it (`renderfx`, `origin`,
//   `lighting_origin`, `ambient_light`, `directed_light`, `light_dir`):
//   `shader_rgba: [u8; 4]` (`e.shaderRGBA`), `h_model: i32` (`e.hModel`),
//   `has_ghoul2: bool` (`e.ghoul2 != NULL`, flattened truthy per the
//   interior-safety law — `RefEntity` may not hold the tier-1 raw pointer),
//   `old_origin: vec3_t` (`e.oldorigin`), `end_time: f32` (`e.endTime`).
//
// `tess` (`shaderCommands_t`) is DISSOLVED into R4's tessellation/
// vertex-building pipeline (R2 `## State ownership` row `tess`) — it never
// gets an R3 carrier. Every fn below that reads `tess.xyz`/`tess.normal`/
// `tess.texCoords`/`tess.numVertexes`/`tess.shaderTime`/`tess.fogNum`/
// `tess.shader` takes the equivalent data as an explicit slice/scalar
// parameter instead (C pointer-walk→slice, out-param→return dictionary
// entries), replacing the implicit global with the caller-supplied buffer
// R4's pipeline will thread through.
//
// `backEnd.ori`/`backEnd.viewParms.ori` (`orientationr_t`) already have a
// real tier-2 shape (`crate::tr_local::orientationr_t`) — threaded directly
// as parameters rather than through the still-empty `FrameState`/
// `OrientationR` placeholder (same choice `tr_light.rs`'s
// `R_TransformDlights` made). `backEnd.refdef.time` and
// `tr.world->fogs[tess.fogNum]` collapse to plain `i32`/`&fog_t` parameters
// for the same reason — `fog_t` is already a real tier-2 struct, and
// `tess.fogNum`'s fog lookup is itself `tess`-dissolved.

/// Per-subsystem render-thread scratch for `tr_shade_calc.cpp`'s file-scope
/// globals with no `## State ownership` row of their own — named by this
/// wave per DEC-37 A13.3 (STATE HOMES row for `RB_CalcSpecularAlpha`/
/// `lightOrigin`).
pub struct ShadeCalcState {
    /// Raven `vec3_t lightOrigin` — fallback light-direction source used by
    /// `RB_CalcSpecularAlpha` when the current entity has no world lights to
    /// derive `lightDir` from. Written by the stage-setup fns this file's
    /// higher waves own; render-thread-local scratch.
    ///
    /// Source: `oracle/codemp/renderer/tr_shade_calc.cpp` (extern `lightOrigin`)
    pub light_origin: vec3_t,
}

/// `FUNCTABLE_SIZE`.
///
/// Source: `oracle/codemp/renderer/tr_local.h:1247`
const FUNCTABLE_SIZE: usize = 1024;

/// `FUNCTABLE_MASK` (`FUNCTABLE_SIZE - 1`).
///
/// Source: `oracle/codemp/renderer/tr_local.h:1248`
const FUNCTABLE_MASK: i32 = FUNCTABLE_SIZE as i32 - 1;

/// Wraps a signed table offset into `[0, FUNCTABLE_SIZE)`, matching the
/// oracle's `off & FUNCTABLE_MASK` bitwise wrap on `int off`.
fn functable_index(off: i32) -> usize {
    (off & FUNCTABLE_MASK) as usize
}

/// Raven `RF_DISINTEGRATE1` — does a procedural hole-ripping thing.
///
/// Source: `oracle/codemp/cgame/tr_types.h:47`
const RF_DISINTEGRATE1: i32 = 0x20000;

/// Raven `RF_DISINTEGRATE2` — does a procedural hole-ripping thing with
/// scaling at the ripping point.
///
/// Source: `oracle/codemp/cgame/tr_types.h:48`
const RF_DISINTEGRATE2: i32 = 0x40000;

/// Raven `static float *TableForFunc( genFunc_t func )`.
///
/// Raven's `tess.shader->name` (used only in the error message) is
/// `tess`-dissolved — the caller passes the shader name explicitly
/// (out-param→return / C pointer-walk→slice dictionary entries).
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:11-32`
fn TableForFunc<'a>(func: genFunc_t, assets: &'a RenderAssets, shader_name: &str) -> &'a [f32] {
    let tables = &assets.function_tables;
    match func {
        genFunc_t::GF_SIN => &tables.sin_table,
        genFunc_t::GF_TRIANGLE => &tables.triangle_table,
        genFunc_t::GF_SQUARE => &tables.square_table,
        genFunc_t::GF_SAWTOOTH => &tables.saw_tooth_table,
        genFunc_t::GF_INVERSE_SAWTOOTH => &tables.inverse_saw_tooth_table,
        // GF_NONE and the default case both fall through to Com_Error in the
        // oracle switch.
        _ => com_error(
            errorParm_t::ERR_DROP,
            format!(
                "TableForFunc called with invalid function '{}' in shader '{}'\n",
                func as i32, shader_name
            ),
        ),
    }
}

/// Raven `void RB_CalcBulgeVertexes( deformStage_t *ds )`.
///
/// Raven: Old bulge code (a per-texcoord variant) is kept as a comment in the
/// oracle, superseded by the height-only fast path below.
///
/// `tess.xyz`/`tess.normal`/`tess.texCoords[0]`/`tess.numVertexes` are
/// `tess`-dissolved — threaded as slices; `backEnd.refdef.time` collapses to
/// a plain `i32`; `tr.sinTable` comes through `RenderAssets`.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:193-255`
pub fn RB_CalcBulgeVertexes(
    ds: &deformStage_t,
    xyz: &mut [[f32; 4]],
    normal: &[[f32; 4]],
    tex_coords0: &[[f32; 2]],
    refdef_time: i32,
    assets: &RenderAssets,
) {
    if ds.bulgeSpeed == 0.0 && ds.bulgeWidth == 0.0 {
        // We don't have a speed and width, so just use height to expand uniformly
        for (v, n) in xyz.iter_mut().zip(normal.iter()) {
            v[0] += n[0] * ds.bulgeHeight;
            v[1] += n[1] * ds.bulgeHeight;
            v[2] += n[2] * ds.bulgeHeight;
        }
    } else {
        // I guess do some extra dumb stuff..the fact that it uses ST seems bad though because skin pages may be set up in certain ways that can cause
        //	very noticeable seams on sufaces ( like on the huge ion_cannon ).
        let now = refdef_time as f32 * ds.bulgeSpeed * 0.001;
        let sin_table = &assets.function_tables.sin_table;

        for ((v, n), st) in xyz.iter_mut().zip(normal.iter()).zip(tex_coords0.iter()) {
            let off = ((FUNCTABLE_SIZE as f32 / (PI * 2.0)) * (st[0] * ds.bulgeWidth + now)) as i32;
            let scale = sin_table[functable_index(off)] * ds.bulgeHeight;

            v[0] += n[0] * scale;
            v[1] += n[1] * scale;
            v[2] += n[2] * scale;
        }
    }
}

/// Raven `static void GlobalVectorToLocal( const vec3_t in, vec3_t out )`.
///
/// Out-param→return (`out`); `backEnd.ori` threads as the already-real
/// `orientationr_t`.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:368-372`
fn GlobalVectorToLocal(input: vec3_t, ori: &orientationr_t) -> vec3_t {
    [
        _DotProduct(input, ori.axis[0]),
        _DotProduct(input, ori.axis[1]),
        _DotProduct(input, ori.axis[2]),
    ]
}

/// Raven `void RB_CalcColorFromEntity( unsigned char *dstColors )`.
///
/// `backEnd.currentEntity` (a nullable pointer) becomes `Option<&RefEntity>`;
/// `tess.numVertexes` is `dst_colors.len()`.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:646-661`
pub fn RB_CalcColorFromEntity(dst_colors: &mut [[u8; 4]], current_entity: Option<&RefEntity>) {
    let Some(ent) = current_entity else {
        return;
    };
    let c = ent.shader_rgba;

    for pixel in dst_colors.iter_mut() {
        *pixel = c;
    }
}

/// Raven `void RB_CalcColorFromOneMinusEntity( unsigned char *dstColors )`.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:691-712`
pub fn RB_CalcColorFromOneMinusEntity(
    dst_colors: &mut [[u8; 4]],
    current_entity: Option<&RefEntity>,
) {
    let Some(ent) = current_entity else {
        return;
    };
    let rgba = ent.shader_rgba;
    // this trashes alpha, but the AGEN block fixes it
    let inv_modulate = [255 - rgba[0], 255 - rgba[1], 255 - rgba[2], 255 - rgba[3]];

    for pixel in dst_colors.iter_mut() {
        *pixel = inv_modulate;
    }
}

/// Raven `void RB_CalcAlphaFromEntity( unsigned char *dstColors )`.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:733-746`
pub fn RB_CalcAlphaFromEntity(dst_colors: &mut [[u8; 4]], current_entity: Option<&RefEntity>) {
    let Some(ent) = current_entity else {
        return;
    };
    let alpha = ent.shader_rgba[3];

    for pixel in dst_colors.iter_mut() {
        pixel[3] = alpha;
    }
}

/// Raven `void RB_CalcAlphaFromOneMinusEntity( unsigned char *dstColors )`.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:767-780`
pub fn RB_CalcAlphaFromOneMinusEntity(
    dst_colors: &mut [[u8; 4]],
    current_entity: Option<&RefEntity>,
) {
    let Some(ent) = current_entity else {
        return;
    };
    let alpha = ent.shader_rgba[3];

    for pixel in dst_colors.iter_mut() {
        pixel[3] = 0xff - alpha;
    }
}

/// Raven `void RB_CalcFogTexCoords( float *st )`.
///
/// `tr.world->fogs + tess.fogNum` is `tess`-dissolved (the fog lookup itself
/// depended on the dissolved `tess.fogNum`) — the caller passes the resolved
/// `fog_t` directly; `backEnd.ori`/`backEnd.viewParms.ori` thread as the
/// already-real `orientationr_t`; `tess.xyz`/output `st` are slices.
///
/// Raven: `#ifdef _XBOX` inverts `fogDistanceVector`'s sign — MP retail
/// builds the non-`_XBOX` branch, transcribed here.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:983-1068`
pub fn RB_CalcFogTexCoords(
    st: &mut [[f32; 2]],
    xyz: &[[f32; 4]],
    fog: &fog_t,
    ori: &orientationr_t,
    view_ori: &orientationr_t,
) {
    // all fogging distance is based on world Z units
    let mut local_vec = [0.0f32; 3];
    _VectorSubtract(ori.origin, view_ori.origin, &mut local_vec);

    let mut fog_distance_vector = [
        -ori.modelMatrix[2],
        -ori.modelMatrix[6],
        -ori.modelMatrix[10],
        _DotProduct(local_vec, view_ori.axis[0]),
    ];

    // scale the fog vectors based on the fog's thickness
    fog_distance_vector[0] *= fog.tcScale;
    fog_distance_vector[1] *= fog.tcScale;
    fog_distance_vector[2] *= fog.tcScale;
    fog_distance_vector[3] *= fog.tcScale;

    // rotate the gradient vector for this orientation
    let eye_t;
    let fog_depth_vector: [f32; 4];
    if fog.hasSurface != 0 {
        let surface = fog.surface;
        let surface3 = [surface[0], surface[1], surface[2]];
        let mut fdv = [0.0f32; 4];
        fdv[0] =
            surface[0] * ori.axis[0][0] + surface[1] * ori.axis[0][1] + surface[2] * ori.axis[0][2];
        fdv[1] =
            surface[0] * ori.axis[1][0] + surface[1] * ori.axis[1][1] + surface[2] * ori.axis[1][2];
        fdv[2] =
            surface[0] * ori.axis[2][0] + surface[1] * ori.axis[2][1] + surface[2] * ori.axis[2][2];
        fdv[3] = -surface[3] + _DotProduct(ori.origin, surface3);
        fog_depth_vector = fdv;

        eye_t = _DotProduct(ori.viewOrigin, [fdv[0], fdv[1], fdv[2]]) + fdv[3];
    } else {
        // non-surface fog always has eye inside
        eye_t = 1.0;
        fog_depth_vector = [0.0, 0.0, 0.0, 1.0];
    }

    // see if the viewpoint is outside
    // this is needed for clipping distance even for constant fog
    let eye_outside = eye_t < 0.0;

    fog_distance_vector[3] += 1.0 / 512.0;

    // calculate density for each point
    for (v, st) in xyz.iter().zip(st.iter_mut()) {
        let v3 = [v[0], v[1], v[2]];
        // calculate the length in fog
        let s = _DotProduct(
            v3,
            [
                fog_distance_vector[0],
                fog_distance_vector[1],
                fog_distance_vector[2],
            ],
        ) + fog_distance_vector[3];
        let mut t = _DotProduct(
            v3,
            [
                fog_depth_vector[0],
                fog_depth_vector[1],
                fog_depth_vector[2],
            ],
        ) + fog_depth_vector[3];

        // partially clipped fogs use the T axis
        if eye_outside {
            if t < 1.0 {
                // point is outside, so no fogging
                t = 1.0 / 32.0;
            } else {
                // cut the distance at the fog plane
                t = 1.0 / 32.0 + 30.0 / 32.0 * t / (t - eye_t);
            }
        } else if t < 0.0 {
            // point is outside, so no fogging
            t = 1.0 / 32.0;
        } else {
            t = 31.0 / 32.0;
        }

        st[0] = s;
        st[1] = t;
    }
}

/// Raven `void RB_CalcEnvironmentTexCoords( float *st )`.
///
/// `VectorNormalizeFast` is an inline header helper with no existing
/// equivalent (resolved call surface) — inlined via `Q_rsqrt`, matching its
/// standard `ilength = Q_rsqrt(DotProduct(v,v)); v *= ilength;` body.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:1075-1099`
pub fn RB_CalcEnvironmentTexCoords(
    st: &mut [[f32; 2]],
    xyz: &[[f32; 4]],
    normal: &[[f32; 4]],
    ori: &orientationr_t,
) {
    for ((v, n), st) in xyz.iter().zip(normal.iter()).zip(st.iter_mut()) {
        let mut viewer = [0.0f32; 3];
        _VectorSubtract(ori.viewOrigin, [v[0], v[1], v[2]], &mut viewer);

        // VectorNormalizeFast( viewer )
        let ilength = Q_rsqrt(_DotProduct(viewer, viewer));
        viewer[0] *= ilength;
        viewer[1] *= ilength;
        viewer[2] *= ilength;

        let n3 = [n[0], n[1], n[2]];
        let d = _DotProduct(n3, viewer);

        let reflected = [
            n3[0] * 2.0 * d - viewer[0],
            n3[1] * 2.0 * d - viewer[1],
            n3[2] * 2.0 * d - viewer[2],
        ];

        st[0] = 0.5 + reflected[1] * 0.5;
        st[1] = 0.5 - reflected[2] * 0.5;
    }
}

/// Raven `void RB_CalcTurbulentTexCoords( const waveForm_t *wf, float *st )`.
///
/// `tess.shaderTime` collapses to a plain `f32`; `tr.sinTable` comes through
/// `RenderAssets`.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:1104-1119`
pub fn RB_CalcTurbulentTexCoords(
    wf: &waveForm_t,
    st: &mut [[f32; 2]],
    xyz: &[[f32; 4]],
    shader_time: f32,
    assets: &RenderAssets,
) {
    let now = wf.phase + shader_time * wf.frequency;
    let sin_table = &assets.function_tables.sin_table;

    for (v, st) in xyz.iter().zip(st.iter_mut()) {
        let s = st[0];
        let t = st[1];

        let idx_s = functable_index(
            (((v[0] + v[2]) * (1.0 / 128.0) * 0.125 + now) * FUNCTABLE_SIZE as f32) as i32,
        );
        let idx_t =
            functable_index(((v[1] * (1.0 / 128.0) * 0.125 + now) * FUNCTABLE_SIZE as f32) as i32);

        st[0] = s + sin_table[idx_s] * wf.amplitude;
        st[1] = t + sin_table[idx_t] * wf.amplitude;
    }
}

/// Raven `void RB_CalcScaleTexCoords( const float scale[2], float *st )`.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:1124-1133`
pub fn RB_CalcScaleTexCoords(scale: [f32; 2], st: &mut [[f32; 2]]) {
    for st in st.iter_mut() {
        st[0] *= scale[0];
        st[1] *= scale[1];
    }
}

/// Raven `void RB_CalcScrollTexCoords( const float scrollSpeed[2], float *st )`.
///
/// `tess.shaderTime` collapses to a plain `f32`; `floor` is `f32::floor`.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:1138-1157`
pub fn RB_CalcScrollTexCoords(scroll_speed: [f32; 2], st: &mut [[f32; 2]], shader_time: f32) {
    let time_scale = shader_time;

    let mut adjusted_scroll_s = scroll_speed[0] * time_scale;
    let mut adjusted_scroll_t = scroll_speed[1] * time_scale;

    // clamp so coordinates don't continuously get larger, causing problems
    // with hardware limits
    adjusted_scroll_s -= adjusted_scroll_s.floor();
    adjusted_scroll_t -= adjusted_scroll_t.floor();

    for st in st.iter_mut() {
        st[0] += adjusted_scroll_s;
        st[1] += adjusted_scroll_t;
    }
}

/// Raven `void RB_CalcTransformTexCoords( const texModInfo_t *tmi, float *st )`.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:1162-1174`
pub fn RB_CalcTransformTexCoords(tmi: &texModInfo_t, st: &mut [[f32; 2]]) {
    for st in st.iter_mut() {
        let s = st[0];
        let t = st[1];

        st[0] = s * tmi.matrix[0][0] + t * tmi.matrix[1][0] + tmi.translate[0];
        st[1] = s * tmi.matrix[0][1] + t * tmi.matrix[1][1] + tmi.translate[1];
    }
}

/// Raven `inline long myftol( float f )`.
///
/// PORT-NOTE: the oracle's `static int tmp` is transient FISTP scratch,
/// written then immediately read back within the same call — kind-2
/// rotating scratch/return buffer (three-kind rule), not cross-frame state,
/// so it becomes the owned return value with no carrier (no escalation).
/// `fistp` rounds per the x87 control word's default rounding mode
/// (round-to-nearest, ties-to-even); `f32::round_ties_even` matches.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:1211-1216`
pub fn myftol(f: f32) -> i32 {
    f.round_ties_even() as i32
}

/// Raven `void RB_CalcSpecularAlpha( unsigned char *alphas )`.
///
/// `backEnd.currentEntity` becomes `Option<&RefEntity>`; `lightOrigin` is
/// the DEC-37 A13.3 struct named above (`ShadeCalcState`); `backEnd.ori`
/// threads as the already-real `orientationr_t`; `VectorNormalizeFast` is
/// inlined via `Q_rsqrt` (see `RB_CalcEnvironmentTexCoords`).
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:1283-1336`
pub fn RB_CalcSpecularAlpha(
    alphas: &mut [[u8; 4]],
    xyz: &[[f32; 4]],
    normal: &[[f32; 4]],
    current_entity: Option<&RefEntity>,
    ori: &orientationr_t,
    shade_state: &ShadeCalcState,
) {
    // this is a model so we can use world lights instead fake light
    let entity_light_dir = current_entity.filter(|ent| ent.h_model != 0 || ent.has_ghoul2);

    for ((v, n), a) in xyz.iter().zip(normal.iter()).zip(alphas.iter_mut()) {
        let v3 = [v[0], v[1], v[2]];
        let n3 = [n[0], n[1], n[2]];

        let light_dir = match entity_light_dir {
            Some(ent) => ent.light_dir,
            None => {
                let mut dir = [0.0f32; 3];
                _VectorSubtract(shade_state.light_origin, v3, &mut dir);
                // VectorNormalizeFast( lightDir )
                let ilength = Q_rsqrt(_DotProduct(dir, dir));
                [dir[0] * ilength, dir[1] * ilength, dir[2] * ilength]
            }
        };

        // calculate the specular color
        let d = 2.0 * _DotProduct(n3, light_dir);

        // we don't optimize for the d < 0 case since this tends to
        // cause visual artifacts such as faceted "snapping"
        let reflected = [
            n3[0] * d - light_dir[0],
            n3[1] * d - light_dir[1],
            n3[2] * d - light_dir[2],
        ];

        let mut viewer = [0.0f32; 3];
        _VectorSubtract(ori.viewOrigin, v3, &mut viewer);
        let ilength = Q_rsqrt(_DotProduct(viewer, viewer));
        let mut l = _DotProduct(reflected, viewer);
        l *= ilength;

        let b: u8 = if l < 0.0 {
            0
        } else {
            let l2 = l * l;
            let l4 = l2 * l2;
            let b = (l4 * 255.0) as i32;
            b.min(255) as u8
        };

        a[3] = b;
    }
}

/// Raven `void RB_CalcDisintegrateColors( unsigned char *colors )`.
///
/// Raven dereferences `backEnd.currentEntity->e` unconditionally (no null
/// guard in the oracle) — `ent: &RefEntity` is the defined-behavior
/// replacement (porting-rules §19) rather than an `Option`.
/// `backEnd.refdef.time` collapses to a plain `i32`; `tess.xyz`/output
/// `colors` are slices.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:1545-1636`
pub fn RB_CalcDisintegrateColors(
    colors: &mut [[u8; 4]],
    xyz: &[[f32; 4]],
    ent: &RefEntity,
    refdef_time: i32,
) {
    // calculate the burn threshold at the given time, anything that passes the threshold will get burnt
    // endTime is really the start time, maybe I should just use a completely meaningless substitute?
    let threshold = (refdef_time as f32 - ent.end_time) * 0.045;

    if ent.renderfx & RF_DISINTEGRATE1 != 0 {
        // this handles the blacken and fading out of the regular player model
        for (v, c) in xyz.iter().zip(colors.iter_mut()) {
            let mut temp = [0.0f32; 3];
            _VectorSubtract(ent.old_origin, [v[0], v[1], v[2]], &mut temp);

            let dis = VectorLengthSquared(temp);

            if dis < threshold * threshold {
                // completely disintegrated
                c[3] = 0x00;
            } else if dis < threshold * threshold + 60.0 {
                // blacken before fading out
                *c = [0x0, 0x0, 0x0, 0xff];
            } else if dis < threshold * threshold + 150.0 {
                // darken more
                *c = [0x6f, 0x6f, 0x6f, 0xff];
            } else if dis < threshold * threshold + 180.0 {
                // darken at edge of burn
                *c = [0xaf, 0xaf, 0xaf, 0xff];
            } else {
                // not burning at all yet
                *c = [0xff, 0xff, 0xff, 0xff];
            }
        }
    } else if ent.renderfx & RF_DISINTEGRATE2 != 0 {
        // this handles the glowing, burning bit that scales away from the model
        for (v, c) in xyz.iter().zip(colors.iter_mut()) {
            let mut temp = [0.0f32; 3];
            _VectorSubtract(ent.old_origin, [v[0], v[1], v[2]], &mut temp);

            let dis = VectorLengthSquared(temp);

            if dis < threshold * threshold {
                // done burning
                *c = [0x00, 0x00, 0x00, 0x00];
            } else {
                // still full burn
                *c = [0xff, 0xff, 0xff, 0xff];
            }
        }
    }
}

/// Raven `void RB_CalcDisintegrateVertDeform( void )`.
///
/// Same non-`Option` `ent: &RefEntity` choice as `RB_CalcDisintegrateColors`
/// (no null guard in the oracle).
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:1640-1671`
pub fn RB_CalcDisintegrateVertDeform(
    xyz: &mut [[f32; 4]],
    normal: &[[f32; 4]],
    ent: &RefEntity,
    refdef_time: i32,
) {
    if ent.renderfx & RF_DISINTEGRATE2 != 0 {
        let threshold = (refdef_time as f32 - ent.end_time) * 0.045;

        for (v, n) in xyz.iter_mut().zip(normal.iter()) {
            let mut temp = [0.0f32; 3];
            _VectorSubtract(ent.old_origin, [v[0], v[1], v[2]], &mut temp);

            let scale = VectorLengthSquared(temp);

            if scale < threshold * threshold {
                v[0] += n[0] * 2.0;
                v[1] += n[1] * 2.0;
                v[2] += n[2] * 0.5;
            } else if scale < threshold * threshold + 50.0 {
                v[0] += n[0] * 1.0;
                v[1] += n[1] * 1.0;
                // xyz[2] += normal[2] * 1;
            }
        }
    }
}
