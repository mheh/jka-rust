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
    _DotProduct, _VectorCopy, _VectorMA, _VectorScale, _VectorSubtract, VectorClear, VectorLength,
    VectorNormalize, VectorNormalize2,
};
use mp_qshared::shared::vec3_t;

use crate::render_state::frame_state::FrameState;
use crate::render_state::placeholders::RefEntity;
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::tr_local::dlight_s::dlight_t;
use crate::tr_local::mgrid_t::mgrid_t;
use crate::tr_local::orientationr_t::orientationr_t;
use crate::tr_shade_calc::myftol;

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

/// Raven `RF_MINLIGHT` — allways have some light (viewmodel, some items).
///
/// Source: `oracle/codemp/cgame/tr_types.h:18`
const RF_MINLIGHT: i32 = 0x00001;

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

/// Raven `RDF_NOWORLDMODEL`. Value confirmed against the already-ported
/// `crates/mp/renderer/src/tr_main.rs`'s `RDF_NOWORLDMODEL` (same literal,
/// same `refdef_rdflags`-parameter threading pattern used below).
///
/// Source: `oracle/codemp/cgame/tr_types.h`
const RDF_NOWORLDMODEL: i32 = 1;

/// `FUNCTABLE_SIZE`.
///
/// Source: `oracle/codemp/renderer/tr_local.h:1247`
const FUNCTABLE_SIZE: usize = 1024;

/// `FUNCTABLE_MASK` (`FUNCTABLE_SIZE - 1`).
///
/// Source: `oracle/codemp/renderer/tr_local.h:1248`
const FUNCTABLE_MASK: usize = FUNCTABLE_SIZE - 1;

/// Raven `DLIGHT_AT_RADIUS`.
///
/// Raven: at the edge of a dlight's influence, this amount of light will be
/// added.
///
/// Source: `oracle/codemp/renderer/tr_light.cpp:8`
const DLIGHT_AT_RADIUS: i32 = 16;

/// Raven `DLIGHT_MINIMUM_RADIUS`.
///
/// Raven: never calculate a range less than this to prevent huge light
/// numbers.
///
/// Source: `oracle/codemp/renderer/tr_light.cpp:11`
const DLIGHT_MINIMUM_RADIUS: i32 = 16;

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

// R3 wave 1 (this wave's 3 fns) needs several fields this file cannot add to
// `RefEntity`/`FrameState` directly — those types live in `render_state/
// placeholders.rs`/`render_state/frame_state.rs`, out of this wave's reach
// (porting-rules process; only `tr_light.rs` may be touched this wave). Per
// this file's own wave-0 precedent (the block above this one lists fields
// wave 0 needed the same way, since landed at integration), the fields below
// are referenced as if real; the integrate phase's field-merge step adds
// them:
// - `RefEntity`: `axis: [vec3_t; 3]` (`e.axis`, `refEntity_t`), `ambient_light_int:
//   [u8; 4]` (`ambientLightInt`, byte-packed per-component rather than the
//   oracle's `(byte*)&int` cast — interior-safety law forbids the raw-pointer
//   reinterpretation), `need_dlights: bool` (`needDlights`), `dlight_bits: i32`
//   (`dlightBits`) — all `trRefEntity_t` fields,
//   `oracle/codemp/renderer/tr_local.h:94-106`.
// - `FrameState`: `identity_light: f32` (`tr.identityLight`),
//   `identity_light_byte: i32` (`tr.identityLightByte`) — both part of the
//   `## State ownership` "tr frontend scratch/counters" bucket,
//   `oracle/codemp/renderer/tr_local.h:1309-1423`.

/// Scoped-local stand-in for the fields of Raven `bmodel_t` that
/// `R_DlightBmodel` reads — this wave (`tr_light`) may not touch `tr_bsp.rs`/
/// `placeholders.rs`'s `WorldAsset` to grow a `bmodels` field. Mirrors
/// `tr_marks::MarkNode`'s established pattern for pre-`tr_bsp` BSP data, and
/// `tr_world::R_DlightFace`/`R_DlightGrid`'s `dlights: &[dlight_t]`
/// param-threading for the same "the real field doesn't exist yet" problem.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:938-942`
pub struct DlightBmodel {
    /// `bounds[2]` — for culling.
    pub bounds: [vec3_t; 2],
    /// `firstSurface[numSurfaces]`, walked as an owned `Vec` instead of a
    /// `msurface_t *` + count pair (C pointer-walk -> slice/Vec, dictionary).
    pub surfaces: Vec<DlightSurface>,
}

/// Scoped-local stand-in for the one field of Raven `msurface_t` (`data`)
/// that `R_DlightBmodel` writes through — see `DlightBmodel`.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:872-878`
pub struct DlightSurface {
    /// any of srf*_t
    pub data: DlightSurfaceData,
}

/// Owned replacement for `msurface_t.data`'s tagged `surfaceType_t *` union —
/// only the three kinds `R_DlightBmodel` writes `dlightBits` into
/// (`SF_FACE`/`SF_GRID`/`SF_TRIANGLES`, `surfaceType_t`
/// `oracle/codemp/renderer/tr_local.h:656-678`); every other kind collapses
/// to `Other`.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:799-812`
/// (`srfSurfaceFace_t`), `:750-774` (`srfGridMesh_t`), `:818-836`
/// (`srfTriangles_t`)
pub enum DlightSurfaceData {
    /// `srfSurfaceFace_t.dlightBits`.
    Face {
        dlightBits: i32,
    },
    /// `srfGridMesh_t.dlightBits`.
    Grid {
        dlightBits: i32,
    },
    /// `srfTriangles_t.dlightBits`.
    Triangles {
        dlightBits: i32,
    },
    Other,
}

/// Raven `R_DlightBmodel`. //rwwRMG - modified args
///
/// PORT-NOTE: `tr.refdef.num_dlights`/`dlights` and `tr.ori` (state home:
/// SPLIT — `## State ownership`'s "tr frontend scratch/counters" bucket,
/// still an empty `FrameState`/`orientationr_t` mismatch this file can't
/// resolve — see wave-0's already-ported `R_TransformDlights(dl: &mut
/// [dlight_t], ori: &orientationr_t)`, which established this exact
/// direct-parameter shape) are threaded in directly rather than reached
/// through `FrameState`. `tr.currentEntity` is threaded via `FrameState`
/// itself since `current_entity: Option<RefEntity>` is already real there.
///
/// Source: `oracle/codemp/renderer/tr_light.cpp:44-92`
pub fn R_DlightBmodel(
    bmodel: &mut DlightBmodel,
    no_light: bool,
    dl: &mut [dlight_t],
    ori: &orientationr_t,
    frame: &mut FrameState,
) {
    // transform all the lights
    R_TransformDlights(dl, ori);

    let mut mask: i32 = 0;
    if !no_light {
        for (i, light) in dl.iter().enumerate() {
            // see if the point is close enough to the bounds to matter
            let mut j = 3;
            for k in 0..3 {
                if light.transformed[k] - bmodel.bounds[1][k] > light.radius {
                    j = k;
                    break;
                }
                if bmodel.bounds[0][k] - light.transformed[k] > light.radius {
                    j = k;
                    break;
                }
            }
            if j < 3 {
                continue;
            }

            // we need to check this light
            mask |= 1 << i;
        }
    }

    let current_entity = frame
        .current_entity
        .as_mut()
        .expect("R_DlightBmodel: tr.currentEntity not set");
    current_entity.need_dlights = mask != 0;
    current_entity.dlight_bits = mask;

    // set the dlight bits in all the surfaces
    for surf in bmodel.surfaces.iter_mut() {
        match &mut surf.data {
            DlightSurfaceData::Face { dlightBits } => *dlightBits = mask,
            DlightSurfaceData::Grid { dlightBits } => *dlightBits = mask,
            DlightSurfaceData::Triangles { dlightBits } => *dlightBits = mask,
            DlightSurfaceData::Other => {}
        }
    }
}

/// Raven `R_SetupEntityLighting`.
///
/// PORT-NOTE: the oracle's `const trRefdef_t *refdef` parameter is read for
/// exactly two fields, neither of which `TrRefdef` carries yet
/// (`rdflags`/`dlights`+`num_dlights` land with a later `tr_scene` wave) —
/// threaded in directly as `refdef_rdflags`/`dlights`, mirroring
/// `tr_main::SetFarClip`'s `refdef_rdflags: i32` param and
/// `tr_world::R_DlightFace`'s `dlights: &[dlight_t]` param (porting-rules
/// §4, "state is threaded, not reached" — the same rationale, not a whole
/// `TrRefdef` reference for two missing fields).
///
/// Source: `oracle/codemp/renderer/tr_light.cpp:350-460`
pub fn R_SetupEntityLighting(
    common: &mut Common,
    cvars: &RendererCvars,
    assets: &RenderAssets,
    frame: &FrameState,
    refdef_rdflags: i32,
    dlights: &[dlight_t],
    ent: &mut RefEntity,
) {
    // lighting calculations
    if ent.lighting_calculated {
        return;
    }
    ent.lighting_calculated = true;

    //
    // trace a sample point down to find ambient light
    //
    let light_origin = if ent.renderfx & RF_LIGHTING_ORIGIN != 0 {
        // seperate lightOrigins are needed so an object that is
        // sinking into the ground can still be lit, and so
        // multi-part models can be lit identically
        ent.lighting_origin
    } else {
        ent.origin
    };

    // if NOWORLDMODEL, only use dynamic lights (menu system, etc)
    let has_light_grid = assets
        .world
        .as_ref()
        .map(|w| w.light_grid_data.is_some())
        .unwrap_or(false);
    if refdef_rdflags & RDF_NOWORLDMODEL == 0 && has_light_grid {
        R_SetupEntityLightingGrid(common, cvars, assets, frame, ent);
    } else {
        ent.ambient_light = [
            frame.identity_light * 150.0,
            frame.identity_light * 150.0,
            frame.identity_light * 150.0,
        ];
        ent.directed_light = ent.ambient_light;
        _VectorCopy(frame.sun_direction, &mut ent.light_dir);
    }

    // bonus items and view weapons have a fixed minimum add
    // PORT-NOTE: retail's guard reads `if ( 1 /* ent->e.renderfx &
    // RF_MINLIGHT */ )` — the bitwise test is commented out and replaced
    // with the literal `1`, so this branch is unconditional in the shipped
    // binary; transcribed as such (porting-rules §2 — port what actually
    // executes, not the disabled comment).
    // give everything a minimum light add
    ent.ambient_light[0] += frame.identity_light * 32.0;
    ent.ambient_light[1] += frame.identity_light * 32.0;
    ent.ambient_light[2] += frame.identity_light * 32.0;

    // the minlight flag is now for items rotating on their holo thing
    if ent.renderfx & RF_MINLIGHT != 0 {
        if ent.shader_rgba[0] == 255 && ent.shader_rgba[1] == 255 && ent.shader_rgba[2] == 0 {
            ent.ambient_light[0] += frame.identity_light * 255.0;
            ent.ambient_light[1] += frame.identity_light * 255.0;
            ent.ambient_light[2] += frame.identity_light * 0.0;
        } else {
            ent.ambient_light[0] += frame.identity_light * 16.0;
            ent.ambient_light[1] += frame.identity_light * 96.0;
            ent.ambient_light[2] += frame.identity_light * 150.0;
        }
    }

    //
    // modify the light by dynamic lights
    //
    let d = VectorLength(ent.directed_light);
    let mut light_dir: vec3_t = [0.0; 3];
    _VectorScale(ent.light_dir, d, &mut light_dir);

    for dl in dlights.iter() {
        let mut dir: vec3_t = [0.0; 3];
        _VectorSubtract(dl.origin, light_origin, &mut dir);
        let mut d = VectorNormalize(&mut dir);

        let power = DLIGHT_AT_RADIUS as f32 * (dl.radius * dl.radius);
        if d < DLIGHT_MINIMUM_RADIUS as f32 {
            d = DLIGHT_MINIMUM_RADIUS as f32;
        }
        d = power / (d * d);

        let mut new_directed: vec3_t = [0.0; 3];
        _VectorMA(ent.directed_light, d, dl.color, &mut new_directed);
        ent.directed_light = new_directed;

        let mut new_light_dir: vec3_t = [0.0; 3];
        _VectorMA(light_dir, d, dir, &mut new_light_dir);
        light_dir = new_light_dir;
    }

    // clamp ambient
    for i in 0..3 {
        if ent.ambient_light[i] > frame.identity_light_byte as f32 {
            ent.ambient_light[i] = frame.identity_light_byte as f32;
        }
    }

    if common.cvar(cvars.r_debugLight).integer != 0 {
        LogLight(common, ent);
    }

    // save out the byte packet version
    // PORT-NOTE: retail writes through `((byte *)&ent->ambientLightInt)[N]`
    // — a raw reinterpret-cast the interior-safety law forbids; `RefEntity`
    // carries the packed color as an owned `[u8; 4]` instead (matching
    // `FrameState::color_2d`'s established `color4ub_t` shape), each
    // component assigned directly rather than through a byte-pointer alias.
    ent.ambient_light_int[0] = myftol(ent.ambient_light[0]) as u8;
    ent.ambient_light_int[1] = myftol(ent.ambient_light[1]) as u8;
    ent.ambient_light_int[2] = myftol(ent.ambient_light[2]) as u8;
    ent.ambient_light_int[3] = 0xff;

    // transform the direction to local space
    let mut normalized_dir: vec3_t = [0.0; 3];
    VectorNormalize2(light_dir, &mut normalized_dir);
    ent.light_dir[0] = _DotProduct(normalized_dir, ent.axis[0]);
    ent.light_dir[1] = _DotProduct(normalized_dir, ent.axis[1]);
    ent.light_dir[2] = _DotProduct(normalized_dir, ent.axis[2]);
}

/// Raven `R_LightForPoint`. The four out-params (a `qboolean` return plus
/// three `vec3_t` out-params) collapse into a single `Option<(vec3_t,
/// vec3_t, vec3_t)>`: `Some((ambientLight, directedLight, lightDir))` on the
/// oracle's `qtrue` path, `None` on its `qfalse` early-out (dictionary:
/// qboolean + out-params -> return value).
///
/// PORT-NOTE: retail dereferences `tr.world` unconditionally
/// (`tr.world->lightGridData`, no null check on `tr.world` itself) — a null
/// `tr.world` would be Raven UB (porting-rules §19); this port treats an
/// absent world the same as an absent light grid (`None`, the `qfalse`
/// path), the one defined behavior, rather than reproducing the crash.
/// Retail's local zero-inited `trRefEntity_t ent` (`Com_Memset` +
/// `VectorCopy` into `.e.origin`) becomes `RefEntity::default()` +
/// `ent.origin = point` — `Com_Memset`'s raw-pointer signature is tier-1-only
/// (interior-safety law), so the idiomatic default-construction stands in
/// for it here (ruling 1: the renderer interior is oracle-match-free).
///
/// Source: `oracle/codemp/renderer/tr_light.cpp:467-483`
pub fn R_LightForPoint(
    common: &Common,
    cvars: &RendererCvars,
    assets: &RenderAssets,
    frame: &FrameState,
    point: vec3_t,
) -> Option<(vec3_t, vec3_t, vec3_t)> {
    // bk010103 - this segfaults with -nolight maps
    let has_light_grid = assets
        .world
        .as_ref()
        .map(|w| w.light_grid_data.is_some())
        .unwrap_or(false);
    if !has_light_grid {
        return None;
    }

    let mut ent = RefEntity::default();
    ent.origin = point;
    R_SetupEntityLightingGrid(common, cvars, assets, frame, &mut ent);

    Some((ent.ambient_light, ent.directed_light, ent.light_dir))
}
