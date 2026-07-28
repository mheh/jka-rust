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

use mp_engine_qcommon::common::{com_error, com_printf, Common};
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::q_color::S_COLOR_YELLOW;
use mp_qshared::shared::q_math::{
    _DotProduct, _VectorAdd, _VectorMA, _VectorScale, _VectorSubtract, vec3_origin, CrossProduct,
    VectorClear, VectorLength, VectorLengthSquared, VectorNormalize,
};
use mp_qshared::shared::vec3_t;
use native_math::qmath::Q_rsqrt;

use crate::render_state::frame_state::FrameState;
use crate::render_state::placeholders::RefEntity;
use crate::render_state::render_assets::RenderAssets;
use crate::tr_image::R_FogFactor;
use crate::tr_local::deform_stage_t::deformStage_t;
use crate::tr_local::deform_t::deform_t;
use crate::tr_local::fog_t::fog_t;
use crate::tr_local::gen_func_t::genFunc_t;
use crate::tr_local::orientationr_t::orientationr_t;
use crate::tr_local::tex_mod_info_t::texModInfo_t;
use crate::tr_local::tex_mod_t::texMod_t;
use crate::tr_local::wave_form_t::waveForm_t;
use crate::tr_noise::{get_noise_time, NoiseState, R_NoiseGet4f};
use crate::tr_shadows::RB_ProjectionShadowDeform;
use crate::tr_surface::{RB_AddQuadStamp, RB_AddQuadStampExt};

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

/// Raven `WAVEVALUE( table, base, amplitude, phase, freq )` — the macro as an
/// inline fn; `tess.shaderTime` is `tess`-dissolved into the `shader_time`
/// parameter, matching this file's other dissolved reads.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:9`
fn WAVEVALUE(
    table: &[f32],
    base: f32,
    amplitude: f32,
    phase: f32,
    freq: f32,
    shader_time: f32,
) -> f32 {
    base + table[functable_index(myftol((phase + shader_time * freq) * FUNCTABLE_SIZE as f32))]
        * amplitude
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

/// Raven `void RB_CalcMoveVertexes( deformStage_t *ds )`.
///
/// `tess.xyz`/`tess.numVertexes` are `tess`-dissolved (mutated-in-place
/// slice); `tess.shaderTime` collapses to a plain `f32`; `tess.shader->name`
/// (used only by `TableForFunc`'s error path) collapses to `shader_name`,
/// same as `RB_CalcBulgeVertexes`.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:265-285`
pub fn RB_CalcMoveVertexes(
    ds: &deformStage_t,
    xyz: &mut [[f32; 4]],
    shader_time: f32,
    assets: &RenderAssets,
    shader_name: &str,
) {
    let table = TableForFunc(ds.deformationWave.func, assets, shader_name);

    let scale = WAVEVALUE(
        table,
        ds.deformationWave.base,
        ds.deformationWave.amplitude,
        ds.deformationWave.phase,
        ds.deformationWave.frequency,
        shader_time,
    );

    let mut offset = [0.0f32; 3];
    _VectorScale(ds.moveVector, scale, &mut offset);

    for v in xyz.iter_mut() {
        v[0] += offset[0];
        v[1] += offset[1];
        v[2] += offset[2];
    }
}

/// Raven `int edgeVerts[6][2]` — the 4-vertex sprite quad's six edges, each a
/// pair of vertex offsets within the quad (DEC-37 A13.3 kind 1: never
/// mutated, so a `const`).
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:453-460`
const EDGE_VERTS: [[usize; 2]; 6] = [[0, 1], [0, 2], [0, 3], [1, 2], [1, 3], [2, 3]];

/// Raven `static void Autosprite2Deform( void )`.
///
/// `tess.xyz`/`tess.indexes`/`tess.numVertexes`/`tess.numIndexes` are
/// `tess`-dissolved, threaded as slices (`numIndexes` collapses to
/// `indexes.len()`); `tess.shader->name` collapses to `shader_name`.
/// `backEnd.currentEntity != &tr.worldEntity` has no home on the owned
/// `RefEntity`/`FrameState` yet (the world-entity sentinel isn't modeled),
/// so the caller resolves the comparison and threads the `bool` result —
/// same out-param collapse this file already applies to `backEnd.refdef.time`.
/// `backEnd.ori`/`backEnd.viewParms.ori` thread as the already-real
/// `orientationr_t`, matching `RB_CalcFogTexCoords`'s `ori`/`view_ori` split.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:462-562`
pub fn Autosprite2Deform(
    xyz: &mut [[f32; 4]],
    indexes: &[i32],
    shader_name: &str,
    is_world_entity: bool,
    ori: &orientationr_t,
    view_ori: &orientationr_t,
    common: &mut Common,
) {
    let num_vertexes = xyz.len() as i32;
    if num_vertexes & 3 != 0 {
        com_printf(
            common,
            &format!(
                "{}Autosprite2 shader {} had odd vertex count",
                S_COLOR_YELLOW.to_str().expect("S_COLOR_YELLOW is ASCII"),
                shader_name
            ),
        );
    }
    if indexes.len() as i32 != (num_vertexes >> 2) * 6 {
        com_printf(
            common,
            &format!(
                "{}Autosprite2 shader {} had odd index count",
                S_COLOR_YELLOW.to_str().expect("S_COLOR_YELLOW is ASCII"),
                shader_name
            ),
        );
    }

    let forward = if is_world_entity {
        view_ori.axis[0]
    } else {
        GlobalVectorToLocal(view_ori.axis[0], ori)
    };

    // this is a lot of work for two triangles...
    // we could precalculate a lot of it is an issue, but it would mess up
    // the shader abstraction
    let mut i = 0usize;
    let mut idx_base = 0usize;
    while i < xyz.len() {
        // find the midpoint
        let quad = [xyz[i], xyz[i + 1], xyz[i + 2], xyz[i + 3]];

        // identify the two shortest edges
        let mut nums = [0usize; 2];
        let mut lengths = [999999.0f32; 2];

        for j in 0..6 {
            let edge = EDGE_VERTS[j];
            let v1 = quad[edge[0]];
            let v2 = quad[edge[1]];

            let mut temp = [0.0f32; 3];
            _VectorSubtract([v1[0], v1[1], v1[2]], [v2[0], v2[1], v2[2]], &mut temp);

            let l = _DotProduct(temp, temp);
            if l < lengths[0] {
                nums[1] = nums[0];
                lengths[1] = lengths[0];
                nums[0] = j;
                lengths[0] = l;
            } else if l < lengths[1] {
                nums[1] = j;
                lengths[1] = l;
            }
        }

        let mut mid = [[0.0f32; 3]; 2];
        for j in 0..2 {
            let edge = EDGE_VERTS[nums[j]];
            let v1 = quad[edge[0]];
            let v2 = quad[edge[1]];

            mid[j][0] = 0.5f32 * (v1[0] + v2[0]);
            mid[j][1] = 0.5f32 * (v1[1] + v2[1]);
            mid[j][2] = 0.5f32 * (v1[2] + v2[2]);
        }

        // find the vector of the major axis
        let mut major = [0.0f32; 3];
        _VectorSubtract(mid[1], mid[0], &mut major);

        // cross this with the view direction to get minor axis
        let mut minor = [0.0f32; 3];
        CrossProduct(major, forward, &mut minor);
        VectorNormalize(&mut minor);

        // re-project the points
        for j in 0..2 {
            let edge = EDGE_VERTS[nums[j]];

            // `0.5 * sqrt(...)` — the oracle's `0.5` here has no `f` suffix
            // and `sqrt` promotes its argument to double, so this evaluates
            // in f64 and rounds to f32 once (wave-0 ruling 12).
            let l = (0.5f64 * (lengths[j] as f64).sqrt()) as f32;

            // we need to see which direction this edge
            // is used to determine direction of projection
            let mut k = 0usize;
            while k < 5 {
                if indexes[idx_base + k] == i as i32 + edge[0] as i32
                    && indexes[idx_base + k + 1] == i as i32 + edge[1] as i32
                {
                    break;
                }
                k += 1;
            }

            let (scale1, scale2) = if k == 5 { (l, -l) } else { (-l, l) };

            let idx0 = i + edge[0];
            let idx1 = i + edge[1];

            let mut out0 = [0.0f32; 3];
            _VectorMA(mid[j], scale1, minor, &mut out0);
            xyz[idx0][0] = out0[0];
            xyz[idx0][1] = out0[1];
            xyz[idx0][2] = out0[2];

            let mut out1 = [0.0f32; 3];
            _VectorMA(mid[j], scale2, minor, &mut out1);
            xyz[idx1][0] = out1[0];
            xyz[idx1][1] = out1[1];
            xyz[idx1][2] = out1[2];
        }

        i += 4;
        idx_base += 6;
    }
}

/// Raven `void RB_CalcModulateColorsByFog( unsigned char *colors )`.
///
/// `tess.numVertexes`-sized `texCoords[SHADER_MAX_VERTEXES][2]` scratch
/// becomes a `Vec` sized to the actual vertex count (`xyz.len()`) rather than
/// the oracle's fixed max-capacity stack buffer — the oracle only ever reads
/// the first `numVertexes` entries, so this is behaviorally identical without
/// needing the unported `SHADER_MAX_VERTEXES` constant. `tess.xyz` collapses
/// to the `xyz` slice `RB_CalcFogTexCoords` needs; `fog`/`ori`/`view_ori`
/// thread through to that same already-ported callee. `assets` carries
/// `tr.fogTable` for `R_FogFactor` (porting-rules §B4).
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:896-911`
pub fn RB_CalcModulateColorsByFog(
    colors: &mut [[u8; 4]],
    xyz: &[[f32; 4]],
    fog: &fog_t,
    ori: &orientationr_t,
    view_ori: &orientationr_t,
    assets: &RenderAssets,
) {
    // calculate texcoords so we can derive density
    // this is not wasted, because it would only have
    // been previously called if the surface was opaque
    let mut tex_coords = vec![[0.0f32; 2]; xyz.len()];
    RB_CalcFogTexCoords(&mut tex_coords, xyz, fog, ori, view_ori);

    for (c, st) in colors.iter_mut().zip(tex_coords.iter()) {
        // C: `1.0` is a double, so the subtraction runs in f64 and rounds
        // once on store to `float f` (ruling 12).
        let f = (1.0f64 - R_FogFactor(assets, st[0], st[1]) as f64) as f32;
        c[0] = (c[0] as f32 * f) as u8;
        c[1] = (c[1] as f32 * f) as u8;
        c[2] = (c[2] as f32 * f) as u8;
    }
}

/// Raven `void RB_CalcModulateAlphasByFog( unsigned char *colors )`.
///
/// Same `texCoords`/`tess.xyz`/`assets` collapse as
/// `RB_CalcModulateColorsByFog`.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:922-935`
pub fn RB_CalcModulateAlphasByFog(
    colors: &mut [[u8; 4]],
    xyz: &[[f32; 4]],
    fog: &fog_t,
    ori: &orientationr_t,
    view_ori: &orientationr_t,
    assets: &RenderAssets,
) {
    // calculate texcoords so we can derive density
    // this is not wasted, because it would only have
    // been previously called if the surface was opaque
    let mut tex_coords = vec![[0.0f32; 2]; xyz.len()];
    RB_CalcFogTexCoords(&mut tex_coords, xyz, fog, ori, view_ori);

    for (c, st) in colors.iter_mut().zip(tex_coords.iter()) {
        // C: `1.0` is a double, so the subtraction runs in f64 and rounds
        // once on store to `float f` (ruling 12).
        let f = (1.0f64 - R_FogFactor(assets, st[0], st[1]) as f64) as f32;
        c[3] = (c[3] as f32 * f) as u8;
    }
}

/// Raven `void RB_CalcModulateRGBAsByFog( unsigned char *colors )`.
///
/// Same `texCoords`/`tess.xyz`/`assets` collapse as
/// `RB_CalcModulateColorsByFog`.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:946-962`
pub fn RB_CalcModulateRGBAsByFog(
    colors: &mut [[u8; 4]],
    xyz: &[[f32; 4]],
    fog: &fog_t,
    ori: &orientationr_t,
    view_ori: &orientationr_t,
    assets: &RenderAssets,
) {
    // calculate texcoords so we can derive density
    // this is not wasted, because it would only have
    // been previously called if the surface was opaque
    let mut tex_coords = vec![[0.0f32; 2]; xyz.len()];
    RB_CalcFogTexCoords(&mut tex_coords, xyz, fog, ori, view_ori);

    for (c, st) in colors.iter_mut().zip(tex_coords.iter()) {
        // C: `1.0` is a double, so the subtraction runs in f64 and rounds
        // once on store to `float f` (ruling 12).
        let f = (1.0f64 - R_FogFactor(assets, st[0], st[1]) as f64) as f32;
        c[0] = (c[0] as f32 * f) as u8;
        c[1] = (c[1] as f32 * f) as u8;
        c[2] = (c[2] as f32 * f) as u8;
        c[3] = (c[3] as f32 * f) as u8;
    }
}

/// Raven `void RB_CalcRotateTexCoords( float degsPerSecond, float *st )`.
///
/// `tess.shaderTime` collapses to a plain `f32`; `tr.sinTable` comes through
/// `RenderAssets`. Raven leaves `texModInfo_t tmi`'s `.type`/`.wave` fields
/// uninitialized stack garbage — `RB_CalcTransformTexCoords` never reads
/// them, so they're zeroed here instead (`TMOD_NONE`/`GF_NONE`, both the
/// oracle's `= 0` enumerator) rather than left as observable UB.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:1179-1202`
pub fn RB_CalcRotateTexCoords(
    degs_per_second: f32,
    st: &mut [[f32; 2]],
    shader_time: f32,
    assets: &RenderAssets,
) {
    let time_scale = shader_time;

    let degs = -degs_per_second * time_scale;
    let index = (degs * (FUNCTABLE_SIZE as f32 / 360.0f32)) as i32;

    let sin_table = &assets.function_tables.sin_table;
    let sin_value = sin_table[functable_index(index)];
    let cos_value = sin_table[functable_index(index + FUNCTABLE_SIZE as i32 / 4)];

    // `0.5 - 0.5 * cosValue + 0.5 * sinValue` — the oracle's `0.5` literals
    // have no `f` suffix, promoting this arithmetic to double; evaluate in
    // f64 and round to f32 once at the assignment into `tmi.translate`
    // (wave-0 ruling 12).
    let translate0 = (0.5f64 - 0.5f64 * cos_value as f64 + 0.5f64 * sin_value as f64) as f32;
    let translate1 = (0.5f64 - 0.5f64 * sin_value as f64 - 0.5f64 * cos_value as f64) as f32;

    let tmi = texModInfo_t {
        r#type: texMod_t::TMOD_NONE,
        wave: waveForm_t {
            func: genFunc_t::GF_NONE,
            base: 0.0,
            amplitude: 0.0,
            phase: 0.0,
            frequency: 0.0,
        },
        matrix: [[cos_value, sin_value], [-sin_value, cos_value]],
        translate: [translate0, translate1],
    };

    RB_CalcTransformTexCoords(&tmi, st);
}

/// Raven `void RB_CalcDiffuseColor( unsigned char *colors )`.
///
/// `backEnd.currentEntity` is dereferenced unconditionally in the oracle (no
/// null guard) — `ent: &RefEntity` is the defined-behavior replacement
/// (porting-rules §19), matching `RB_CalcDisintegrateColors`. The oracle
/// walks `tess.xyz` (`v`) in lockstep with `tess.normal` but never actually
/// dereferences `v` in the loop body, so it's not threaded as a parameter —
/// only `tess.normal`/`tess.numVertexes` (the `normal`/`colors.len()`
/// dictionary collapse) are live reads. `ent->ambientLightInt` has no home
/// on `RefEntity` yet (not among the fields an earlier wave landed, and this
/// wave may not extend `placeholders.rs`), so it threads as an explicit
/// `i32` parameter — the same scalar-collapse pattern this file already uses
/// for `backEnd.refdef.time`. `*(int *)&colors[i*4] = ambientLightInt` (raw
/// pointer reinterpretation) becomes `ambient_light_int.to_le_bytes()` — the
/// interior-safety law forbids the pointer cast, and `to_le_bytes` reproduces
/// the same byte layout on the little-endian target platforms.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:1344-1392`
pub fn RB_CalcDiffuseColor(
    colors: &mut [[u8; 4]],
    normal: &[[f32; 4]],
    ent: &RefEntity,
    ambient_light_int: i32,
) {
    let ambient_light = ent.ambient_light;
    let directed_light = ent.directed_light;
    let light_dir = ent.light_dir;

    for (n, c) in normal.iter().zip(colors.iter_mut()) {
        let n3 = [n[0], n[1], n[2]];
        let incoming = _DotProduct(n3, light_dir);
        if incoming <= 0.0 {
            *c = ambient_light_int.to_le_bytes();
            continue;
        }

        let mut j = myftol(ambient_light[0] + incoming * directed_light[0]);
        if j > 255 {
            j = 255;
        }
        c[0] = j as u8;

        let mut j = myftol(ambient_light[1] + incoming * directed_light[1]);
        if j > 255 {
            j = 255;
        }
        c[1] = j as u8;

        let mut j = myftol(ambient_light[2] + incoming * directed_light[2]);
        if j > 255 {
            j = 255;
        }
        c[2] = j as u8;

        c[3] = 255;
    }
}

/// Raven `static float EvalWaveForm( const waveForm_t *wf )`.
///
/// `backEnd.refdef.floatTime`/`backEnd.refdef.time` and `tess.shaderTime` are
/// `tess`/`refdef`-dissolved (`trRefdef_t`'s `floatTime`/`time` haven't landed
/// on `TrRefdef` yet, matching this file's existing `refdef_time: i32`
/// collapse pattern) — threaded as explicit scalar params;
/// `TableForFunc`'s `assets`/`shader_name` thread through unchanged.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:40-56`
fn EvalWaveForm(
    wf: &waveForm_t,
    noise: &NoiseState,
    refdef_time: i32,
    refdef_float_time: f32,
    shader_time: f32,
    assets: &RenderAssets,
    shader_name: &str,
) -> f32 {
    match wf.func {
        genFunc_t::GF_NOISE => {
            wf.base
                + R_NoiseGet4f(
                    noise,
                    0.0,
                    0.0,
                    0.0,
                    (refdef_float_time + wf.phase) * wf.frequency,
                ) * wf.amplitude
        }
        genFunc_t::GF_RAND => {
            // `backEnd.refdef.time + wf->phase` is `int + float`, promoting to
            // float; the result truncates toward zero on the implicit
            // float->int conversion into `GetNoiseTime`'s `int t` parameter
            // (standard C conversion, not `myftol`/FISTP) — `as i32` matches.
            if get_noise_time(noise, (refdef_time as f32 + wf.phase) as i32) <= wf.frequency {
                wf.base + wf.amplitude
            } else {
                wf.base
            }
        }
        _ => {
            let table = TableForFunc(wf.func, assets, shader_name);
            WAVEVALUE(
                table,
                wf.base,
                wf.amplitude,
                wf.phase,
                wf.frequency,
                shader_time,
            )
        }
    }
}

/// Raven `void RB_CalcDeformNormals( deformStage_t *ds )`.
///
/// `tess.xyz`/`tess.normal`/`tess.numVertexes` are `tess`-dissolved, threaded
/// as slices (`xyz` read-only, `normal` mutated in place); `tess.shaderTime`
/// collapses to a plain `f32`. `VectorNormalizeFast` is an inline header
/// helper with no existing equivalent (resolved call surface) — inlined via
/// `Q_rsqrt`, matching this file's established
/// `RB_CalcEnvironmentTexCoords`/`RB_CalcSpecularAlpha` pattern.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:161-185`
pub fn RB_CalcDeformNormals(
    ds: &deformStage_t,
    xyz: &[[f32; 4]],
    normal: &mut [[f32; 4]],
    shader_time: f32,
    noise: &NoiseState,
) {
    for (v, n) in xyz.iter().zip(normal.iter_mut()) {
        let scale = 0.98f32;
        let scale = R_NoiseGet4f(
            noise,
            v[0] * scale,
            v[1] * scale,
            v[2] * scale,
            shader_time * ds.deformationWave.frequency,
        );
        n[0] += ds.deformationWave.amplitude * scale;

        let scale = 0.98f32;
        let scale = R_NoiseGet4f(
            noise,
            100.0 + v[0] * scale,
            v[1] * scale,
            v[2] * scale,
            shader_time * ds.deformationWave.frequency,
        );
        n[1] += ds.deformationWave.amplitude * scale;

        let scale = 0.98f32;
        let scale = R_NoiseGet4f(
            noise,
            200.0 + v[0] * scale,
            v[1] * scale,
            v[2] * scale,
            shader_time * ds.deformationWave.frequency,
        );
        n[2] += ds.deformationWave.amplitude * scale;

        // VectorNormalizeFast( normal )
        let n3 = [n[0], n[1], n[2]];
        let ilength = Q_rsqrt(_DotProduct(n3, n3));
        n[0] *= ilength;
        n[1] *= ilength;
        n[2] *= ilength;
    }
}

/// Raven `void RB_CalcDiffuseEntityColor( unsigned char *colors )`.
///
/// PORT-NOTE: the oracle's `if ( !backEnd.currentEntity )` branch calls
/// `RB_CalcDiffuseColor(colors)` for the "error" fallback but has no
/// `return;` after it — control falls straight through into
/// `ent = backEnd.currentEntity;` and its unconditional dereferences
/// (`VectorCopy(ent->ambientLight, ...)` etc.) on a possibly-null `ent`, a
/// genuine Raven UB path (porting-rules §19). It is also unreachable under
/// any defined interpretation: `RB_CalcDiffuseColor`'s own port (this file,
/// above) already requires `ent: &RefEntity` by value, so the null branch
/// could never actually supply it. Picked defined behavior: `current_entity`
/// threads as `ent: &RefEntity` (always valid), matching this file's
/// established precedent for the identical unconditional-deref-without-guard
/// pattern (`RB_CalcDiffuseColor`/`RB_CalcDisintegrateColors` above); the dead
/// fallback branch is dropped.
///
/// `tess.xyz` (`v`) is walked but never dereferenced in the oracle loop body
/// (same finding as `RB_CalcDiffuseColor`), so it is not threaded as a
/// parameter — only `tess.normal`/output `colors` are live reads/writes.
/// `*(int *)&ambientLightInt` / `*(int *)&colors[i*4] = ambientLightInt` (a
/// pack into an `int` immediately unpacked back to bytes via a second
/// reinterpret-cast) collapses to a single owned `[u8; 4]` built directly in
/// byte order — the interior-safety law forbids both pointer casts, and
/// skipping the round-trip through `int` reproduces the same bytes on any
/// platform (no endianness dependency, since both casts targeted the same
/// byte order).
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:1399-1463`
pub fn RB_CalcDiffuseEntityColor(colors: &mut [[u8; 4]], normal: &[[f32; 4]], ent: &RefEntity) {
    let ambient_light = ent.ambient_light;
    let directed_light = ent.directed_light;
    let light_dir = ent.light_dir;

    let r = ent.shader_rgba[0] as f32 / 255.0;
    let g = ent.shader_rgba[1] as f32 / 255.0;
    let b = ent.shader_rgba[2] as f32 / 255.0;

    let ambient_light_int: [u8; 4] = [
        myftol(r * ambient_light[0]) as u8,
        myftol(g * ambient_light[1]) as u8,
        myftol(b * ambient_light[2]) as u8,
        ent.shader_rgba[3],
    ];

    for (n, c) in normal.iter().zip(colors.iter_mut()) {
        let n3 = [n[0], n[1], n[2]];
        let incoming = _DotProduct(n3, light_dir);
        if incoming <= 0.0 {
            *c = ambient_light_int;
            continue;
        }

        let mut j = ambient_light[0] + incoming * directed_light[0];
        if j > 255.0 {
            j = 255.0;
        }
        c[0] = myftol(j * r) as u8;

        let mut j = ambient_light[1] + incoming * directed_light[1];
        if j > 255.0 {
            j = 255.0;
        }
        c[1] = myftol(j * g) as u8;

        let mut j = ambient_light[2] + incoming * directed_light[2];
        if j > 255.0 {
            j = 255.0;
        }
        c[2] = myftol(j * b) as u8;

        c[3] = ent.shader_rgba[3];
    }
}

/// Raven `static float EvalWaveFormClamped( const waveForm_t *wf )`.
///
/// `EvalWaveForm`'s own dissolved params (`noise`/`refdef_time`/
/// `refdef_float_time`/`shader_time`/`assets`/`shader_name`) thread straight
/// through, same collapse this file already applies at every `EvalWaveForm`
/// call site.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:58-73`
fn EvalWaveFormClamped(
    wf: &waveForm_t,
    noise: &NoiseState,
    refdef_time: i32,
    refdef_float_time: f32,
    shader_time: f32,
    assets: &RenderAssets,
    shader_name: &str,
) -> f32 {
    let glow = EvalWaveForm(
        wf,
        noise,
        refdef_time,
        refdef_float_time,
        shader_time,
        assets,
        shader_name,
    );

    if glow < 0.0 {
        return 0.0;
    }

    if glow > 1.0 {
        return 1.0;
    }

    glow
}

/// Raven `void RB_CalcStretchTexCoords( const waveForm_t *wf, float *st )`.
///
/// Same uninitialized-`tmi.type`/`tmi.wave` handling as
/// `RB_CalcRotateTexCoords` (the oracle never sets either field and
/// `RB_CalcTransformTexCoords` never reads them; zeroed here rather than left
/// as observable UB). `EvalWaveForm`'s dissolved params thread straight
/// through, same as `EvalWaveFormClamped` above.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:78-94`
pub fn RB_CalcStretchTexCoords(
    wf: &waveForm_t,
    st: &mut [[f32; 2]],
    noise: &NoiseState,
    refdef_time: i32,
    refdef_float_time: f32,
    shader_time: f32,
    assets: &RenderAssets,
    shader_name: &str,
) {
    let p = 1.0f32
        / EvalWaveForm(
            wf,
            noise,
            refdef_time,
            refdef_float_time,
            shader_time,
            assets,
            shader_name,
        );

    let tmi = texModInfo_t {
        r#type: texMod_t::TMOD_NONE,
        wave: waveForm_t {
            func: genFunc_t::GF_NONE,
            base: 0.0,
            amplitude: 0.0,
            phase: 0.0,
            frequency: 0.0,
        },
        matrix: [[p, 0.0], [0.0, p]],
        translate: [0.5f32 - 0.5f32 * p, 0.5f32 - 0.5f32 * p],
    };

    RB_CalcTransformTexCoords(&tmi, st);
}

/// Raven `void RB_CalcDeformVertexes( deformStage_t *ds )`.
///
/// `tess.xyz`/`tess.normal`/`tess.numVertexes` are `tess`-dissolved
/// (R2 `## State ownership` row `tess`) — threaded as slices (`xyz` mutated
/// in place, `normal` read-only, same split `RB_CalcDeformNormals` uses);
/// `tess.shaderTime` collapses to a plain `f32`; `EvalWaveForm`'s/
/// `TableForFunc`'s dissolved params (`noise`/`refdef_time`/
/// `refdef_float_time`/`assets`/`shader_name`) thread straight through, same
/// as `RB_CalcMoveVertexes`.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:110-152`
pub fn RB_CalcDeformVertexes(
    ds: &deformStage_t,
    xyz: &mut [[f32; 4]],
    normal: &[[f32; 4]],
    noise: &NoiseState,
    refdef_time: i32,
    refdef_float_time: f32,
    shader_time: f32,
    assets: &RenderAssets,
    shader_name: &str,
) {
    if ds.deformationWave.frequency == 0.0 {
        let scale = EvalWaveForm(
            &ds.deformationWave,
            noise,
            refdef_time,
            refdef_float_time,
            shader_time,
            assets,
            shader_name,
        );

        for (v, n) in xyz.iter_mut().zip(normal.iter()) {
            let mut offset = [0.0f32; 3];
            _VectorScale([n[0], n[1], n[2]], scale, &mut offset);

            v[0] += offset[0];
            v[1] += offset[1];
            v[2] += offset[2];
        }
    } else {
        let table = TableForFunc(ds.deformationWave.func, assets, shader_name);

        for (v, n) in xyz.iter_mut().zip(normal.iter()) {
            let off = (v[0] + v[1] + v[2]) * ds.deformationSpread;

            let scale = WAVEVALUE(
                table,
                ds.deformationWave.base,
                ds.deformationWave.amplitude,
                ds.deformationWave.phase + off,
                ds.deformationWave.frequency,
                shader_time,
            );

            let mut offset = [0.0f32; 3];
            _VectorScale([n[0], n[1], n[2]], scale, &mut offset);

            v[0] += offset[0];
            v[1] += offset[1];
            v[2] += offset[2];
        }
    }
}

/// Raven `void RB_CalcWaveColor( const waveForm_t *wf, unsigned char *dstColors )`.
///
/// `tess.shaderTime` collapses to a plain `f32`; `tess.numVertexes` is
/// `dst_colors.len()`. `tr.identityLight` is R2 `## State ownership`
/// row-`tr`-SPLIT frontend scratch, homed on `FrameState::identity_light`
/// (`crate::render_state::frame_state`, landed by the `tr_light` R3 wave) —
/// collapsed to a plain `f32` parameter, matching this file's established
/// scalar-collapse pattern for every other dissolved single-field read
/// (`refdef_time` etc.) rather than threading the whole struct.
/// `int *colors = (int *)dstColors` / `*(int *)color` (pack-then-broadcast
/// through a raw `int` reinterpretation) collapses to a single owned
/// `[u8; 4]` broadcast directly — the interior-safety law forbids the
/// pointer casts, and skipping the round-trip through `int` reproduces the
/// same bytes (no endianness dependency: both casts target the same byte
/// order), same reasoning as `RB_CalcDiffuseEntityColor` above.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:817-847`
pub fn RB_CalcWaveColor(
    wf: &waveForm_t,
    dst_colors: &mut [[u8; 4]],
    noise: &NoiseState,
    refdef_time: i32,
    refdef_float_time: f32,
    shader_time: f32,
    identity_light: f32,
    assets: &RenderAssets,
    shader_name: &str,
) {
    let mut glow = match wf.func {
        genFunc_t::GF_NOISE => {
            wf.base
                + R_NoiseGet4f(
                    noise,
                    0.0,
                    0.0,
                    0.0,
                    (shader_time + wf.phase) * wf.frequency,
                ) * wf.amplitude
        }
        _ => {
            EvalWaveForm(
                wf,
                noise,
                refdef_time,
                refdef_float_time,
                shader_time,
                assets,
                shader_name,
            ) * identity_light
        }
    };

    if glow < 0.0 {
        glow = 0.0;
    } else if glow > 1.0 {
        glow = 1.0;
    }

    let v = myftol(255.0 * glow) as u8;
    let color: [u8; 4] = [v, v, v, 255];

    for c in dst_colors.iter_mut() {
        *c = color;
    }
}

/// Raven `void RB_CalcWaveAlpha( const waveForm_t *wf, unsigned char *dstColors )`.
///
/// `tess.numVertexes` collapses to `dst_colors.len()`; `EvalWaveFormClamped`'s
/// dissolved params thread straight through, same as this file's other
/// `EvalWaveForm`/`EvalWaveFormClamped` call sites (`RB_CalcStretchTexCoords`
/// etc.). `v = 255 * glow` assigns a `float` into `int v` — a C truncating
/// (toward-zero) conversion, not `myftol`'s FISTP rounding, so `as i32`
/// matches; the loop only ever writes the low byte (`dstColors[3] = v`), so
/// the intermediate `i32` narrows via `as u8` to reproduce the same 8-bit
/// truncation as the oracle's implicit `unsigned char = int` store.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:871-885`
pub fn RB_CalcWaveAlpha(
    wf: &waveForm_t,
    dst_colors: &mut [[u8; 4]],
    noise: &NoiseState,
    refdef_time: i32,
    refdef_float_time: f32,
    shader_time: f32,
    assets: &RenderAssets,
    shader_name: &str,
) {
    let glow = EvalWaveFormClamped(
        wf,
        noise,
        refdef_time,
        refdef_float_time,
        shader_time,
        assets,
        shader_name,
    );

    let v = (255.0 * glow) as i32;

    for c in dst_colors.iter_mut() {
        c[3] = v as u8;
    }
}

/// Raven `void DeformText( const char *text )`.
///
/// `tess.normal[0]` collapses to the single `normal0` vec3 param (only index
/// 0 is ever read); `tess.xyz` collapses to the fixed `xyz` array — the
/// oracle always reads exactly indices 0-3 (`for ( i = 0 ; i < 4 ; i++ )`),
/// so a `[[f32; 4]; 4]`-shaped param carries the same invariant a
/// bounds-check on a slice would, without needing one. `strlen` is `text.len()` (Latin-1
/// byte-seam discipline, translation dictionary); the character loop walks
/// `text.bytes()` directly — `ch &= 255` is a no-op once each element is
/// already a `u8`.
///
/// DEFERRED: R4 — `tess.numIndexes = 0; tess.numVertexes = 0;` ("clear the
/// shader indexes") has no R3 target: no `tess` carrier exists. `tess` is
/// DISSOLVED into R4's tessellation/vertex-building pipeline with no
/// replacement scratch carrier (R2 `## State ownership` row `tess`; packet
/// STATE HOMES row `DeformText`), so there are no counters to zero here.
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:335-337`
///
/// Panics via `RB_AddQuadStampExt`'s loud stub until its owning wave lands.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:295-361`
pub fn DeformText(text: &str, normal0: vec3_t, xyz: [[f32; 4]; 4], frame: &mut FrameState) {
    let mut height: vec3_t = [0.0, 0.0, -1.0];
    let mut width: vec3_t = [0.0; 3];
    CrossProduct(normal0, height, &mut width);

    // find the midpoint of the box
    let mut mid: vec3_t = [0.0; 3];
    VectorClear(&mut mid);
    let mut bottom = 999999.0f32;
    let mut top = -999999.0f32;
    for vert in xyz.iter() {
        let v3 = [vert[0], vert[1], vert[2]];
        let mut next_mid = [0.0; 3];
        _VectorAdd(v3, mid, &mut next_mid);
        mid = next_mid;
        if vert[2] < bottom {
            bottom = vert[2];
        }
        if vert[2] > top {
            top = vert[2];
        }
    }
    let mut origin: vec3_t = [0.0; 3];
    _VectorScale(mid, 0.25, &mut origin);

    // determine the individual character size
    height[0] = 0.0;
    height[1] = 0.0;
    height[2] = (top - bottom) * 0.5;

    let mut scaled_width: vec3_t = [0.0; 3];
    _VectorScale(width, height[2] * -0.75, &mut scaled_width);
    width = scaled_width;

    // determine the starting position
    let len = text.len() as i32;
    let mut ma_origin: vec3_t = [0.0; 3];
    _VectorMA(origin, (len - 1) as f32, width, &mut ma_origin);
    origin = ma_origin;

    // clear the shader indexes
    // PORT-NOTE: dropped — see fn doc comment (`tess` dissolution).

    let color: [u8; 4] = [255, 255, 255, 255];

    // draw each character
    for ch in text.bytes() {
        if ch != b' ' {
            let row = ch >> 4;
            let col = ch & 15;

            let frow = row as f32 * 0.0625;
            let fcol = col as f32 * 0.0625;
            let size = 0.0625;

            RB_AddQuadStampExt(
                origin,
                width,
                height,
                color,
                fcol,
                frow,
                fcol + size,
                frow + size,
                frame,
            );
        }
        let mut next_origin: vec3_t = [0.0; 3];
        _VectorMA(origin, -2.0, width, &mut next_origin);
        origin = next_origin;
    }
}

/// Raven `static void AutospriteDeform( void )`.
///
/// `tess.numVertexes`/`tess.numIndexes`/`tess.xyz`/`tess.vertexColors` are
/// `tess`-dissolved (R2 `## State ownership` row `tess`) — `xyz`/
/// `vertex_colors` thread as slices sized to the oracle's `oldVerts` snapshot
/// (the old vertex count the loop walks before rebuilding via
/// `RB_AddQuadStamp`); `tess.numIndexes` collapses to the explicit
/// `num_indexes` scalar, same scalar-collapse pattern this file already
/// applies to every other single dissolved-field diagnostic read (compare
/// `shader_time`/`refdef_time`); `tess.shader->name` collapses to
/// `shader_name`, same as `RB_CalcBulgeVertexes`/`RB_CalcMoveVertexes`.
///
/// `backEnd.currentEntity != &tr.worldEntity` has no home on the owned
/// `RefEntity`/`FrameState` yet (the world-entity sentinel isn't modeled), so
/// the caller resolves the comparison and threads `is_world_entity` — same
/// collapse `Autosprite2Deform` (this file) already applies. `backEnd.ori`/
/// `backEnd.viewParms.ori` thread as the already-real `orientationr_t`,
/// matching `Autosprite2Deform`'s `ori`/`view_ori` split;
/// `backEnd.viewParms.isMirror` collapses to the `is_mirror` bool (`ViewParms`
/// is still an empty placeholder — same reasoning this file's header comment
/// gives for threading `ori` directly instead of the empty `OrientationR`).
/// `backEnd.currentEntity->e.nonNormalizedAxes` has no home on `RefEntity`
/// yet (not among the fields an earlier wave landed, and this wave may not
/// extend `placeholders.rs`) — threaded as the explicit `non_normalized_axes`
/// bool, same "field not yet landed" pattern this file already uses for
/// `ent->ambientLightInt` (`RB_CalcDiffuseColor`); `.e.axis[0]` IS already on
/// `RefEntity`, so `current_entity` threads through for that read.
///
/// DEFERRED: R4 — `tess.numVertexes = 0; tess.numIndexes = 0;` (the reset
/// before rebuilding via `RB_AddQuadStamp`) has no R3 target: no `tess`
/// carrier exists, same finding as `DeformText`'s identical reset (this
/// file, `oracle/codemp/renderer/tr_shade_calc.cpp:335-337`).
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:398-400`
///
/// Panics via `RB_AddQuadStamp`'s callee `RB_AddQuadStampExt`'s loud stub
/// until its owning wave lands.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:382-443`
pub fn AutospriteDeform(
    xyz: &[[f32; 4]],
    vertex_colors: &[[u8; 4]],
    num_indexes: i32,
    shader_name: &str,
    is_world_entity: bool,
    current_entity: &RefEntity,
    non_normalized_axes: bool,
    is_mirror: bool,
    ori: &orientationr_t,
    view_ori: &orientationr_t,
    common: &mut Common,
    frame: &mut FrameState,
) {
    let old_verts = xyz.len();

    if old_verts & 3 != 0 {
        com_printf(
            common,
            &format!(
                "{}Autosprite shader {} had odd vertex count",
                S_COLOR_YELLOW.to_str().expect("S_COLOR_YELLOW is ASCII"),
                shader_name
            ),
        );
    }
    if num_indexes != (old_verts as i32 >> 2) * 6 {
        com_printf(
            common,
            &format!(
                "{}Autosprite shader {} had odd index count",
                S_COLOR_YELLOW.to_str().expect("S_COLOR_YELLOW is ASCII"),
                shader_name
            ),
        );
    }

    // PORT-NOTE: `oldVerts = tess.numVertexes; tess.numVertexes = 0;
    // tess.numIndexes = 0;` dropped — see fn doc comment (`tess` dissolution,
    // same finding as `DeformText`'s reset).

    let (left_dir, up_dir) = if is_world_entity {
        (view_ori.axis[1], view_ori.axis[2])
    } else {
        (
            GlobalVectorToLocal(view_ori.axis[1], ori),
            GlobalVectorToLocal(view_ori.axis[2], ori),
        )
    };

    let mut i = 0usize;
    while i < old_verts {
        // find the midpoint
        let mut mid: vec3_t = [0.0; 3];
        mid[0] = 0.25 * (xyz[i][0] + xyz[i + 1][0] + xyz[i + 2][0] + xyz[i + 3][0]);
        mid[1] = 0.25 * (xyz[i][1] + xyz[i + 1][1] + xyz[i + 2][1] + xyz[i + 3][1]);
        mid[2] = 0.25 * (xyz[i][2] + xyz[i + 1][2] + xyz[i + 2][2] + xyz[i + 3][2]);

        let mut delta: vec3_t = [0.0; 3];
        _VectorSubtract([xyz[i][0], xyz[i][1], xyz[i][2]], mid, &mut delta);
        // / sqrt(2)
        let radius = VectorLength(delta) * 0.707f32;

        let mut left: vec3_t = [0.0; 3];
        _VectorScale(left_dir, radius, &mut left);
        let mut up: vec3_t = [0.0; 3];
        _VectorScale(up_dir, radius, &mut up);

        if is_mirror {
            let mut mirrored_left: vec3_t = [0.0; 3];
            _VectorSubtract(vec3_origin, left, &mut mirrored_left);
            left = mirrored_left;
        }

        // compensate for scale in the axes if necessary
        if non_normalized_axes {
            let axis_length = VectorLength(current_entity.axis[0]);
            let axis_length = if axis_length == 0.0 {
                0.0
            } else {
                1.0f32 / axis_length
            };

            let mut scaled_left: vec3_t = [0.0; 3];
            _VectorScale(left, axis_length, &mut scaled_left);
            left = scaled_left;

            let mut scaled_up: vec3_t = [0.0; 3];
            _VectorScale(up, axis_length, &mut scaled_up);
            up = scaled_up;
        }

        RB_AddQuadStamp(mid, left, up, vertex_colors[i], frame);

        i += 4;
    }
}

/// Raven `void RB_DeformTessGeometry( void )` — walks the current shader's
/// deform-stage list, dispatching each stage to its calc/deform fn.
///
/// `tess.shader->numDeforms`/`tess.shader->deforms` are `tess`-dissolved
/// (R2 `## State ownership` row `tess`) — the caller passes the stage list
/// directly as `deforms` (`numDeforms` collapses to `deforms.len()`, i.e. the
/// loop bound), the same tess-dissolved-into-slice-parameter pattern this
/// file already applies everywhere else. `tess.xyz`/`tess.normal` collapse to
/// the `xyz`/`normal` slices every branch below shares (mutability per branch
/// matches each already-ported callee's own settled signature — read-only
/// borrows are taken via `&*xyz`/`&*normal` reborrows, matched to the four
/// callees that don't mutate them).
///
/// `backEnd.refdef.text[ds->deformation - DEFORM_TEXT0]` — `TrRefdef::text`
/// hasn't landed yet (its doc comment: "lands with the `tr_scene` R3 wave"),
/// so the 8 render strings thread as the explicit `refdef_text` slice, same
/// "field not yet landed → explicit parameter" pattern this file already uses
/// for `ambientLightInt`/`nonNormalizedAxes`. `deform_t` derives no `Copy`
/// (out of scope — this packet may touch only this file), so the switch on
/// `ds->deformation` matches on `&ds.deformation` via match ergonomics
/// (avoids moving out of the borrow) and the oracle's
/// `ds->deformation - DEFORM_TEXT0` index arithmetic on the enum becomes 8
/// explicit `DEFORM_TEXT0..=DEFORM_TEXT7` arms with a literal index each,
/// rather than an enum-to-int cast.
///
/// `current_entity`/`is_world_entity`/`non_normalized_axes`/`is_mirror`/
/// `ori`/`view_ori` thread straight through to `AutospriteDeform`/
/// `Autosprite2Deform`, matching those fns' own already-settled parameter
/// lists (this file, above); `common`/`frame`/`assets`/`noise`/`shader_name`/
/// `refdef_time`/`refdef_float_time`/`shader_time`/`tex_coords0`/`indexes`/
/// `vertex_colors`/`num_indexes` are the same dissolved-field collapses each
/// individual callee already established.
///
/// Panics via `RB_AddQuadStampExt`'s loud stub (through `AutospriteDeform`/
/// `DeformText`) until its owning wave lands, for any shader whose deform
/// list reaches `DEFORM_AUTOSPRITE`/`DEFORM_TEXT0..7`.
///
/// Source: `oracle/codemp/renderer/tr_shade_calc.cpp:571-614`
#[allow(clippy::too_many_arguments)]
pub fn RB_DeformTessGeometry(
    deforms: &[deformStage_t],
    xyz: &mut [[f32; 4]],
    normal: &mut [[f32; 4]],
    tex_coords0: &[[f32; 2]],
    indexes: &[i32],
    vertex_colors: &[[u8; 4]],
    num_indexes: i32,
    shader_name: &str,
    refdef_text: &[String],
    refdef_time: i32,
    refdef_float_time: f32,
    shader_time: f32,
    noise: &NoiseState,
    assets: &RenderAssets,
    is_world_entity: bool,
    current_entity: &RefEntity,
    non_normalized_axes: bool,
    is_mirror: bool,
    ori: &orientationr_t,
    view_ori: &orientationr_t,
    common: &mut Common,
    frame: &mut FrameState,
) {
    for ds in deforms {
        match &ds.deformation {
            deform_t::DEFORM_NONE => {}
            deform_t::DEFORM_NORMALS => {
                RB_CalcDeformNormals(ds, &*xyz, normal, shader_time, noise);
            }
            deform_t::DEFORM_WAVE => {
                RB_CalcDeformVertexes(
                    ds,
                    xyz,
                    &*normal,
                    noise,
                    refdef_time,
                    refdef_float_time,
                    shader_time,
                    assets,
                    shader_name,
                );
            }
            deform_t::DEFORM_BULGE => {
                RB_CalcBulgeVertexes(ds, xyz, &*normal, tex_coords0, refdef_time, assets);
            }
            deform_t::DEFORM_MOVE => {
                RB_CalcMoveVertexes(ds, xyz, shader_time, assets, shader_name);
            }
            deform_t::DEFORM_PROJECTION_SHADOW => {
                RB_ProjectionShadowDeform();
            }
            deform_t::DEFORM_AUTOSPRITE => {
                AutospriteDeform(
                    &*xyz,
                    vertex_colors,
                    num_indexes,
                    shader_name,
                    is_world_entity,
                    current_entity,
                    non_normalized_axes,
                    is_mirror,
                    ori,
                    view_ori,
                    common,
                    frame,
                );
            }
            deform_t::DEFORM_AUTOSPRITE2 => {
                Autosprite2Deform(
                    xyz,
                    indexes,
                    shader_name,
                    is_world_entity,
                    ori,
                    view_ori,
                    common,
                );
            }
            deform_t::DEFORM_TEXT0 => {
                DeformText(
                    &refdef_text[0],
                    [normal[0][0], normal[0][1], normal[0][2]],
                    [xyz[0], xyz[1], xyz[2], xyz[3]],
                    frame,
                );
            }
            deform_t::DEFORM_TEXT1 => {
                DeformText(
                    &refdef_text[1],
                    [normal[0][0], normal[0][1], normal[0][2]],
                    [xyz[0], xyz[1], xyz[2], xyz[3]],
                    frame,
                );
            }
            deform_t::DEFORM_TEXT2 => {
                DeformText(
                    &refdef_text[2],
                    [normal[0][0], normal[0][1], normal[0][2]],
                    [xyz[0], xyz[1], xyz[2], xyz[3]],
                    frame,
                );
            }
            deform_t::DEFORM_TEXT3 => {
                DeformText(
                    &refdef_text[3],
                    [normal[0][0], normal[0][1], normal[0][2]],
                    [xyz[0], xyz[1], xyz[2], xyz[3]],
                    frame,
                );
            }
            deform_t::DEFORM_TEXT4 => {
                DeformText(
                    &refdef_text[4],
                    [normal[0][0], normal[0][1], normal[0][2]],
                    [xyz[0], xyz[1], xyz[2], xyz[3]],
                    frame,
                );
            }
            deform_t::DEFORM_TEXT5 => {
                DeformText(
                    &refdef_text[5],
                    [normal[0][0], normal[0][1], normal[0][2]],
                    [xyz[0], xyz[1], xyz[2], xyz[3]],
                    frame,
                );
            }
            deform_t::DEFORM_TEXT6 => {
                DeformText(
                    &refdef_text[6],
                    [normal[0][0], normal[0][1], normal[0][2]],
                    [xyz[0], xyz[1], xyz[2], xyz[3]],
                    frame,
                );
            }
            deform_t::DEFORM_TEXT7 => {
                DeformText(
                    &refdef_text[7],
                    [normal[0][0], normal[0][1], normal[0][2]],
                    [xyz[0], xyz[1], xyz[2], xyz[3]],
                    frame,
                );
            }
        }
    }
}
