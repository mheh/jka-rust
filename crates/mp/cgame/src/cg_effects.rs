//! Port of `oracle/codemp/cgame/cg_effects.c` — client-side effect spawners (puffs, debris, glass, chunks). Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::{c_int, c_uint};

use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::common::mp::cgame::tr_types::RF_THIRD_PERSON;
use mp_qshared::common::mp::gentity::{
    material_t, MAT_CRATE1, MAT_CRATE2, MAT_DRK_STONE, MAT_ELECTRICAL, MAT_ELEC_METAL, MAT_GLASS,
    MAT_GLASS_METAL, MAT_GRATE1, MAT_GREY_STONE, MAT_LT_STONE, MAT_METAL, MAT_METAL2, MAT_METAL3,
    MAT_NONE, MAT_ROPE, MAT_SNOWY_ROCK, MAT_WHITE_METAL,
};
use mp_qshared::common::mp::qcommon::saber::saber_colors::{
    SABER_BLUE, SABER_GREEN, SABER_ORANGE, SABER_PURPLE, SABER_RED, SABER_YELLOW,
};
use mp_qshared::shared::q_math::{
    _VectorAdd, _VectorCopy, _VectorMA, _VectorScale, _VectorSubtract, AnglesToAxis, AxisClear,
    CrossProduct, DistanceSquared, Q_random, RotateAroundDirection, VectorClear, VectorLength,
    VectorNormalize, VectorSet, YAW,
};
use mp_qshared::shared::{
    addpolyArgStruct_t, qhandle_t, qtrue, trType_t, vec2_t, vec3_t, CHAN_AUTO, CHAN_BODY,
};

use crate::cg_ents::ScaleModelAxis;
use crate::cg_localents::CG_AllocLocalEntity;
use crate::cg_main::CG_Error;
use crate::cg_view::CGCam_Shake;
use crate::local::le_bounce_sound_type_t::leBounceSoundType_t;
use crate::local::le_flag_t::leFlag_t;
use crate::local::le_mark_type_t::leMarkType_t;
use crate::local::le_type_t::leType_t;
use crate::trap;
use crate::world::cg_context::CgContext;
use crate::world::cg_world::CgWorld;
use crate::world::effect_handle::EffectHandle;

// FILE-SCOPE CONSTANTS
// Source: `oracle/codemp/cgame/cg_effects.c:257-259,452-454,756-764,1384-1386`

/// Raven `FX_ALPHA_NONLINEAR` — `addpolyArgStruct_t.flags` bit: fade the alpha ramp non-linearly.
/// Source: `oracle/codemp/cgame/cg_effects.c:257`
pub const FX_ALPHA_NONLINEAR: c_int = 0x00000004;

/// Raven `FX_APPLY_PHYSICS` — `addpolyArgStruct_t.flags` bit: integrate `vel`/`accel`/bounce each frame.
/// Source: `oracle/codemp/cgame/cg_effects.c:258`
pub const FX_APPLY_PHYSICS: c_int = 0x02000000;

/// Raven `FX_USE_ALPHA` — `addpolyArgStruct_t.flags` bit: honor `alpha1`/`alpha2`/`alphaParm`.
/// Source: `oracle/codemp/cgame/cg_effects.c:259`
pub const FX_USE_ALPHA: c_int = 0x08000000;

/// Raven `TIME_DECAY_SLOW`.
/// Source: `oracle/codemp/cgame/cg_effects.c:452`
pub const TIME_DECAY_SLOW: f32 = 0.1;

/// Raven `TIME_DECAY_MED`.
/// Source: `oracle/codemp/cgame/cg_effects.c:453`
pub const TIME_DECAY_MED: f32 = 0.04;

/// Raven `TIME_DECAY_FAST`.
/// Source: `oracle/codemp/cgame/cg_effects.c:454`
pub const TIME_DECAY_FAST: f32 = 0.009;

/// Raven `DEBRIS_SPECIALCASE_ROCK` — negative sentinel debris-model index.
/// Source: `oracle/codemp/cgame/cg_effects.c:756`
pub const DEBRIS_SPECIALCASE_ROCK: c_int = -1;

/// Raven `DEBRIS_SPECIALCASE_CHUNKS` — negative sentinel debris-model index.
/// Source: `oracle/codemp/cgame/cg_effects.c:757`
pub const DEBRIS_SPECIALCASE_CHUNKS: c_int = -2;

/// Raven `DEBRIS_SPECIALCASE_WOOD` — negative sentinel debris-model index.
/// Source: `oracle/codemp/cgame/cg_effects.c:758`
pub const DEBRIS_SPECIALCASE_WOOD: c_int = -3;

/// Raven `DEBRIS_SPECIALCASE_GLASS` — negative sentinel debris-model index.
/// Source: `oracle/codemp/cgame/cg_effects.c:759`
pub const DEBRIS_SPECIALCASE_GLASS: c_int = -4;

/// Raven `NUM_DEBRIS_MODELS_GLASS`.
/// Source: `oracle/codemp/cgame/cg_effects.c:761`
pub const NUM_DEBRIS_MODELS_GLASS: usize = 8;

/// Raven `NUM_DEBRIS_MODELS_WOOD`.
/// Source: `oracle/codemp/cgame/cg_effects.c:762`
pub const NUM_DEBRIS_MODELS_WOOD: usize = 8;

/// Raven `NUM_DEBRIS_MODELS_CHUNKS`.
/// Source: `oracle/codemp/cgame/cg_effects.c:763`
pub const NUM_DEBRIS_MODELS_CHUNKS: usize = 3;

/// Raven `NUM_DEBRIS_MODELS_ROCKS` (Raven comment: `//12` — the table shipped
/// with 4 entries, not the originally planned 12).
/// Source: `oracle/codemp/cgame/cg_effects.c:764`
pub const NUM_DEBRIS_MODELS_ROCKS: usize = 4;

/// Raven `NUM_SPARKS`.
/// Source: `oracle/codemp/cgame/cg_effects.c:1384`
pub const NUM_SPARKS: usize = 12;

/// Raven `NUM_PUFFS`.
/// Source: `oracle/codemp/cgame/cg_effects.c:1385`
pub const NUM_PUFFS: usize = 1;

/// Raven `NUM_EXPLOSIONS`.
/// Source: `oracle/codemp/cgame/cg_effects.c:1386`
pub const NUM_EXPLOSIONS: usize = 4;

// Constants the fns below read that live outside `cg_effects.c` and have no
// ported cross-crate home yet, so they land beside their readers (the
// `RF_THIRD_PERSON` treatment in `cg_players.rs`).

/// Raven `vec3_t axisDefault[3]` — the identity basis every
/// `AxisCopy( axisDefault, … )` below copies. `q_math.c`'s global is not ported
/// anywhere in the tree; `cg_localents.c`'s `CG_AddFadeScaleModel` inlines the
/// same literal.
/// Source: `oracle/codemp/game/q_math.c:8`
const AXIS_DEFAULT: [vec3_t; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

// PORT-NOTE: `cg_local.h`'s chunk-type enum is anonymous, so per the
// anonymous-enum convention these are `const`s. They index the first axis of
// `cgs.media.chunkModels`. `cg_main.rs` carries a private copy of the same
// eight (it registers the models); neither file can see the other's, so both
// declare them until the header constants get a shared home.
/// Source: `oracle/codemp/cgame/cg_local.h:1048-1059`
const CHUNK_METAL1: usize = 0;
/// Source: `oracle/codemp/cgame/cg_local.h:1048-1059`
const CHUNK_METAL2: usize = 1;
/// Source: `oracle/codemp/cgame/cg_local.h:1048-1059`
const CHUNK_ROCK1: usize = 2;
/// Source: `oracle/codemp/cgame/cg_local.h:1048-1059`
const CHUNK_ROCK2: usize = 3;
/// Source: `oracle/codemp/cgame/cg_local.h:1048-1059`
const CHUNK_ROCK3: usize = 4;
/// Source: `oracle/codemp/cgame/cg_local.h:1048-1059`
const CHUNK_CRATE1: usize = 5;
/// Source: `oracle/codemp/cgame/cg_local.h:1048-1059`
const CHUNK_CRATE2: usize = 6;
/// Source: `oracle/codemp/cgame/cg_local.h:1048-1059`
const CHUNK_WHITE_METAL: usize = 7;

/// Raven `CGDEBUG_SaberColor` — debug-draw color for a saber blade color enum,
/// packed `0x00bbggrr`.
///
/// Source: `oracle/codemp/cgame/cg_effects.c:135-161`
pub fn CGDEBUG_SaberColor(saberColor: c_int) -> c_int {
    match saberColor {
        SABER_RED => 0x000000ff,
        SABER_ORANGE => 0x000088ff,
        SABER_YELLOW => 0x0000ffff,
        SABER_GREEN => 0x0000ff00,
        SABER_BLUE => 0x00ff0000,
        SABER_PURPLE => 0x00ff00ff,
        _ => saberColor,
    }
}

/// Raven `CG_DoGlassQuad` — spawns one glass-shard poly via `trap_FX_AddPoly`,
/// tumbling under gravity with a bit of bounce.
///
/// Raven: "rww - this is dirty." — the ideal `FX_AddPoly`/`CPoly` path above
/// this function is dead/commented-out in the oracle; this transcribes only
/// the live `addpolyArgStruct_t` path Raven actually ships.
///
/// Source: `oracle/codemp/cgame/cg_effects.c:261-365`
pub fn CG_DoGlassQuad(
    ctx: &mut CgContext,
    p: &[vec3_t; 4],
    uv: &[vec2_t; 4],
    stick: bool,
    time: c_int,
    dmgDir: &vec3_t,
) {
    let mut vel = [0.0f32; 3];
    VectorSet(
        &mut vel,
        (ctx.world.bg_state.rng.crandom() * 12.0) as f32,
        (ctx.world.bg_state.rng.crandom() * 12.0) as f32,
        -1.0,
    );

    if !stick {
        // We aren't a motion delayed chunk, so let us move quickly
        _VectorMA(vel, 0.3, *dmgDir, &mut vel);
    }

    // Set up acceleration due to gravity, 800 is standard QuakeIII gravity, so let's use something close
    let mut accel = [0.0f32; 3];
    VectorSet(
        &mut accel,
        0.0,
        0.0,
        -(600.0 + ctx.world.bg_state.rng.random() * 100.0),
    );

    // We are using an additive shader, so let's set the RGB low so we look more like transparent glass
    let rgb1: vec3_t = [1.0, 1.0, 1.0];

    // Being glass, we don't want to bounce much
    let bounce = ctx.world.bg_state.rng.random() * 0.2 + 0.15;

    // Set up our random rotate, we only do PITCH and YAW, not ROLL. This is something like degrees per second
    let mut rotationDelta = [0.0f32; 3];
    VectorSet(
        &mut rotationDelta,
        (ctx.world.bg_state.rng.crandom() * 40.0) as f32,
        (ctx.world.bg_state.rng.crandom() * 40.0) as f32,
        0.0,
    );

    let mut apArgs = addpolyArgStruct_t {
        p: *p,
        ev: *uv,
        numVerts: 4,
        vel,
        accel,
        alpha1: 0.15,
        alpha2: 0.0,
        alphaParm: 85.0,
        rgb1,
        rgb2: rgb1,
        rgbParm: 0.0,
        rotationDelta,
        bounce,
        motionDelay: time,
        killTime: 6000,
        shader: ctx.world.cgs.media.glassShardShader,
        flags: FX_APPLY_PHYSICS | FX_ALPHA_NONLINEAR | FX_USE_ALPHA,
    };

    trap::FX_AddPoly(ctx.engine, &mut apArgs);
}

/// Raven `CG_CalcBiLerp` — bilinearly interpolates the four corner `verts` at
/// each of `uv`'s four sample points into `subVerts` (Raven's out-param
/// becomes the return per §C7).
///
/// Source: `oracle/codemp/cgame/cg_effects.c:367-399`
pub fn CG_CalcBiLerp(verts: &[vec3_t; 4], uv: &[vec2_t; 4]) -> [vec3_t; 4] {
    let mut subVerts = [[0.0f32; 3]; 4];
    let mut temp = [0.0f32; 3];

    // Nasty crap
    for i in 0..4 {
        _VectorScale(verts[0], 1.0 - uv[i][0], &mut subVerts[i]);
        _VectorMA(subVerts[i], uv[i][0], verts[1], &mut subVerts[i]);
        _VectorScale(subVerts[i], 1.0 - uv[i][1], &mut temp);
        _VectorScale(verts[3], 1.0 - uv[i][0], &mut subVerts[i]);
        _VectorMA(subVerts[i], uv[i][0], verts[2], &mut subVerts[i]);
        _VectorMA(temp, uv[i][1], subVerts[i], &mut subVerts[i]);
    }

    subVerts
}

/// Raven `CG_CalcHeightWidth` — the average cross-product-derived height and
/// width of the `verts` quad (Raven's two out-params become the `(height,
/// width)` return per §C7, matching the declared param order).
///
/// PORT-NOTE: Raven normalizes `dir1` in place on the first cross product of
/// each pair, then reuses the now-unit-length `dir1` for the second cross
/// product — not a fresh direction vector. Preserved verbatim; it's what the
/// oracle actually computes.
///
/// Source: `oracle/codemp/cgame/cg_effects.c:404-425`
pub fn CG_CalcHeightWidth(verts: &[vec3_t; 4]) -> (f32, f32) {
    let mut dir1 = [0.0f32; 3];
    let mut dir2 = [0.0f32; 3];
    let mut cross = [0.0f32; 3];

    _VectorSubtract(verts[3], verts[0], &mut dir1); // v
    _VectorSubtract(verts[1], verts[0], &mut dir2); // p-a
    CrossProduct(dir1, dir2, &mut cross);
    let mut width = VectorNormalize(&mut cross) / VectorNormalize(&mut dir1); // v
    _VectorSubtract(verts[2], verts[0], &mut dir2); // p-a
    CrossProduct(dir1, dir2, &mut cross);
    width += VectorNormalize(&mut cross) / VectorNormalize(&mut dir1); // v
    width *= 0.5;

    _VectorSubtract(verts[1], verts[0], &mut dir1); // v
    _VectorSubtract(verts[2], verts[0], &mut dir2); // p-a
    CrossProduct(dir1, dir2, &mut cross);
    let mut height = VectorNormalize(&mut cross) / VectorNormalize(&mut dir1); // v
    _VectorSubtract(verts[3], verts[0], &mut dir2); // p-a
    CrossProduct(dir1, dir2, &mut cross);
    height += VectorNormalize(&mut cross) / VectorNormalize(&mut dir1); // v
    height *= 0.5;

    (height, width)
}

/// Raven `CG_InitGlass` — builds the 20x20 random crack-offset table once up
/// front, so the glass tesselation looks less predictable without paying the
/// random cost per-shatter.
///
/// Source: `oracle/codemp/cgame/cg_effects.c:430-444`
pub fn CG_InitGlass(world: &mut CgWorld) {
    // Build a table first, so that we can do a more unpredictable crack scheme
    // do it once, up front to save a bit of time.
    for i in 0..20usize {
        for t in 0..20usize {
            world.effects.offX[t][i] = (world.bg_state.rng.crandom() * 0.03) as f32;
            world.effects.offZ[i][t] = (world.bg_state.rng.crandom() * 0.03) as f32;
        }
    }
}

/// Raven `Vector2Set` — writes `(b, c)` into `a`. Kept as an out-param
/// receiver (not a return) to match `VectorSet`'s established 3D twin in this
/// tree.
///
/// Source: `oracle/codemp/cgame/cg_effects.c:446-450`
pub fn Vector2Set(a: &mut vec2_t, b: f32, c: f32) {
    a[0] = b;
    a[1] = c;
}

/// Raven `CG_MiscModelExplosion` — spawns a `material_t`-flavored debris burst
/// of registered particle effects roughly inside `[mins, maxs]`, shot outward
/// from the box center.
///
/// Source: `oracle/codemp/cgame/cg_effects.c:949-1051`
pub fn CG_MiscModelExplosion(
    ctx: &mut CgContext,
    mins: vec3_t,
    maxs: vec3_t,
    size: c_int,
    chunkType: material_t,
) {
    let mut ct: c_int = 13;
    let mut effect: Option<&str> = None;
    let mut effect2: Option<&str> = None;

    let mut mid = [0.0f32; 3];
    _VectorAdd(mins, maxs, &mut mid);
    _VectorScale(mid, 0.5, &mut mid);

    match chunkType {
        MAT_GLASS => {
            effect = Some("chunks/glassbreak");
            ct = 5;
        }
        MAT_GLASS_METAL => {
            effect = Some("chunks/glassbreak");
            effect2 = Some("chunks/metalexplode");
            ct = 5;
        }
        MAT_ELECTRICAL | MAT_ELEC_METAL => {
            effect = Some("chunks/sparkexplode");
            ct = 5;
        }
        MAT_METAL | MAT_METAL2 | MAT_METAL3 | MAT_CRATE1 | MAT_CRATE2 => {
            effect = Some("chunks/metalexplode");
            ct = 2;
        }
        MAT_GRATE1 => {
            effect = Some("chunks/grateexplode");
            ct = 8;
        }
        MAT_ROPE => {
            ct = 20;
            effect = Some("chunks/ropebreak");
        }
        // not sure what this crap is really supposed to be..
        MAT_WHITE_METAL | MAT_DRK_STONE | MAT_LT_STONE | MAT_GREY_STONE | MAT_SNOWY_ROCK => {
            effect = Some(match size {
                2 => "chunks/rockbreaklg",
                // 1 and default
                _ => "chunks/rockbreakmed",
            });
        }
        _ => {}
    }

    let Some(effect) = effect else {
        return;
    };

    ct += 7 * size;

    // FIXME: real precache .. VERify that these need to be here...don't think they would because
    // the effects should be registered in g_breakable
    // rww - No they don't.. indexed effects gameside get precached on load clientside, as server
    // objects are setup before client asset load time. However, we need to index them, so..
    let eID1 = trap::FX_RegisterEffect(ctx.engine, effect);

    let has_effect2 = effect2.is_some_and(|e| !e.is_empty());
    let eID2 = if has_effect2 {
        // FIXME: real precache
        trap::FX_RegisterEffect(ctx.engine, effect2.unwrap())
    } else {
        0
    };

    // spawn chunk roughly in the bbox of the thing..
    for _ in 0..ct {
        let mut org = [0.0f32; 3];
        for j in 0..3 {
            let r = ctx.world.bg_state.rng.random() * 0.8 + 0.1;
            org[j] = r * mins[j] + (1.0 - r) * maxs[j];
        }

        // shoot effect away from center
        let mut dir = [0.0f32; 3];
        _VectorSubtract(org, mid, &mut dir);
        VectorNormalize(&mut dir);

        if has_effect2 && (ctx.world.bg_state.rng.rand() & 1) != 0 {
            trap::FX_PlayEffectID(ctx.engine, eID2, &org, &dir, -1, -1);
        } else {
            trap::FX_PlayEffectID(ctx.engine, eID1, &org, &dir, -1, -1);
        }
    }
}

/// Raven `CG_DoGlass` — tesselates a shattered window (`verts`) into glass
/// shard polys, denser and longer-lived near the impact (`dmgPt`), picking a
/// coarser or finer "LOD" from the window's height/width.
///
/// PORT-NOTE: `mxWidth` scales with the brush's `width` (unlike the fixed
/// `mxHeight` steps above it) and can run past the 20-wide `offX`/`offZ`
/// crack table `CG_InitGlass` built — the oracle then reads past the table;
/// the row side has the same edge (`i + 1` can reach 20 against the 20-row
/// table). §F19: the `ix()` clamp pins BOTH indices to the table's last
/// entry instead of reproducing the out-of-bounds reads.
///
/// `normal` is accepted (Raven's own signature) but never read in the body.
///
/// Source: `oracle/codemp/cgame/cg_effects.c:456-648`
#[allow(unused_variables)]
pub fn CG_DoGlass(
    ctx: &mut CgContext,
    verts: &[vec3_t; 4],
    normal: &vec3_t,
    dmgPt: &vec3_t,
    dmgDir: &vec3_t,
    dmgRadius: f32,
    maxShards: c_int,
) {
    // To do a smarter tesselation, we should figure out the relative height and width of the brush face,
    //	then use this to pick a lod value from 1-3 in each axis.  This will give us 1-9 lod levels, which will
    //	hopefully be sufficient.
    let (height, width) = CG_CalcHeightWidth(verts);

    let sfx = trap::S_RegisterSound(ctx.engine, "sound/effects/glassbreak1.wav");
    trap::S_StartSound(ctx.engine, Some(dmgPt), -1, CHAN_AUTO, sfx);

    // Pick "LOD" for height
    let stepHeight: f32;
    let mxHeight: c_int;
    let mut timeDecay: f32;
    if height < 100.0 {
        stepHeight = 0.2;
        mxHeight = 5;
        timeDecay = TIME_DECAY_SLOW;
    } else if height > 220.0 {
        stepHeight = 0.05;
        mxHeight = 20;
        timeDecay = TIME_DECAY_FAST;
    } else {
        stepHeight = 0.1;
        mxHeight = 10;
        timeDecay = TIME_DECAY_MED;
    }

    // Attempt to scale the glass directly to the size of the window
    let mut stepWidth = (0.25 - (width as f64 * 0.0002)) as f32; //(width*0.0005));
    let mut mxWidth = (width as f64 * 0.2) as c_int;
    timeDecay = (timeDecay + TIME_DECAY_FAST) * 0.5;

    if stepWidth < 0.01 {
        stepWidth = 0.01;
    }
    if mxWidth < 5 {
        mxWidth = 5;
    }

    // mxWidth can exceed the 20-wide offX/offZ table (see doc comment above);
    // clamp instead of reading past it.
    let ix = |v: c_int| -> usize { v.clamp(0, 19) as usize };

    let mut glassShards: c_int = 0;
    let mut z = 0.0f32;
    let mut i: c_int = 0;
    while z < 1.0 {
        let mut x = 0.0f32;
        let mut t: c_int = 0;
        while x < 1.0 {
            let mut biPoints = [[0.0f32; 2]; 4];

            // This is nasty..
            let xx = if t > 0 && t < mxWidth {
                x - ctx.world.effects.offX[ix(i)][ix(t)]
            } else {
                x
            };
            let zz = if i > 0 && i < mxHeight {
                z - ctx.world.effects.offZ[ix(t)][ix(i)]
            } else {
                z
            };
            Vector2Set(&mut biPoints[0], xx, zz);

            let xx = if t + 1 > 0 && t + 1 < mxWidth {
                x - ctx.world.effects.offX[ix(i)][ix(t + 1)]
            } else {
                x
            };
            let zz = if i > 0 && i < mxHeight {
                z - ctx.world.effects.offZ[ix(t + 1)][ix(i)]
            } else {
                z
            };
            Vector2Set(&mut biPoints[1], xx + stepWidth, zz);

            let xx = if t + 1 > 0 && t + 1 < mxWidth {
                x - ctx.world.effects.offX[ix(i + 1)][ix(t + 1)]
            } else {
                x
            };
            let zz = if i + 1 > 0 && i + 1 < mxHeight {
                z - ctx.world.effects.offZ[ix(t + 1)][ix(i + 1)]
            } else {
                z
            };
            Vector2Set(&mut biPoints[2], xx + stepWidth, zz + stepHeight);

            let xx = if t > 0 && t < mxWidth {
                x - ctx.world.effects.offX[ix(i + 1)][ix(t)]
            } else {
                x
            };
            let zz = if i + 1 > 0 && i + 1 < mxHeight {
                z - ctx.world.effects.offZ[ix(t)][ix(i + 1)]
            } else {
                z
            };
            Vector2Set(&mut biPoints[3], xx, zz + stepHeight);

            let subVerts = CG_CalcBiLerp(verts, &biPoints);

            let mut dif = DistanceSquared(subVerts[0], *dmgPt) * timeDecay
                - ctx.world.bg_state.rng.random() * 32.0;

            // If we decrease dif, we are increasing the impact area, making it more likely to blow out large holes
            dif -= dmgRadius * dmgRadius;

            let (stick, time) = if dif > 1.0 {
                (
                    true,
                    (dif + ctx.world.bg_state.rng.random() * 200.0) as c_int,
                )
            } else {
                (false, 0)
            };

            CG_DoGlassQuad(ctx, &subVerts, &biPoints, stick, time, dmgDir);
            glassShards += 1;

            if maxShards != 0 && glassShards >= maxShards {
                return;
            }

            x += stepWidth;
            t += 1;
        }
        z += stepHeight;
        i += 1;
    }
}

/// Raven `CG_ExplosionEffects` — shakes the camera in proportion to how close
/// the view is to an explosion at `origin`, falling off linearly to nothing
/// at `radius`.
///
/// Source: `oracle/codemp/cgame/cg_effects.c:919-939`
pub fn CG_ExplosionEffects(
    world: &mut CgWorld,
    origin: &vec3_t,
    intensity: f32,
    radius: c_int,
    time: c_int,
) {
    // FIXME: When exactly is the vieworg calculated in relation to the rest of the frame?s

    let mut dir = [0.0f32; 3];
    _VectorSubtract(world.cg.refdef.vieworg, *origin, &mut dir);
    let dist = VectorNormalize(&mut dir);

    // Use the dir to add kick to the explosion

    if dist > radius as f32 {
        return;
    }

    let intensityScale = 1.0 - (dist / radius as f32);
    let realIntensity = intensity * intensityScale;

    CGCam_Shake(world, realIntensity, time);
}

/// Raven `CG_GlassShatter` — tesselates the `entnum` brush model's window into
/// glass shards, if it actually has one registered.
///
/// Raven: "otherwise something awful has happened." — the missing-model arm
/// is a no-op, matching the oracle's silent bail.
///
/// Source: `oracle/codemp/cgame/cg_effects.c:656-666`
pub fn CG_GlassShatter(
    ctx: &mut CgContext,
    entnum: usize,
    dmgPt: &vec3_t,
    dmgDir: &vec3_t,
    dmgRadius: f32,
    maxShards: c_int,
) {
    let modelindex = ctx.world.entities[entnum].currentState.modelindex as usize;
    let bmodel = ctx.world.cgs.inline_draw_model(modelindex);
    if bmodel != 0 {
        let mut verts = [[0.0f32; 3]; 4];
        let normal = [0.0f32; 3];
        trap::R_GetBModelVerts(ctx.engine, bmodel, &mut verts, &normal);
        CG_DoGlass(ctx, &verts, &normal, dmgPt, dmgDir, dmgRadius, maxShards);
    }
    // otherwise something awful has happened.
}

/// Raven `CG_BubbleTrail` — strings water bubbles along the `start`..`end`
/// segment, one every `spacing` units, drifting up and outward.
///
/// PORT-NOTE: `re->customShader = 0` — Raven commented out
/// `cgs.media.waterBubbleShader`, so the trail spawns shaderless. Preserved.
///
/// Source: `oracle/codemp/cgame/cg_effects.c:15-69`
pub fn CG_BubbleTrail(world: &mut CgWorld, start: &vec3_t, end: &vec3_t, spacing: f32) {
    if world.cvars.cg_noProjectileTrail.integer != 0 {
        return;
    }

    let mut r#move: vec3_t = [0.0; 3];
    let mut vec: vec3_t = [0.0; 3];
    _VectorCopy(*start, &mut r#move);
    _VectorSubtract(*end, *start, &mut vec);
    let len = VectorNormalize(&mut vec);

    // advance a random amount first
    // §F19: Raven's `rand() % (int)spacing` divides by zero when a caller passes
    // spacing < 1 — SIGFPE on x86, a panic here. Same crash, no invented guard.
    let mut i = world.bg_state.rng.rand() % spacing as c_int;
    _VectorMA(r#move, i as f32, vec, &mut r#move);

    _VectorScale(vec, spacing, &mut vec);

    while (i as f32) < len {
        let handle = CG_AllocLocalEntity(world);
        let time = world.cg.time;
        let endRnd = world.bg_state.rng.random();
        let trDelta = [
            (world.bg_state.rng.crandom() * 5.0) as f32,
            (world.bg_state.rng.crandom() * 5.0) as f32,
            (world.bg_state.rng.crandom() * 5.0 + 6.0) as f32,
        ];

        let le = world
            .cg_localEntities
            .get_mut(handle)
            .expect("CG_BubbleTrail: fresh slot");
        le.leFlags = leFlag_t::LEF_PUFF_DONT_SCALE as c_int;
        le.leType = leType_t::LE_MOVE_SCALE_FADE;
        le.startTime = time;
        le.endTime = ((time + 1000) as f32 + endRnd * 250.0) as c_int;
        le.lifeRate = (1.0 / (le.endTime - le.startTime) as f64) as f32;

        le.refEntity.shaderTime = time as f32 / 1000.0;

        le.refEntity.reType = refEntityType_t::RT_SPRITE;
        le.refEntity.rotation = 0.0;
        le.refEntity.radius = 3.0;
        le.refEntity.customShader = 0; //cgs.media.waterBubbleShader;
        le.refEntity.shaderRGBA[0] = 0xff;
        le.refEntity.shaderRGBA[1] = 0xff;
        le.refEntity.shaderRGBA[2] = 0xff;
        le.refEntity.shaderRGBA[3] = 0xff;

        le.color[3] = 1.0;

        le.pos.trType = trType_t::TR_LINEAR;
        le.pos.trTime = time;
        _VectorCopy(r#move, &mut le.pos.trBase);
        le.pos.trDelta = trDelta;

        _VectorAdd(r#move, vec, &mut r#move);
        i = (i as f32 + spacing) as c_int;
    }
}

/// Raven `CG_SmokePuff` — spawns one move-scale-fade sprite and hands back its
/// pool slot so the caller can keep tuning it.
///
/// Raven: the commented-out `int fadeInTime = startTime + duration / 2;` was
/// promoted to a parameter.
///
/// Source: `oracle/codemp/cgame/cg_effects.c:78-133`
#[allow(clippy::too_many_arguments)]
pub fn CG_SmokePuff(
    world: &mut CgWorld,
    p: &vec3_t,
    vel: &vec3_t,
    radius: f32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
    duration: f32,
    startTime: c_int,
    fadeInTime: c_int,
    leFlags: c_int,
    hShader: qhandle_t,
) -> EffectHandle {
    let handle = CG_AllocLocalEntity(world);
    let rotation = Q_random(&mut world.effects.seed) * 360.0;

    let le = world
        .cg_localEntities
        .get_mut(handle)
        .expect("CG_SmokePuff: fresh slot");
    le.leFlags = leFlags;
    le.radius = radius;

    le.refEntity.rotation = rotation;
    le.refEntity.radius = radius;
    le.refEntity.shaderTime = startTime as f32 / 1000.0;

    le.leType = leType_t::LE_MOVE_SCALE_FADE;
    le.startTime = startTime;
    le.fadeInTime = fadeInTime;
    le.endTime = (startTime as f32 + duration) as c_int;
    if fadeInTime > startTime {
        le.lifeRate = (1.0 / (le.endTime - le.fadeInTime) as f64) as f32;
    } else {
        le.lifeRate = (1.0 / (le.endTime - le.startTime) as f64) as f32;
    }
    le.color[0] = r;
    le.color[1] = g;
    le.color[2] = b;
    le.color[3] = a;

    le.pos.trType = trType_t::TR_LINEAR;
    le.pos.trTime = startTime;
    _VectorCopy(*vel, &mut le.pos.trDelta);
    _VectorCopy(*p, &mut le.pos.trBase);

    _VectorCopy(*p, &mut le.refEntity.origin);
    le.refEntity.customShader = hShader;

    // x86 float->byte truncates through an int
    le.refEntity.shaderRGBA[0] = (le.color[0] * 0xff as f32) as i32 as u8;
    le.refEntity.shaderRGBA[1] = (le.color[1] * 0xff as f32) as i32 as u8;
    le.refEntity.shaderRGBA[2] = (le.color[2] * 0xff as f32) as i32 as u8;
    le.refEntity.shaderRGBA[3] = 0xff;

    le.refEntity.reType = refEntityType_t::RT_SPRITE;
    le.refEntity.radius = le.radius;

    handle
}

/// Raven `CG_TestLine` — debug-draws a `time`-msec line from `start` to `end`,
/// `color` 0 meaning plain white and anything else running through
/// [`CGDEBUG_SaberColor`].
///
/// Source: `oracle/codemp/cgame/cg_effects.c:163-204`
pub fn CG_TestLine(
    world: &mut CgWorld,
    start: &vec3_t,
    end: &vec3_t,
    time: c_int,
    color: c_uint,
    radius: c_int,
) {
    let handle = CG_AllocLocalEntity(world);
    let now = world.cg.time;
    let whiteShader = world.cgs.media.whiteShader;

    let le = world
        .cg_localEntities
        .get_mut(handle)
        .expect("CG_TestLine: fresh slot");
    le.leType = leType_t::LE_LINE;
    le.startTime = now;
    le.endTime = now + time;
    le.lifeRate = (1.0 / (le.endTime - le.startTime) as f64) as f32;

    _VectorCopy(*start, &mut le.refEntity.origin);
    _VectorCopy(*end, &mut le.refEntity.oldorigin);
    le.refEntity.shaderTime = now as f32 / 1000.0;

    le.refEntity.reType = refEntityType_t::RT_LINE;
    le.refEntity.radius = (0.5 * radius as f64) as f32;
    le.refEntity.customShader = whiteShader; //trap_R_RegisterShaderNoMip("textures/colombia/canvas_doublesided");

    le.refEntity.shaderTexCoord[1] = 1.0;
    le.refEntity.shaderTexCoord[0] = le.refEntity.shaderTexCoord[1];

    if color == 0 {
        le.refEntity.shaderRGBA[3] = 0xff;
        le.refEntity.shaderRGBA[2] = le.refEntity.shaderRGBA[3];
        le.refEntity.shaderRGBA[1] = le.refEntity.shaderRGBA[2];
        le.refEntity.shaderRGBA[0] = le.refEntity.shaderRGBA[1];
    } else {
        let mut color = CGDEBUG_SaberColor(color as c_int) as c_uint;
        le.refEntity.shaderRGBA[0] = (color & 0xff) as u8;
        color >>= 8;
        le.refEntity.shaderRGBA[1] = (color & 0xff) as u8;
        color >>= 8;
        le.refEntity.shaderRGBA[2] = (color & 0xff) as u8;
        //		color >>= 8;
        //		re->shaderRGBA[3] = color & 0xff;
        le.refEntity.shaderRGBA[3] = 0xff;
    }

    le.color[3] = 1.0;

    //re->renderfx |= RF_DEPTHHACK;
}

/// Raven `CG_ThrowChunk` — tosses one tumbling gravity fragment of `hModel`
/// out of `origin`, living 5-8 seconds.
///
/// Source: `oracle/codemp/cgame/cg_effects.c:211-243`
pub fn CG_ThrowChunk(
    world: &mut CgWorld,
    origin: &vec3_t,
    velocity: &vec3_t,
    hModel: qhandle_t,
    optionalSound: c_int,
    startalpha: c_int,
) {
    let handle = CG_AllocLocalEntity(world);
    let now = world.cg.time;
    let endRnd = world.bg_state.rng.random();

    let le = world
        .cg_localEntities
        .get_mut(handle)
        .expect("CG_ThrowChunk: fresh slot");

    le.leType = leType_t::LE_FRAGMENT;
    le.startTime = now;
    le.endTime = ((le.startTime + 5000) as f32 + endRnd * 3000.0) as c_int;

    _VectorCopy(*origin, &mut le.refEntity.origin);
    le.refEntity.axis = AXIS_DEFAULT;
    le.refEntity.hModel = hModel;

    le.pos.trType = trType_t::TR_GRAVITY;
    le.angles.trType = trType_t::TR_GRAVITY;
    _VectorCopy(*origin, &mut le.pos.trBase);
    _VectorCopy(*velocity, &mut le.pos.trDelta);
    VectorSet(&mut le.angles.trBase, 20.0, 20.0, 20.0);
    _VectorCopy(*velocity, &mut le.angles.trDelta);
    le.pos.trTime = now;
    le.angles.trTime = now;

    le.leFlags = leFlag_t::LEF_TUMBLE as c_int;

    le.angles.trBase[YAW] = 180.0;

    le.bounceFactor = 0.3;
    le.bounceSound = optionalSound;

    le.forceAlpha = startalpha;
}

/// Raven `CG_Chunks` — breaks a `chunkType` surface: plays the material's
/// break sound once, then throws `numChunks` tumbling model fragments out of
/// the `[mins, maxs]` box. The glass/grate/spark/rope materials are pure sound
/// — their debris is effects, not models, so they play and return.
///
/// `normal` is accepted (Raven's own signature) but never read in the body.
///
/// Once a `customChunk` model resolves, Raven's `chunk` latch stays set for the
/// whole loop, so every remaining fragment reuses it instead of re-rolling a
/// random one. Preserved.
///
/// Source: `oracle/codemp/cgame/cg_effects.c:1061-1254`
#[allow(unused_variables)]
#[allow(clippy::too_many_arguments)]
pub fn CG_Chunks(
    ctx: &mut CgContext,
    owner: c_int,
    origin: &vec3_t,
    normal: &vec3_t,
    mins: &vec3_t,
    maxs: &vec3_t,
    speed: f32,
    numChunks: c_int,
    chunkType: material_t,
    customChunk: c_int,
    baseScale: f32,
) {
    let mut chunkModel: qhandle_t = 0;
    let mut bounce = leBounceSoundType_t::LEBS_NONE;
    let mut speedMod = 1.0f32;
    let mut chunk = false;

    if chunkType == MAT_NONE {
        // Well, we should do nothing
        return;
    }

    // Set up our chunk sound info...breaking sounds are done here so they are done once on
    // breaking..some return instantly because the chunks are done with effects instead of models
    match chunkType {
        MAT_GLASS => {
            let sfx = ctx.world.cgs.media.glassChunkSound;
            trap::S_StartSound(ctx.engine, None, owner, CHAN_BODY, sfx);
            return;
        }
        MAT_GRATE1 => {
            let sfx = ctx.world.cgs.media.grateSound;
            trap::S_StartSound(ctx.engine, None, owner, CHAN_BODY, sfx);
            return;
        }
        // (sparks)
        MAT_ELECTRICAL => {
            let n = ctx.world.bg_state.rng.Q_irand(1, 6);
            let sfx = trap::S_RegisterSound(ctx.engine, &format!("sound/ambience/spark{n}.wav"));
            trap::S_StartSound(ctx.engine, None, owner, CHAN_BODY, sfx);
            return;
        }
        // MAT_WHITE_METAL: not quite sure what this stuff is supposed to be...it's for Stu
        MAT_DRK_STONE | MAT_LT_STONE | MAT_GREY_STONE | MAT_WHITE_METAL | MAT_SNOWY_ROCK => {
            let sfx = ctx.world.cgs.media.rockBreakSound;
            trap::S_StartSound(ctx.engine, None, owner, CHAN_BODY, sfx);
            bounce = leBounceSoundType_t::LEBS_ROCK;
            speedMod = 0.5; // rock blows up less
        }
        MAT_GLASS_METAL => {
            // Raven FIXME: should probably have a custom sound
            let sfx = ctx.world.cgs.media.glassChunkSound;
            trap::S_StartSound(ctx.engine, None, owner, CHAN_BODY, sfx);
            bounce = leBounceSoundType_t::LEBS_METAL;
        }
        MAT_CRATE1 | MAT_CRATE2 => {
            let i = ctx.world.bg_state.rng.Q_irand(0, 1) as usize;
            let sfx = ctx.world.cgs.media.crateBreakSound[i];
            trap::S_StartSound(ctx.engine, None, owner, CHAN_BODY, sfx);
        }
        // MAT_ELEC_METAL: Raven FIXME: maybe have its own sound?
        MAT_METAL | MAT_METAL2 | MAT_METAL3 | MAT_ELEC_METAL => {
            let sfx = ctx.world.cgs.media.chunkSound;
            trap::S_StartSound(ctx.engine, None, owner, CHAN_BODY, sfx);
            bounce = leBounceSoundType_t::LEBS_METAL;
            speedMod = 0.8; // metal blows up a bit more
        }
        MAT_ROPE => {
            //		trap_S_StartSound( NULL, owner, CHAN_BODY, cgi_S_RegisterSound( "" ));  FIXME:  needs a sound
            return;
        }
        _ => {}
    }

    let mut baseScale = baseScale;
    if baseScale <= 0.0 {
        baseScale = 1.0;
    }

    // Chunks
    for _i in 0..numChunks {
        if customChunk > 0 {
            // Try to use a custom chunk.
            if ctx.world.cgs.gameModels[customChunk as usize] != 0 {
                chunk = true;
                chunkModel = ctx.world.cgs.gameModels[customChunk as usize];
            }
        }

        if !chunk {
            // No custom chunk.  Pick a random chunk type at run-time so we don't get the same chunks
            match chunkType {
                // bluegrey
                MAT_METAL2 => {
                    let r = ctx.world.bg_state.rng.Q_irand(0, 3) as usize;
                    chunkModel = ctx.world.cgs.media.chunkModels[CHUNK_METAL2][r];
                }
                // gray
                MAT_GREY_STONE => {
                    let r = ctx.world.bg_state.rng.Q_irand(0, 3) as usize;
                    chunkModel = ctx.world.cgs.media.chunkModels[CHUNK_ROCK1][r];
                }
                // tan
                MAT_LT_STONE => {
                    let r = ctx.world.bg_state.rng.Q_irand(0, 3) as usize;
                    chunkModel = ctx.world.cgs.media.chunkModels[CHUNK_ROCK2][r];
                }
                // brown
                MAT_DRK_STONE => {
                    let r = ctx.world.bg_state.rng.Q_irand(0, 3) as usize;
                    chunkModel = ctx.world.cgs.media.chunkModels[CHUNK_ROCK3][r];
                }
                // gray & brown
                MAT_SNOWY_ROCK => {
                    if ctx.world.bg_state.rng.Q_irand(0, 1) != 0 {
                        let r = ctx.world.bg_state.rng.Q_irand(0, 3) as usize;
                        chunkModel = ctx.world.cgs.media.chunkModels[CHUNK_ROCK1][r];
                    } else {
                        let r = ctx.world.bg_state.rng.Q_irand(0, 3) as usize;
                        chunkModel = ctx.world.cgs.media.chunkModels[CHUNK_ROCK3][r];
                    }
                }
                MAT_WHITE_METAL => {
                    let r = ctx.world.bg_state.rng.Q_irand(0, 3) as usize;
                    chunkModel = ctx.world.cgs.media.chunkModels[CHUNK_WHITE_METAL][r];
                }
                // yellow multi-colored crate chunks
                MAT_CRATE1 => {
                    let r = ctx.world.bg_state.rng.Q_irand(0, 3) as usize;
                    chunkModel = ctx.world.cgs.media.chunkModels[CHUNK_CRATE1][r];
                }
                // red multi-colored crate chunks
                MAT_CRATE2 => {
                    let r = ctx.world.bg_state.rng.Q_irand(0, 3) as usize;
                    chunkModel = ctx.world.cgs.media.chunkModels[CHUNK_CRATE2][r];
                }
                // grey
                MAT_ELEC_METAL | MAT_GLASS_METAL | MAT_METAL => {
                    let r = ctx.world.bg_state.rng.Q_irand(0, 3) as usize;
                    chunkModel = ctx.world.cgs.media.chunkModels[CHUNK_METAL1][r];
                }
                MAT_METAL3 => {
                    if ctx.world.bg_state.rng.rand() & 1 != 0 {
                        let r = ctx.world.bg_state.rng.Q_irand(0, 3) as usize;
                        chunkModel = ctx.world.cgs.media.chunkModels[CHUNK_METAL1][r];
                    } else {
                        let r = ctx.world.bg_state.rng.Q_irand(0, 3) as usize;
                        chunkModel = ctx.world.cgs.media.chunkModels[CHUNK_METAL2][r];
                    }
                }
                _ => {}
            }
        }

        // It wouldn't look good to throw a bunch of RGB axis models...so make sure we have something to work with.
        if chunkModel != 0 {
            let handle = CG_AllocLocalEntity(ctx.world);
            let now = ctx.world.cg.time;
            let endRnd = ctx.world.bg_state.rng.random();

            {
                let le = ctx
                    .world
                    .cg_localEntities
                    .get_mut(handle)
                    .expect("CG_Chunks: fresh slot");
                le.refEntity.hModel = chunkModel;
                le.leType = leType_t::LE_FRAGMENT;
                le.endTime = ((now + 1300) as f32 + endRnd * 900.0) as c_int;
            }

            // spawn chunk roughly in the bbox of the thing...bias towards center in case thing
            // blowing up doesn't complete fill its bbox.
            for j in 0..3 {
                let r = ctx.world.bg_state.rng.random() * 0.8 + 0.1;
                let le = ctx
                    .world
                    .cg_localEntities
                    .get_mut(handle)
                    .expect("CG_Chunks: live slot");
                le.refEntity.origin[j] = r * mins[j] + (1.0 - r) * maxs[j];
            }

            let scatter = ctx.world.bg_state.rng.flrand(speed * 0.5, speed * 1.25) * speedMod;
            let angBase = [
                ctx.world.bg_state.rng.random() * 360.0,
                ctx.world.bg_state.rng.random() * 360.0,
                ctx.world.bg_state.rng.random() * 360.0,
            ];
            let angDelta = [
                ctx.world.bg_state.rng.crandom() as f32,
                ctx.world.bg_state.rng.crandom() as f32,
                0.0, // don't do roll
            ];
            let angScale = ctx.world.bg_state.rng.random() * 600.0 + 200.0;
            let bounceFactor = 0.2 + ctx.world.bg_state.rng.random() * 0.2;
            let radius = ctx
                .world
                .bg_state
                .rng
                .flrand(baseScale * 0.75, baseScale * 1.25);

            let le = ctx
                .world
                .cg_localEntities
                .get_mut(handle)
                .expect("CG_Chunks: live slot");
            _VectorCopy(le.refEntity.origin, &mut le.pos.trBase);

            // Move out from center of thing, otherwise you can end up things moving across the
            // brush in an undesirable direction.  Visually looks wrong
            let mut dir: vec3_t = [0.0; 3];
            _VectorSubtract(le.refEntity.origin, *origin, &mut dir);
            VectorNormalize(&mut dir);
            _VectorScale(dir, scatter, &mut le.pos.trDelta);

            // Angular Velocity
            VectorSet(&mut le.angles.trBase, angBase[0], angBase[1], angBase[2]);

            le.angles.trDelta = angDelta;

            _VectorScale(le.angles.trDelta, angScale, &mut le.angles.trDelta);

            le.pos.trType = trType_t::TR_GRAVITY;
            le.angles.trType = trType_t::TR_LINEAR;
            le.angles.trTime = now;
            le.pos.trTime = le.angles.trTime;
            le.bounceFactor = bounceFactor;
            le.leFlags |= leFlag_t::LEF_TUMBLE as c_int;
            //le->ownerGentNum = owner;
            le.leBounceSoundType = bounce;

            // Make sure that we have the desired start size set
            le.radius = radius;
            le.refEntity.nonNormalizedAxes = qtrue;
            // could do an angles to axis, but this is cheaper and works ok
            le.refEntity.axis = AXIS_DEFAULT;
            for k in 0..3 {
                le.refEntity.modelScale[k] = le.radius;
            }
            ScaleModelAxis(&mut le.refEntity);
            /*
            for( k = 0; k < 3; k++ )
            {
                VectorScale( re->axis[k], le->radius, re->axis[k] );
            }
            */
        }
    }
}

/// Raven `CG_ScorePlum` — floats the score number over `org`, only for the
/// client that actually scored, nudging it down when the last plum was at
/// nearly the same height.
///
/// Source: `oracle/codemp/cgame/cg_effects.c:1261-1299`
pub fn CG_ScorePlum(world: &mut CgWorld, client: c_int, org: &vec3_t, score: c_int) {
    // only visualize for the client that scored
    if client != world.cg.predictedPlayerState.clientNum || world.cvars.cg_scorePlum.integer == 0 {
        return;
    }

    let handle = CG_AllocLocalEntity(world);
    let now = world.cg.time;
    let lastPos = world.effects.lastPos;

    let le = world
        .cg_localEntities
        .get_mut(handle)
        .expect("CG_ScorePlum: fresh slot");
    le.leFlags = 0;
    le.leType = leType_t::LE_SCOREPLUM;
    le.startTime = now;
    le.endTime = now + 4000;
    le.lifeRate = (1.0 / (le.endTime - le.startTime) as f64) as f32;

    le.color[3] = 1.0;
    le.color[2] = le.color[3];
    le.color[1] = le.color[2];
    le.color[0] = le.color[1];
    le.radius = score as f32;

    _VectorCopy(*org, &mut le.pos.trBase);
    if org[2] >= lastPos[2] - 20.0 && org[2] <= lastPos[2] + 20.0 {
        le.pos.trBase[2] -= 20.0;
    }

    //CG_Printf( "Plum origin %i %i %i -- %i\n", (int)org[0], (int)org[1], (int)org[2], (int)Distance(org, lastPos));
    _VectorCopy(*org, &mut world.effects.lastPos);

    let le = world
        .cg_localEntities
        .get_mut(handle)
        .expect("CG_ScorePlum: live slot");

    le.refEntity.reType = refEntityType_t::RT_SPRITE;
    le.refEntity.radius = 16.0;

    let mut angles: vec3_t = [0.0; 3];
    VectorClear(&mut angles);
    AnglesToAxis(angles, le.refEntity.axis.as_mut_ptr());
}

/// Raven `CG_MakeExplosion` — spawns a model or sprite explosion at `origin`,
/// time-skewed so a burst of them doesn't animate in lockstep, and hands back
/// its pool slot.
///
/// `dir` is Raven's nullable `vec3_t` pointer — the non-sprite arm tests it and
/// falls back to the identity basis. The `msec <= 0` arm is Raven's `CG_Error`,
/// which longjmps out; ours returns, so the slot never gets allocated and the
/// return is `None`.
///
/// Source: `oracle/codemp/cgame/cg_effects.c:1306-1373`
#[allow(clippy::too_many_arguments)]
pub fn CG_MakeExplosion(
    ctx: &mut CgContext,
    origin: &vec3_t,
    dir: Option<&vec3_t>,
    hModel: qhandle_t,
    numFrames: c_int,
    shader: qhandle_t,
    msec: c_int,
    isSprite: bool,
    scale: f32,
    flags: c_int,
) -> Option<EffectHandle> {
    let mut ang = 0.0f32;
    let mut tmpVec: vec3_t = [0.0; 3];
    let mut newOrigin: vec3_t = [0.0; 3];

    if msec <= 0 {
        CG_Error(ctx, &format!("CG_MakeExplosion: msec = {msec}"));
        // Raven's CG_Error longjmps out; ours returns, so stop here
        return None;
    }

    // skew the time a bit so they aren't all in sync
    let offset = ctx.world.bg_state.rng.rand() & 63;

    let handle = CG_AllocLocalEntity(ctx.world);
    let now = ctx.world.cg.time;

    // the rotation draw below is conditional in Raven, so it stays inline -
    // hoisting it would advance the shared stream on paths that never rolled
    let ex = ctx
        .world
        .cg_localEntities
        .get_mut(handle)
        .expect("CG_MakeExplosion: fresh slot");
    if isSprite {
        ex.leType = leType_t::LE_SPRITE_EXPLOSION;
        ex.refEntity.rotation = (ctx.world.bg_state.rng.rand() % 360) as f32;
        ex.radius = scale;
        // §F19: Raven dereferences `dir` here without the NULL guard its `else`
        // arm has, so a NULL `dir` with `isSprite` is a crash in the oracle; we
        // take the zero vector, leaving `newOrigin` at `origin`.
        let d = dir.copied().unwrap_or([0.0; 3]);
        _VectorScale(d, 16.0, &mut tmpVec);
        _VectorAdd(tmpVec, *origin, &mut newOrigin);
    } else {
        ex.leType = leType_t::LE_EXPLOSION;
        _VectorCopy(*origin, &mut newOrigin);

        // set axis with random rotate when necessary
        match dir {
            None => {
                AxisClear(ex.refEntity.axis.as_mut_ptr());
            }
            Some(dir) => {
                if (flags & leFlag_t::LEF_NO_RANDOM_ROTATE as c_int) == 0 {
                    ang = (ctx.world.bg_state.rng.rand() % 360) as f32;
                }
                _VectorCopy(*dir, &mut ex.refEntity.axis[0]);
                RotateAroundDirection(ex.refEntity.axis.as_mut_ptr(), ang);
            }
        }
    }

    ex.startTime = now - offset;
    ex.endTime = ex.startTime + msec;

    // bias the time so all shader effects start correctly
    ex.refEntity.shaderTime = ex.startTime as f32 / 1000.0;

    ex.refEntity.hModel = hModel;
    ex.refEntity.customShader = shader;
    ex.lifeRate = numFrames as f32 / msec as f32;
    ex.leFlags = flags;

    //Scale the explosion
    if scale != 1.0 {
        ex.refEntity.nonNormalizedAxes = qtrue;

        _VectorScale(ex.refEntity.axis[0], scale, &mut ex.refEntity.axis[0]);
        _VectorScale(ex.refEntity.axis[1], scale, &mut ex.refEntity.axis[1]);
        _VectorScale(ex.refEntity.axis[2], scale, &mut ex.refEntity.axis[2]);
    }
    // set origin
    _VectorCopy(newOrigin, &mut ex.refEntity.origin);
    _VectorCopy(newOrigin, &mut ex.refEntity.oldorigin);

    ex.color[2] = 1.0;
    ex.color[1] = ex.color[2];
    ex.color[0] = ex.color[1];

    Some(handle)
}

/// Raven `CG_Bleed` — a half-second blood sprite at `origin`, hidden from the
/// bleeding player's own first-person view.
///
/// PORT-NOTE: `customShader = 0` — Raven commented out
/// `cgs.media.bloodExplosionShader`, so the sprite spawns shaderless.
/// Preserved.
///
/// Source: `oracle/codemp/cgame/cg_effects.c:1489-1513`
pub fn CG_Bleed(world: &mut CgWorld, origin: &vec3_t, entityNum: c_int) {
    if world.cvars.cg_blood.integer == 0 {
        return;
    }

    let handle = CG_AllocLocalEntity(world);
    let now = world.cg.time;
    let spin = world.bg_state.rng.rand() % 360;
    // §F19: Raven reads `cg.snap->ps` unguarded; with no snapshot yet there is
    // no local client to hide the blood from, so the arm just doesn't fire.
    let localClient = world.cg.snap_ref().map(|snap| snap.ps.clientNum);

    let ex = world
        .cg_localEntities
        .get_mut(handle)
        .expect("CG_Bleed: fresh slot");
    ex.leType = leType_t::LE_EXPLOSION;

    ex.startTime = now;
    ex.endTime = ex.startTime + 500;

    _VectorCopy(*origin, &mut ex.refEntity.origin);
    ex.refEntity.reType = refEntityType_t::RT_SPRITE;
    ex.refEntity.rotation = spin as f32;
    ex.refEntity.radius = 24.0;

    ex.refEntity.customShader = 0; //cgs.media.bloodExplosionShader;

    // don't show player's own blood in view
    if localClient == Some(entityNum) {
        ex.refEntity.renderfx |= RF_THIRD_PERSON;
    }
}

/// Raven `CG_GlassShatter_Old` — tumbles crandom-scattered glass chunks out of
/// a shattered `[mins, maxs]` window, throwing them until the accumulated
/// throw-count matches the window's rough size.
///
/// Source: `oracle/codemp/cgame/cg_effects.c:674-748`
pub fn CG_GlassShatter_Old(
    ctx: &mut CgContext,
    entnum: c_int,
    org: &vec3_t,
    mins: &vec3_t,
    maxs: &vec3_t,
) {
    let sfx = trap::S_RegisterSound(ctx.engine, "sound/effects/glassbreak1.wav");
    trap::S_StartSound(ctx.engine, Some(org), entnum, CHAN_BODY, sfx);

    let mut a = [0.0f32; 3];
    _VectorSubtract(*maxs, *mins, &mut a);

    // should give us some idea of how big the chunk of glass is
    let windowmass = VectorLength(a);

    let mut shardsthrow = 0.0f32;
    while shardsthrow < windowmass {
        let velocity: vec3_t = [
            (ctx.world.bg_state.rng.crandom() * 150.0) as f32,
            (ctx.world.bg_state.rng.crandom() * 150.0) as f32,
            150.0 + (ctx.world.bg_state.rng.crandom() * 75.0) as f32,
        ];

        let chunkname = format!(
            "models/chunks/glass/glchunks_{}.md3",
            ctx.world.bg_state.rng.Q_irand(1, 6)
        );
        let mut shardorg = *org;

        let mut dif = [0.0f32; 3];
        dif[0] = (maxs[0] - mins[0]) / 2.0;
        dif[1] = (maxs[1] - mins[1]) / 2.0;
        dif[2] = (maxs[2] - mins[2]) / 2.0;

        if dif[0] < 2.0 {
            dif[0] = 2.0;
        }
        if dif[1] < 2.0 {
            dif[1] = 2.0;
        }
        if dif[2] < 2.0 {
            dif[2] = 2.0;
        }

        let difx: vec3_t = [
            ctx.world
                .bg_state
                .rng
                .Q_irand(1, ((dif[0] as f64 * 0.9) * 2.0) as c_int) as f32,
            ctx.world
                .bg_state
                .rng
                .Q_irand(1, ((dif[1] as f64 * 0.9) * 2.0) as c_int) as f32,
            ctx.world
                .bg_state
                .rng
                .Q_irand(1, ((dif[2] as f64 * 0.9) * 2.0) as c_int) as f32,
        ];

        if difx[0] > dif[0] {
            shardorg[0] += difx[0] - dif[0];
        } else {
            shardorg[0] -= difx[0];
        }
        if difx[1] > dif[1] {
            shardorg[1] += difx[1] - dif[1];
        } else {
            shardorg[1] -= difx[1];
        }
        if difx[2] > dif[2] {
            shardorg[2] += difx[2] - dif[2];
        } else {
            shardorg[2] -= difx[2];
        }

        // CG_TestLine(org, shardorg, 5000, 0x0000ff, 3);

        let model = trap::R_RegisterModel(ctx.engine, &chunkname);
        CG_ThrowChunk(ctx.world, &shardorg, &velocity, model, 0, 254);

        shardsthrow += 10.0;
    }
}

/// Raven `CG_CreateDebris` — same tumbling-chunk toss as
/// [`CG_GlassShatter_Old`], but the model comes from `debrismodel` (or, for
/// the special-case negative sentinels, a lazily-registered debris-model
/// table picked at random each throw) instead of a fixed glass shard.
///
/// Source: `oracle/codemp/cgame/cg_effects.c:771-904`
#[allow(clippy::too_many_arguments)]
pub fn CG_CreateDebris(
    ctx: &mut CgContext,
    _entnum: c_int,
    org: &vec3_t,
    mins: &vec3_t,
    maxs: &vec3_t,
    debrissound: c_int,
    debrismodel: c_int,
) {
    let omodel = debrismodel;
    let mut debrismodel = debrismodel;

    if omodel == DEBRIS_SPECIALCASE_GLASS && ctx.world.effects.dbModels_Glass[0] == 0 {
        // glass no longer exists, using it for metal.
        ctx.world.effects.dbModels_Glass[0] =
            trap::R_RegisterModel(ctx.engine, "models/chunks/metal/metal1_1.md3");
        ctx.world.effects.dbModels_Glass[1] =
            trap::R_RegisterModel(ctx.engine, "models/chunks/metal/metal1_2.md3");
        ctx.world.effects.dbModels_Glass[2] =
            trap::R_RegisterModel(ctx.engine, "models/chunks/metal/metal1_3.md3");
        ctx.world.effects.dbModels_Glass[3] =
            trap::R_RegisterModel(ctx.engine, "models/chunks/metal/metal1_4.md3");
        ctx.world.effects.dbModels_Glass[4] =
            trap::R_RegisterModel(ctx.engine, "models/chunks/metal/metal2_1.md3");
        ctx.world.effects.dbModels_Glass[5] =
            trap::R_RegisterModel(ctx.engine, "models/chunks/metal/metal2_2.md3");
        ctx.world.effects.dbModels_Glass[6] =
            trap::R_RegisterModel(ctx.engine, "models/chunks/metal/metal2_3.md3");
        ctx.world.effects.dbModels_Glass[7] =
            trap::R_RegisterModel(ctx.engine, "models/chunks/metal/metal2_4.md3");
    }
    if omodel == DEBRIS_SPECIALCASE_WOOD && ctx.world.effects.dbModels_Wood[0] == 0 {
        ctx.world.effects.dbModels_Wood[0] =
            trap::R_RegisterModel(ctx.engine, "models/chunks/crate/crate1_1.md3");
        ctx.world.effects.dbModels_Wood[1] =
            trap::R_RegisterModel(ctx.engine, "models/chunks/crate/crate1_2.md3");
        ctx.world.effects.dbModels_Wood[2] =
            trap::R_RegisterModel(ctx.engine, "models/chunks/crate/crate1_3.md3");
        ctx.world.effects.dbModels_Wood[3] =
            trap::R_RegisterModel(ctx.engine, "models/chunks/crate/crate1_4.md3");
        ctx.world.effects.dbModels_Wood[4] =
            trap::R_RegisterModel(ctx.engine, "models/chunks/crate/crate2_1.md3");
        ctx.world.effects.dbModels_Wood[5] =
            trap::R_RegisterModel(ctx.engine, "models/chunks/crate/crate2_2.md3");
        ctx.world.effects.dbModels_Wood[6] =
            trap::R_RegisterModel(ctx.engine, "models/chunks/crate/crate2_3.md3");
        ctx.world.effects.dbModels_Wood[7] =
            trap::R_RegisterModel(ctx.engine, "models/chunks/crate/crate2_4.md3");
    }
    if omodel == DEBRIS_SPECIALCASE_CHUNKS && ctx.world.effects.dbModels_Chunks[0] == 0 {
        ctx.world.effects.dbModels_Chunks[0] =
            trap::R_RegisterModel(ctx.engine, "models/chunks/generic/chunks_1.md3");
        ctx.world.effects.dbModels_Chunks[1] =
            trap::R_RegisterModel(ctx.engine, "models/chunks/generic/chunks_2.md3");
    }
    if omodel == DEBRIS_SPECIALCASE_ROCK && ctx.world.effects.dbModels_Rocks[0] == 0 {
        ctx.world.effects.dbModels_Rocks[0] =
            trap::R_RegisterModel(ctx.engine, "models/chunks/rock/rock1_1.md3");
        ctx.world.effects.dbModels_Rocks[1] =
            trap::R_RegisterModel(ctx.engine, "models/chunks/rock/rock1_2.md3");
        ctx.world.effects.dbModels_Rocks[2] =
            trap::R_RegisterModel(ctx.engine, "models/chunks/rock/rock1_3.md3");
        ctx.world.effects.dbModels_Rocks[3] =
            trap::R_RegisterModel(ctx.engine, "models/chunks/rock/rock1_4.md3");
        /*
        dbModels_Rocks[4] = trap_R_RegisterModel("models/chunks/rock/rock2_1.md3");
        dbModels_Rocks[5] = trap_R_RegisterModel("models/chunks/rock/rock2_2.md3");
        dbModels_Rocks[6] = trap_R_RegisterModel("models/chunks/rock/rock2_3.md3");
        dbModels_Rocks[7] = trap_R_RegisterModel("models/chunks/rock/rock2_4.md3");
        dbModels_Rocks[8] = trap_R_RegisterModel("models/chunks/rock/rock3_1.md3");
        dbModels_Rocks[9] = trap_R_RegisterModel("models/chunks/rock/rock3_2.md3");
        dbModels_Rocks[10] = trap_R_RegisterModel("models/chunks/rock/rock3_3.md3");
        dbModels_Rocks[11] = trap_R_RegisterModel("models/chunks/rock/rock3_4.md3");
        */
    }

    let mut a = [0.0f32; 3];
    _VectorSubtract(*maxs, *mins, &mut a);

    // should give us some idea of how big the chunk of glass is
    let windowmass = VectorLength(a);

    let mut shardsthrow = 0.0f32;
    while shardsthrow < windowmass {
        let velocity: vec3_t = [
            (ctx.world.bg_state.rng.crandom() * 150.0) as f32,
            (ctx.world.bg_state.rng.crandom() * 150.0) as f32,
            150.0 + (ctx.world.bg_state.rng.crandom() * 75.0) as f32,
        ];

        if omodel == DEBRIS_SPECIALCASE_GLASS {
            let i = ctx
                .world
                .bg_state
                .rng
                .Q_irand(0, NUM_DEBRIS_MODELS_GLASS as c_int - 1) as usize;
            debrismodel = ctx.world.effects.dbModels_Glass[i];
        } else if omodel == DEBRIS_SPECIALCASE_WOOD {
            let i = ctx
                .world
                .bg_state
                .rng
                .Q_irand(0, NUM_DEBRIS_MODELS_WOOD as c_int - 1) as usize;
            debrismodel = ctx.world.effects.dbModels_Wood[i];
        } else if omodel == DEBRIS_SPECIALCASE_CHUNKS {
            let i = ctx
                .world
                .bg_state
                .rng
                .Q_irand(0, NUM_DEBRIS_MODELS_CHUNKS as c_int - 1) as usize;
            debrismodel = ctx.world.effects.dbModels_Chunks[i];
        } else if omodel == DEBRIS_SPECIALCASE_ROCK {
            let i = ctx
                .world
                .bg_state
                .rng
                .Q_irand(0, NUM_DEBRIS_MODELS_ROCKS as c_int - 1) as usize;
            debrismodel = ctx.world.effects.dbModels_Rocks[i];
        }

        let mut shardorg = *org;

        let mut dif = [0.0f32; 3];
        dif[0] = (maxs[0] - mins[0]) / 2.0;
        dif[1] = (maxs[1] - mins[1]) / 2.0;
        dif[2] = (maxs[2] - mins[2]) / 2.0;

        if dif[0] < 2.0 {
            dif[0] = 2.0;
        }
        if dif[1] < 2.0 {
            dif[1] = 2.0;
        }
        if dif[2] < 2.0 {
            dif[2] = 2.0;
        }

        let difx: vec3_t = [
            ctx.world
                .bg_state
                .rng
                .Q_irand(1, ((dif[0] as f64 * 0.9) * 2.0) as c_int) as f32,
            ctx.world
                .bg_state
                .rng
                .Q_irand(1, ((dif[1] as f64 * 0.9) * 2.0) as c_int) as f32,
            ctx.world
                .bg_state
                .rng
                .Q_irand(1, ((dif[2] as f64 * 0.9) * 2.0) as c_int) as f32,
        ];

        if difx[0] > dif[0] {
            shardorg[0] += difx[0] - dif[0];
        } else {
            shardorg[0] -= difx[0];
        }
        if difx[1] > dif[1] {
            shardorg[1] += difx[1] - dif[1];
        } else {
            shardorg[1] -= difx[1];
        }
        if difx[2] > dif[2] {
            shardorg[2] += difx[2] - dif[2];
        } else {
            shardorg[2] -= difx[2];
        }

        // CG_TestLine(org, shardorg, 5000, 0x0000ff, 3);

        CG_ThrowChunk(ctx.world, &shardorg, &velocity, debrismodel, debrissound, 0);

        shardsthrow += 10.0;
    }
}

/// Raven `CG_SurfaceExplosion` — a light-tagged core explosion model plus a
/// scatter of secondary ones and a camera shake at a surface impact; the
/// spark-trail and smoke-sprite/spawner/impact-mark work sits entirely inside
/// commented-out `FX_*` calls Raven itself never compiled (dead in the
/// oracle), so nothing but the loop bounds and RNG draws survive from those
/// blocks.
///
/// Source: `oracle/codemp/cgame/cg_effects.c:1388-1480`
pub fn CG_SurfaceExplosion(
    ctx: &mut CgContext,
    origin: &vec3_t,
    normal: &vec3_t,
    radius: f32,
    shake_speed: f32,
    smoke: bool,
) {
    // Sparks
    let numSparks = 16 + (ctx.world.bg_state.rng.random() * 16.0) as c_int;

    for _ in 0..numSparks {
        let scale = 0.25 + (ctx.world.bg_state.rng.random() * 2.0) as f32;
        let _dscale = -scale * 0.5;

        // particle = FX_AddTrail(...) / FXE_Spray(...) — dead in the oracle
        // (commented out), so nothing to transcribe past the RNG draws above.
    }

    // Smoke
    // Move this out a little from the impact surface
    let mut new_org = [0.0f32; 3];
    _VectorMA(*origin, 4.0, *normal, &mut new_org);
    let velocity: vec3_t = [0.0, 0.0, 16.0];

    for _ in 0..4 {
        let _temp_org: vec3_t = [
            new_org[0] + (ctx.world.bg_state.rng.crandom() * 16.0) as f32,
            new_org[1] + (ctx.world.bg_state.rng.crandom() * 16.0) as f32,
            new_org[2] + (ctx.world.bg_state.rng.random() * 4.0) as f32,
        ];
        let _temp_vel: vec3_t = [
            velocity[0] + (ctx.world.bg_state.rng.crandom() * 8.0) as f32,
            velocity[1] + (ctx.world.bg_state.rng.crandom() * 8.0) as f32,
            velocity[2] + (ctx.world.bg_state.rng.crandom() * 8.0) as f32,
        ];

        // FX_AddSprite(...) — dead in the oracle (commented out); the temp
        // org/vel computation above is kept for RNG-stream parity, matching
        // the dead-store convention (CG_DrawNewTeamInfo precedent).
    }

    // Core of the explosion

    // Orient the explosions to face the camera
    let mut direction = [0.0f32; 3];
    _VectorSubtract(ctx.world.cg.refdef.vieworg, *origin, &mut direction);
    VectorNormalize(&mut direction);

    // Tag the last one with a light
    let explosionModel = ctx.world.cgs.media.explosionModel;
    let surfaceExplosionShader = ctx.world.cgs.media.surfaceExplosionShader;
    let sizeRand = radius * 0.02 + (ctx.world.bg_state.rng.random() * 0.3) as f32;
    let handle = CG_MakeExplosion(
        ctx,
        origin,
        Some(&direction),
        explosionModel,
        6,
        surfaceExplosionShader,
        500,
        false,
        sizeRand,
        0,
    );
    if let Some(handle) = handle {
        let le = ctx
            .world
            .cg_localEntities
            .get_mut(handle)
            .expect("CG_SurfaceExplosion: fresh slot");
        le.light = 150.0;
        VectorSet(&mut le.lightColor, 0.9, 0.8, 0.5);
    }

    for _ in 0..NUM_EXPLOSIONS - 1 {
        let new_org: vec3_t = [
            origin[0]
                + (16.0 + (ctx.world.bg_state.rng.crandom() * 8.0) as f32)
                    * ctx.world.bg_state.rng.crandom() as f32,
            origin[1]
                + (16.0 + (ctx.world.bg_state.rng.crandom() * 8.0) as f32)
                    * ctx.world.bg_state.rng.crandom() as f32,
            origin[2]
                + (16.0 + (ctx.world.bg_state.rng.crandom() * 8.0) as f32)
                    * ctx.world.bg_state.rng.crandom() as f32,
        ];
        let explosionModel = ctx.world.cgs.media.explosionModel;
        let surfaceExplosionShader = ctx.world.cgs.media.surfaceExplosionShader;
        let rockRand = ctx.world.bg_state.rng.rand() & 99;
        let sizeRand = radius * 0.05 + (ctx.world.bg_state.rng.crandom() * 0.3) as f32;
        CG_MakeExplosion(
            ctx,
            &new_org,
            Some(&direction),
            explosionModel,
            6,
            surfaceExplosionShader,
            300 + rockRand,
            false,
            sizeRand,
            0,
        );
    }

    // Shake the camera
    CG_ExplosionEffects(ctx.world, origin, shake_speed, 350, 750);

    // The level designers wanted to be able to turn the smoke spawners off.  The rationale is that they
    //	want to blow up catwalks and such that fall down...when that happens, it shouldn't really leave a mark
    //	and a smoke spewer at the explosion point...
    if smoke {
        let mut temp_org = [0.0f32; 3];
        _VectorMA(*origin, -8.0, *normal, &mut temp_org);
        // FX_AddSpawner(...) — dead in the oracle (commented out).

        // Impact mark
        // FIXME: Replace mark
        // CG_ImpactMark(...) — dead in the oracle (commented out).
        let _ = temp_org;
    }
}

/// Raven `CG_LaunchGib` — throws a bouncing, blood-marking gib fragment of
/// `hModel` out of `origin`.
///
/// Source: `oracle/codemp/cgame/cg_effects.c:1522-1546`
pub fn CG_LaunchGib(world: &mut CgWorld, origin: &vec3_t, velocity: &vec3_t, hModel: qhandle_t) {
    let handle = CG_AllocLocalEntity(world);
    let now = world.cg.time;
    let endRnd = world.bg_state.rng.random();

    let le = world
        .cg_localEntities
        .get_mut(handle)
        .expect("CG_LaunchGib: fresh slot");

    le.leType = leType_t::LE_FRAGMENT;
    le.startTime = now;
    le.endTime = ((le.startTime + 5000) as f32 + endRnd * 3000.0) as c_int;

    _VectorCopy(*origin, &mut le.refEntity.origin);
    le.refEntity.axis = AXIS_DEFAULT;
    le.refEntity.hModel = hModel;

    le.pos.trType = trType_t::TR_GRAVITY;
    _VectorCopy(*origin, &mut le.pos.trBase);
    _VectorCopy(*velocity, &mut le.pos.trDelta);
    le.pos.trTime = now;

    le.bounceFactor = 0.6;

    le.leBounceSoundType = leBounceSoundType_t::LEBS_BLOOD;
    le.leMarkType = leMarkType_t::LEMT_BLOOD;
}
