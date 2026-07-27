//! Raven `tr_light.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_light.cpp`

// Raven-named functions/types keep their original casing across this
// transcription, matching the rest of the renderer/engine crates.
#![allow(non_snake_case)]
// Wave-0 ports of Raven `static` helpers: private by fidelity, with their
// callers landing in later R3 waves.
#![allow(dead_code)]

use mp_engine_qcommon::common::{com_printf, Common};
use mp_engine_qcommon::qfiles::draw_vert_t::MAXLIGHTMAPS;
use mp_engine_qcommon::qfiles::light_style_limits::LS_LSNONE;
use mp_qshared::shared::q_math::{
    _DotProduct, _VectorCopy, _VectorMA, _VectorScale, _VectorSubtract, VectorClear,
    VectorNormalize2,
};
use mp_qshared::shared::vec3_t;

use crate::render_state::frame_state::FrameState;
use crate::render_state::placeholders::RefEntity;
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::tr_local::dlight_s::dlight_t;
use crate::tr_local::mgrid_t::mgrid_t;
use crate::tr_local::orientationr_t::orientationr_t;

// This wave threads `RenderAssets`, `FrameState`, `RendererCvars` (the R2
// STATE HOMES rows for `R_SetupEntityLightingGrid`) and `RefEntity`
// (`crate::render_state::placeholders`) as the fns below expect them. All
// are still skeleton stubs owned by other waves this wave may not touch
// (porting-rules process); the fields this file's stateful fn reads/writes
// land on those structs with the field-merge step of this wave's
// integration:
// - `RenderAssets::function_tables` (`FunctionTables`, owned by the
//   `tr_init` R3 wave): `sin_table: [f32; FUNCTABLE_SIZE]` (`tr.sinTable`).
// - `RenderAssets::world` (`WorldAsset`, owned by the `tr_bsp` R3 wave):
//   `light_grid_origin`/`light_grid_inverse_size: vec3_t`,
//   `light_grid_bounds: [i32; 3]`, `num_grid_array_elements: i32`,
//   `light_grid_array: Vec<u16>` (already assumed by `tr_bsp.rs`),
//   `light_grid_data: Option<Vec<mgrid_t>>`. PORT-NOTE: `tr_bsp.rs`'s own
//   field-needs comment guesses `Option<Vec<u8>>` for `light_grid_data` —
//   it never actually reads a record through that field, only writes `None`
//   on a lump-size mismatch. This fn needs typed, randomly-indexed
//   `mgrid_t` records (matching the oracle's `mgrid_t *lightGridData`
//   directly, not a raw byte buffer), so `Vec<mgrid_t>` is the shape
//   carried here — flagged for the integrate phase's field-merge step to
//   reconcile the two.
// - `FrameState`: `sun_direction: vec3_t` (`tr.sunDirection`, part of the R2
//   `tr` SPLIT row's "sun/fog fields" bucket).
// - `RefEntity` (owned by the `tr_scene` R3 wave): `renderfx: i32`,
//   `origin`/`lighting_origin`/`ambient_light`/`directed_light`/
//   `light_dir: vec3_t` — the subset of `refEntity_t`/`trRefEntity_t` this
//   file touches, flattened directly onto `RefEntity` rather than nested
//   under an `.e` sub-field (no other wave has established that nesting
//   yet).

/// Raven `RF_FIRST_PERSON` — only draw through eyes (view weapon, damage
/// blood blob).
///
/// Source: `oracle/codemp/cgame/tr_types.h:20`
const RF_FIRST_PERSON: i32 = 0x00004;

/// Raven `RF_LIGHTING_ORIGIN` — use `refEntity->lightingOrigin` instead of
/// `refEntity->origin` for lighting.
///
/// Source: `oracle/codemp/cgame/tr_types.h:28`
const RF_LIGHTING_ORIGIN: i32 = 0x00080;

/// `FUNCTABLE_SIZE`.
///
/// Source: `oracle/codemp/renderer/tr_local.h:1247`
const FUNCTABLE_SIZE: usize = 1024;

/// `FUNCTABLE_MASK` (`FUNCTABLE_SIZE - 1`).
///
/// Source: `oracle/codemp/renderer/tr_local.h:1248`
const FUNCTABLE_MASK: usize = FUNCTABLE_SIZE - 1;

/// Raven `R_TransformDlights`. The oracle's `int count, dlight_t *dl` pair
/// collapses to a slice (out-params→returns / C pointer-walk→slice
/// dictionary entries).
///
/// Source: `oracle/codemp/renderer/tr_light.cpp:24-34`
pub fn R_TransformDlights(dl: &mut [dlight_t], ori: &orientationr_t) {
    for light in dl.iter_mut() {
        let mut temp: vec3_t = [0.0; 3];
        _VectorSubtract(light.origin, ori.origin, &mut temp);
        light.transformed[0] = _DotProduct(temp, ori.axis[0]);
        light.transformed[1] = _DotProduct(temp, ori.axis[1]);
        light.transformed[2] = _DotProduct(temp, ori.axis[2]);
    }
}

/// Raven `R_SetupEntityLightingGrid`.
///
/// Source: `oracle/codemp/renderer/tr_light.cpp:119-310`
fn R_SetupEntityLightingGrid(
    common: &Common,
    cvars: &RendererCvars,
    assets: &RenderAssets,
    frame: &FrameState,
    ent: &mut RefEntity,
) {
    if common.cvar(cvars.r_fullbright).integer != 0 {
        ent.ambient_light = [255.0, 255.0, 255.0];
        ent.directed_light = [255.0, 255.0, 255.0];
        _VectorCopy(frame.sun_direction, &mut ent.light_dir);
        return;
    }

    let mut light_origin: vec3_t;
    if ent.renderfx & RF_LIGHTING_ORIGIN != 0 {
        // seperate lightOrigins are needed so an object that is
        // sinking into the ground can still be lit, and so
        // multi-part models can be lit identically
        light_origin = ent.lighting_origin;
    } else {
        light_origin = ent.origin;
    }

    let world = assets
        .world
        .as_ref()
        .expect("R_SetupEntityLightingGrid: tr.world not loaded");

    let mut relative = [0.0f32; 3];
    _VectorSubtract(light_origin, world.light_grid_origin, &mut relative);
    light_origin = relative;

    let mut pos = [0i32; 3];
    let mut frac = [0.0f32; 3];
    for i in 0..3 {
        let v = light_origin[i] * world.light_grid_inverse_size[i];
        let mut p = v.floor() as i32;
        frac[i] = v - p as f32;
        if p < 0 {
            p = 0;
        } else if p >= world.light_grid_bounds[i] - 1 {
            p = world.light_grid_bounds[i] - 1;
        }
        pos[i] = p;
    }

    VectorClear(&mut ent.ambient_light);
    VectorClear(&mut ent.directed_light);
    let mut direction: vec3_t = [0.0; 3];

    // trilerp the light value
    let grid_step = [
        1,
        world.light_grid_bounds[0],
        world.light_grid_bounds[0] * world.light_grid_bounds[1],
    ];
    let start_grid_pos = pos[0] * grid_step[0] + pos[1] * grid_step[1] + pos[2] * grid_step[2];

    let mut total_factor = 0.0f32;
    for i in 0..8i32 {
        let mut factor = 1.0f32;
        let mut grid_pos = start_grid_pos;
        for j in 0..3usize {
            if i & (1 << j) != 0 {
                factor *= frac[j];
                grid_pos += grid_step[j];
            } else {
                factor *= 1.0 - frac[j];
            }
        }

        if grid_pos < 0 || grid_pos >= world.num_grid_array_elements {
            // we've gone off the array somehow
            continue;
        }

        let index = match world.light_grid_array.get(grid_pos as usize) {
            Some(v) => *v,
            None => continue,
        };

        let data: &mgrid_t = match world
            .light_grid_data
            .as_ref()
            .and_then(|d| d.get(index as usize))
        {
            Some(d) => d,
            None => continue,
        };

        if data.styles[0] == LS_LSNONE {
            // ignore samples in walls
            continue;
        }

        total_factor += factor;

        for j in 0..MAXLIGHTMAPS {
            if data.styles[j] != LS_LSNONE {
                let style = data.styles[j] as usize;

                ent.ambient_light[0] += factor
                    * data.ambientLight[j][0] as f32
                    * frame.scene_light_styles[style][0] as f32
                    / 255.0;
                ent.ambient_light[1] += factor
                    * data.ambientLight[j][1] as f32
                    * frame.scene_light_styles[style][1] as f32
                    / 255.0;
                ent.ambient_light[2] += factor
                    * data.ambientLight[j][2] as f32
                    * frame.scene_light_styles[style][2] as f32
                    / 255.0;

                ent.directed_light[0] += factor
                    * data.directLight[j][0] as f32
                    * frame.scene_light_styles[style][0] as f32
                    / 255.0;
                ent.directed_light[1] += factor
                    * data.directLight[j][1] as f32
                    * frame.scene_light_styles[style][1] as f32
                    / 255.0;
                ent.directed_light[2] += factor
                    * data.directLight[j][2] as f32
                    * frame.scene_light_styles[style][2] as f32
                    / 255.0;
            } else {
                break;
            }
        }

        let mut lat = data.latLong[1] as i32;
        let mut lng = data.latLong[0] as i32;
        lat *= (FUNCTABLE_SIZE / 256) as i32;
        lng *= (FUNCTABLE_SIZE / 256) as i32;

        // decode X as cos( lat ) * sin( long )
        // decode Y as sin( lat ) * sin( long )
        // decode Z as cos( long )
        let sin_table = &assets.function_tables.sin_table;
        let mut normal: vec3_t = [0.0; 3];
        normal[0] = sin_table[((lat + (FUNCTABLE_SIZE as i32 / 4)) as usize) & FUNCTABLE_MASK]
            * sin_table[lng as usize];
        normal[1] = sin_table[lat as usize] * sin_table[lng as usize];
        normal[2] = sin_table[((lng + (FUNCTABLE_SIZE as i32 / 4)) as usize) & FUNCTABLE_MASK];

        _VectorMA(direction, factor, normal, &mut direction);
    }

    if total_factor > 0.0 && total_factor < 0.99 {
        total_factor = 1.0 / total_factor;
        _VectorScale(ent.ambient_light, total_factor, &mut ent.ambient_light);
        _VectorScale(ent.directed_light, total_factor, &mut ent.directed_light);
    }

    _VectorScale(
        ent.ambient_light,
        common.cvar(cvars.r_ambientScale).value,
        &mut ent.ambient_light,
    );
    _VectorScale(
        ent.directed_light,
        common.cvar(cvars.r_directedScale).value,
        &mut ent.directed_light,
    );

    VectorNormalize2(direction, &mut ent.light_dir);
}

/// Raven `LogLight`.
///
/// Source: `oracle/codemp/renderer/tr_light.cpp:318-340`
fn LogLight(common: &mut Common, ent: &RefEntity) {
    if ent.renderfx & RF_FIRST_PERSON == 0 {
        return;
    }

    let mut max1 = ent.ambient_light[0] as i32;
    if ent.ambient_light[1] as i32 > max1 {
        max1 = ent.ambient_light[1] as i32;
    } else if ent.ambient_light[2] as i32 > max1 {
        max1 = ent.ambient_light[2] as i32;
    }

    let mut max2 = ent.directed_light[0] as i32;
    if ent.directed_light[1] as i32 > max2 {
        max2 = ent.directed_light[1] as i32;
    } else if ent.directed_light[2] as i32 > max2 {
        max2 = ent.directed_light[2] as i32;
    }

    com_printf(common, &format!("amb:{}  dir:{}\n", max1, max2));
}
